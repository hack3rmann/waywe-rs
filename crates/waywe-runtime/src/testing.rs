//! Offscreen platform and GPU readback helpers for rendering golden tests.

use crate::{
    gpu::Wgpu,
    platform::{
        CurrentSurfaceTexture, MonitorName, PlatformEvent, PlatformEventSource, Scale, SurfaceInfo,
        SurfaceTexture, WaywePlatform, WayweSurface,
    },
};
use calloop::{EventIterator, Poll, PostAction, Readiness, Token, TokenFactory};
use glam::UVec2;
use image::{ImageBuffer, Rgba, RgbaImage};
use std::{
    collections::{BTreeMap, VecDeque},
    error::Error,
    fs,
    sync::{Arc, Mutex},
};

/// Default offscreen test surface size.
pub const DEFAULT_SIZE: UVec2 = UVec2::new(64, 64);

/// Default offscreen surface format (matches typical Vulkan swapchain-friendly UNORM).
pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// Injectable acquire results for [`OffscreenSurface`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InjectedAcquire {
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
    Suboptimal,
}

struct OffscreenSurfaceInner {
    size: UVec2,
    format: wgpu::TextureFormat,
    texture: wgpu::Texture,
    pending: VecDeque<InjectedAcquire>,
}

/// Texture-backed [`WayweSurface`] used by golden tests.
pub struct OffscreenSurface {
    inner: Mutex<OffscreenSurfaceInner>,
}

impl OffscreenSurface {
    pub fn new(device: &wgpu::Device, size: UVec2, format: wgpu::TextureFormat) -> Self {
        let texture = create_offscreen_texture(device, size, format);
        Self {
            inner: Mutex::new(OffscreenSurfaceInner {
                size,
                format,
                texture,
                pending: VecDeque::new(),
            }),
        }
    }

    pub fn inject(&self, result: InjectedAcquire) {
        self.inner.lock().unwrap().pending.push_back(result);
    }

    pub fn texture_clone(&self) -> wgpu::Texture {
        self.inner.lock().unwrap().texture.clone()
    }

    pub fn size(&self) -> UVec2 {
        self.inner.lock().unwrap().size
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.inner.lock().unwrap().format
    }
}

impl WayweSurface for OffscreenSurface {
    fn get_current_texture(&self) -> CurrentSurfaceTexture {
        let mut inner = self.inner.lock().unwrap();
        match inner.pending.pop_front() {
            Some(InjectedAcquire::Timeout) => CurrentSurfaceTexture::Timeout,
            Some(InjectedAcquire::Occluded) => CurrentSurfaceTexture::Occluded,
            Some(InjectedAcquire::Outdated) => CurrentSurfaceTexture::Outdated,
            Some(InjectedAcquire::Lost) => CurrentSurfaceTexture::Lost,
            Some(InjectedAcquire::Validation) => CurrentSurfaceTexture::Validation,
            Some(InjectedAcquire::Suboptimal) => {
                CurrentSurfaceTexture::Suboptimal(Box::new(OffscreenSurfaceTexture {
                    texture: inner.texture.clone(),
                }))
            }
            None => CurrentSurfaceTexture::Success(Box::new(OffscreenSurfaceTexture {
                texture: inner.texture.clone(),
            })),
        }
    }

    fn configure(&self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        let mut inner = self.inner.lock().unwrap();
        inner.size = UVec2::new(config.width, config.height);
        inner.format = config.format;
        inner.texture = create_offscreen_texture(device, inner.size, inner.format);
    }

    fn get_format(&self, _: &wgpu::Adapter) -> wgpu::TextureFormat {
        self.inner.lock().unwrap().format
    }

    fn get_config(&self, _: &wgpu::Adapter, physical_size: UVec2) -> wgpu::SurfaceConfiguration {
        let format = self.inner.lock().unwrap().format;
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            format,
            width: physical_size.x.max(1),
            height: physical_size.y.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![format.remove_srgb_suffix()],
            color_space: wgpu::SurfaceColorSpace::default(),
        }
    }
}

struct OffscreenSurfaceTexture {
    texture: wgpu::Texture,
}

impl SurfaceTexture for OffscreenSurfaceTexture {
    fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn present(self: Box<Self>, _: &wgpu::Queue) {}
}

/// Stub event source that never wakes the poll.
pub struct NullEventSource;

impl PlatformEventSource for NullEventSource {
    fn process_events(
        &mut self,
        _: Readiness,
        _: Token,
        _: &mut dyn FnMut(PlatformEvent),
    ) -> Result<PostAction, Box<dyn Error + Send + Sync>> {
        Ok(PostAction::Continue)
    }

    fn register(&mut self, _: &mut Poll, _: &mut TokenFactory) -> calloop::Result<()> {
        Ok(())
    }

    fn reregister(&mut self, _: &mut Poll, _: &mut TokenFactory) -> calloop::Result<()> {
        Ok(())
    }

    fn unregister(&mut self, _: &mut Poll) -> calloop::Result<()> {
        Ok(())
    }

    fn before_sleep(&mut self) -> calloop::Result<Option<(Readiness, Token)>> {
        Ok(None)
    }

    fn before_handle_events(&mut self, _: EventIterator<'_>) {}
}

struct OffscreenPlatformInner {
    gpu: Arc<Wgpu>,
    stored_events: VecDeque<PlatformEvent>,
    surfaces: BTreeMap<MonitorName, Arc<OffscreenSurface>>,
    format: wgpu::TextureFormat,
}

/// Mock [`WaywePlatform`] that creates texture-backed surfaces.
pub struct OffscreenPlatform {
    inner: Mutex<OffscreenPlatformInner>,
}

impl OffscreenPlatform {
    pub fn new(gpu: Arc<Wgpu>, monitors: impl IntoIterator<Item = SurfaceInfo>) -> Arc<Self> {
        Self::with_format(gpu, monitors, DEFAULT_FORMAT)
    }

    pub fn with_format(
        gpu: Arc<Wgpu>,
        monitors: impl IntoIterator<Item = SurfaceInfo>,
        format: wgpu::TextureFormat,
    ) -> Arc<Self> {
        let monitors: Vec<_> = monitors.into_iter().collect();
        let stored_events = monitors
            .iter()
            .cloned()
            .map(|info| PlatformEvent::MonitorPlugged { info })
            .collect();

        Arc::new(Self {
            inner: Mutex::new(OffscreenPlatformInner {
                gpu,
                stored_events,
                surfaces: BTreeMap::new(),
                format,
            }),
        })
    }

    pub fn single_monitor(gpu: Arc<Wgpu>) -> Arc<Self> {
        Self::new(
            gpu,
            [SurfaceInfo {
                monitor_name: MonitorName::from("DP-1"),
                physical_size: DEFAULT_SIZE,
                scale: Scale::ONE,
            }],
        )
    }

    pub fn push_event(&self, event: PlatformEvent) {
        self.inner.lock().unwrap().stored_events.push_back(event);
    }

    pub fn surface(&self, monitor_name: &str) -> Option<Arc<OffscreenSurface>> {
        self.inner
            .lock()
            .unwrap()
            .surfaces
            .get(monitor_name)
            .cloned()
    }

    pub fn inject(&self, monitor_name: &str, result: InjectedAcquire) {
        let surface = self
            .surface(monitor_name)
            .unwrap_or_else(|| panic!("no offscreen surface for {monitor_name}"));
        surface.inject(result);
    }
}

impl WaywePlatform for OffscreenPlatform {
    fn create_surface(&self, _: &wgpu::Instance, monitor_name: &str) -> Box<dyn WayweSurface> {
        let mut inner = self.inner.lock().unwrap();
        let size = inner
            .surfaces
            .get(monitor_name)
            .map(|s| s.size())
            .or_else(|| {
                inner.stored_events.iter().find_map(|event| match event {
                    PlatformEvent::MonitorPlugged { info }
                        if info.monitor_name.as_str() == monitor_name =>
                    {
                        Some(info.physical_size)
                    }
                    _ => None,
                })
            })
            .unwrap_or(DEFAULT_SIZE);

        let surface = Arc::new(OffscreenSurface::new(&inner.gpu.device, size, inner.format));
        inner
            .surfaces
            .insert(MonitorName::from(monitor_name), surface.clone());
        Box::new(OffscreenSurfaceHandle { surface })
    }

    fn event_source(&self) -> Box<dyn PlatformEventSource> {
        Box::new(NullEventSource)
    }

    fn drain_stored_events(&self, handle: &mut dyn FnMut(PlatformEvent)) {
        let mut inner = self.inner.lock().unwrap();
        while let Some(event) = inner.stored_events.pop_front() {
            handle(event);
        }
    }
}

/// Thin Arc wrapper so [`Wgpu`] owns a `Box<dyn WayweSurface>` that shares state with the platform map.
struct OffscreenSurfaceHandle {
    surface: Arc<OffscreenSurface>,
}

impl WayweSurface for OffscreenSurfaceHandle {
    fn get_current_texture(&self) -> CurrentSurfaceTexture {
        self.surface.get_current_texture()
    }

    fn configure(&self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.surface.configure(device, config)
    }

    fn get_format(&self, adapter: &wgpu::Adapter) -> wgpu::TextureFormat {
        self.surface.get_format(adapter)
    }

    fn get_config(
        &self,
        adapter: &wgpu::Adapter,
        physical_size: UVec2,
    ) -> wgpu::SurfaceConfiguration {
        self.surface.get_config(adapter, physical_size)
    }
}

fn create_offscreen_texture(
    device: &wgpu::Device,
    size: UVec2,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen-test-surface"),
        size: wgpu::Extent3d {
            width: size.x.max(1),
            height: size.y.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[format.remove_srgb_suffix()],
    })
}

/// Axis-aligned pixel rectangle (inclusive-exclusive).
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn full(size: UVec2) -> Self {
        Self {
            x: 0,
            y: 0,
            width: size.x,
            height: size.y,
        }
    }
}

/// Read a texture back as tightly packed RGBA8 (converts from BGRA if needed).
pub fn readback_rgba8(gpu: &Wgpu, texture: &wgpu::Texture) -> RgbaImage {
    let width = texture.width();
    let height = texture.height();
    let format = texture.format().remove_srgb_suffix();
    let unpadded_bytes_per_row = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("offscreen-readback"),
        size: padded_bytes_per_row as u64 * height as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    let (tx, rx) = std::sync::mpsc::channel();
    encoder.map_buffer_on_submit(&buffer, wgpu::MapMode::Read, .., move |result| {
        tx.send(result).unwrap();
    });
    gpu.queue.submit([encoder.finish()]);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    rx.recv().unwrap().expect("map readback buffer");

    let view = buffer.get_mapped_range(..).expect("get mapped range");
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);

    for row in 0..height {
        let start = (row * padded_bytes_per_row) as usize;
        let end = start + unpadded_bytes_per_row as usize;
        let row_bytes = &view[start..end];

        match format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {
                pixels.extend_from_slice(row_bytes);
            }
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                for chunk in row_bytes.chunks_exact(4) {
                    pixels.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
                }
            }
            other => panic!("unsupported readback format: {other:?}"),
        }
    }

    drop(view);
    buffer.unmap();

    ImageBuffer::from_raw(width, height, pixels).expect("rgba image from readback")
}

/// Clear a texture to a solid wgpu color, submit, and flush.
pub fn clear_texture(gpu: &Wgpu, texture: &wgpu::Texture, color: wgpu::Color) {
    let view = texture.create_view(&Default::default());
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("offscreen-clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    gpu.queue.submit([encoder.finish()]);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
}

fn dump_failure(name: &str, image: &RgbaImage) {
    let dir = std::env::temp_dir().join("waywe-test-failures");
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("{name}.png"));
    if let Err(error) = image.save(&path) {
        eprintln!("failed to dump test image to {}: {error}", path.display());
    } else {
        eprintln!("dumped failing image to {}", path.display());
    }
}

/// Assert every pixel in `rect` is within `tol` of `expected` (RGBA).
pub fn assert_region_approx(
    image: &RgbaImage,
    rect: Rect,
    expected: [u8; 4],
    tol: u8,
    test_name: &str,
) {
    let expected = Rgba(expected);
    for y in rect.y..rect.y.saturating_add(rect.height).min(image.height()) {
        for x in rect.x..rect.x.saturating_add(rect.width).min(image.width()) {
            let pixel = *image.get_pixel(x, y);
            if !rgba_within(pixel, expected, tol) {
                dump_failure(test_name, image);
                panic!(
                    "{test_name}: pixel ({x},{y}) = {:?} not within {tol} of {:?}",
                    pixel.0, expected.0
                );
            }
        }
    }
}

/// Assert the average color of `rect` is within `tol` of `expected` (RGBA).
pub fn assert_avg_color(
    image: &RgbaImage,
    rect: Rect,
    expected: [u8; 4],
    tol: u8,
    test_name: &str,
) {
    let mut sum = [0u64; 4];
    let mut count = 0u64;

    for y in rect.y..rect.y.saturating_add(rect.height).min(image.height()) {
        for x in rect.x..rect.x.saturating_add(rect.width).min(image.width()) {
            let pixel = image.get_pixel(x, y).0;
            for i in 0..4 {
                sum[i] += u64::from(pixel[i]);
            }
            count += 1;
        }
    }

    assert!(count > 0, "{test_name}: empty region");
    let avg = [
        (sum[0] / count) as u8,
        (sum[1] / count) as u8,
        (sum[2] / count) as u8,
        (sum[3] / count) as u8,
    ];

    if !rgba_within(Rgba(avg), Rgba(expected), tol) {
        dump_failure(test_name, image);
        panic!(
            "{test_name}: avg {:?} not within {tol} of {:?}",
            avg, expected
        );
    }
}

fn rgba_within(actual: Rgba<u8>, expected: Rgba<u8>, tol: u8) -> bool {
    actual
        .0
        .iter()
        .zip(expected.0.iter())
        .all(|(a, e)| a.abs_diff(*e) <= tol)
}

/// Create a solid-color RGBA8 image buffer (for wallpaper inputs).
pub fn solid_rgba_image(width: u32, height: u32, rgba: [u8; 4]) -> RgbaImage {
    ImageBuffer::from_pixel(width, height, Rgba(rgba))
}
