pub mod video_pipeline;

use glam::UVec2;
use raw_window_handle::{HasDisplayHandle as _, RawWindowHandle, WaylandWindowHandle};
use std::{
    error::Error,
    ffi::CStr,
    pin::pin,
    sync::{
        Once,
        atomic::{AtomicU32, Ordering::Relaxed},
    },
    thread,
    time::{Duration, Instant},
};
use video::{
    BackendError, Codec, CodecContext, FormatContext, Frame, FrameDuration, MediaType, RatioI32,
    VideoPixelFormat,
};
use video_pipeline::VideoPipeline;
use wayland_client::{
    WlSmallVecMessageBuffer,
    interface::{
        WlCompositorCreateSurfaceRequest, WlSurfaceCommitRequest, WlSurfaceSetBufferScaleRequest,
        ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellLayer,
        ZwlrLayerSurfaceAckConfigureRequest, ZwlrLayerSurfaceAnchor,
        ZwlrLayerSurfaceConfigureEvent, ZwlrLayerSurfaceKeyboardInteractivity,
        ZwlrLayerSurfaceSetAnchorRequest, ZwlrLayerSurfaceSetExclusiveZoneRequest,
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest, ZwlrLayerSurfaceSetMarginRequest,
    },
    object::{HasObjectType, WlObjectType},
    sys::{
        display::WlDisplay,
        object::{FromProxy, WlObject, WlObjectHandle, dispatch::Dispatch},
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{WlMessage, WlStackMessageBuffer},
    },
};

#[derive(Default, Debug)]
struct ClientState {
    pub width: AtomicU32,
    pub height: AtomicU32,
}

struct Compositor;

impl HasObjectType for Compositor {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
}

impl Dispatch for Compositor {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for Compositor {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

struct LayerShell;

impl HasObjectType for LayerShell {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerShell;
}

impl Dispatch for LayerShell {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for LayerShell {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

struct Surface;

impl HasObjectType for Surface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
}

impl Dispatch for Surface {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for Surface {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

struct LayerSurface {
    handle: WlObjectHandle<Self>,
}

impl HasObjectType for LayerSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
}

impl Dispatch for LayerSurface {
    type State = ClientState;

    fn dispatch(
        &mut self,
        state: &Self::State,
        storage: &mut WlObjectStorage<'_, Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(ZwlrLayerSurfaceConfigureEvent {
            serial,
            width,
            height,
        }) = message.as_event()
        else {
            return;
        };

        state.width.store(width, Relaxed);
        state.height.store(height, Relaxed);

        let mut buf = WlSmallVecMessageBuffer::<3>::new();

        self.handle.request(
            &mut buf,
            storage,
            ZwlrLayerSurfaceAckConfigureRequest { serial },
        );
    }
}

impl FromProxy for LayerSurface {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

pub const WLR_NAMESPACE: &CStr = c"waywe-runtime";

trait SurfaceExtension {
    fn raw_window_handle(&self) -> RawWindowHandle;
}

impl SurfaceExtension for WlObject<Surface> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Wayland(WaylandWindowHandle::new(self.proxy().as_raw()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    video::init();

    let mut client_state = pin!(ClientState::default());

    let display = WlDisplay::connect(client_state.as_mut()).unwrap();

    let mut buf = WlStackMessageBuffer::new();
    let mut queue = pin!(display.take_main_queue().unwrap());

    let registry = display.create_registry(&mut buf, queue.as_mut().storage_mut());

    display.roundtrip(queue.as_mut(), client_state.as_ref());

    let compositor = registry
        .bind::<Compositor>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let layer_shell = registry
        .bind::<LayerShell>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let surface: WlObjectHandle<Surface> = compositor.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlCompositorCreateSurfaceRequest,
    );

    let layer_surface: WlObjectHandle<LayerSurface> = layer_shell.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        ZwlrLayerShellGetLayerSurfaceRequest {
            surface: surface.id(),
            output: None,
            layer: ZwlrLayerShellLayer::Background,
            namespace: WLR_NAMESPACE,
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetAnchorRequest {
            anchor: ZwlrLayerSurfaceAnchor::all(),
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetExclusiveZoneRequest { zone: -1 },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetMarginRequest {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest {
            keyboard_interactivity: ZwlrLayerSurfaceKeyboardInteractivity::None,
        },
    );

    surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        WlSurfaceSetBufferScaleRequest { scale: 1 },
    );

    surface.request(&mut buf, &queue.as_ref().storage(), WlSurfaceCommitRequest);

    display.roundtrip(queue.as_mut(), client_state.as_ref());

    let screen_size = UVec2::new(
        client_state.width.load(Relaxed),
        client_state.height.load(Relaxed),
    );

    assert_ne!(screen_size.x, 0);
    assert_ne!(screen_size.y, 0);

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: if cfg!(debug_assertions) {
            wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION
        } else {
            wgpu::InstanceFlags::empty()
        },
        ..Default::default()
    });

    let raw_display_handle = display.display_handle().unwrap().as_raw();
    let raw_window_handle = queue.as_ref().storage().object(surface).raw_window_handle();

    let wgpu_surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })
            .unwrap()
    };

    let Some(adapter) = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: Some(&wgpu_surface),
        })
        .await
    else {
        panic!("failed to request adapter");
    };

    let (device, gpu_queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::PUSH_CONSTANTS,
                label: None,
                required_limits: adapter.limits(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )
        .await?;

    wgpu_surface.configure(
        &device,
        &wgpu_surface
            .get_default_config(&adapter, screen_size.x, screen_size.y)
            .unwrap(),
    );

    let Some(surface_format) = wgpu_surface
        .get_capabilities(&adapter)
        .formats
        .first()
        .copied()
    else {
        panic!("no surface format supported");
    };

    const FILE_NAMES: &[&CStr] = &[
        c"/home/hack3rmann/Downloads/Telegram Desktop/845514446/video.mp4",
        c"/home/hack3rmann/Videos/ObsRecordings/2025-05-16 15-51-50.mp4",
        c"/home/hack3rmann/Pictures/Wallpapers/night-sky-purple-moon-clouds-3840x2160.mp4",
        c"/home/hack3rmann/Downloads/sample-1.avi",
        c"http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
    ];

    let mut format_context = FormatContext::from_input(FILE_NAMES[2])?;

    let best_stream = format_context.find_best_stream(MediaType::Video)?;
    let time_base = best_stream.time_base();
    let best_stream_index = best_stream.index();
    let codec_parameters = best_stream.codec_parameters();
    let frame_rate = codec_parameters.frame_rate().unwrap();
    let video_size = codec_parameters.video_size().unwrap();
    assert!(
        matches!(
            codec_parameters.format(),
            Some(video::AudioVideoFormat::Video(VideoPixelFormat::Yuv420p))
        ),
        "invalid format"
    );
    let mut codec_context = CodecContext::from_parameters(codec_parameters)?;

    const FRAME_DURATION_60_FPS: Duration = RatioI32::new(1, 60).unwrap().to_duration_seconds();

    let frame_time_fallback = move || match frame_rate.inv() {
        Some(duration) => duration.to_duration_seconds(),
        None => {
            static ONCE: Once = Once::new();
            ONCE.call_once(|| {
                tracing::warn!("can not determine vodeo frame rate, falling back to 60 fps")
            });
            FRAME_DURATION_60_FPS
        }
    };

    let video_pipeline = VideoPipeline::new(&device, surface_format, video_size, screen_size);

    let Some(decoder) = Codec::find_for_id(codec_context.codec_id()) else {
        panic!("failed to find decoder");
    };

    codec_context.open(decoder)?;

    let frame_loop_start = Instant::now();
    let mut last_instant = frame_loop_start;

    let mut frame = Frame::new();

    const DO_LOOP_VIDEO: bool = true;

    loop {
        let packet = match format_context.read_packet() {
            Ok(packet) => packet,
            Err(BackendError::EOF) => {
                if !DO_LOOP_VIDEO {
                    break;
                }

                format_context.repeat_stream(best_stream_index)?;
                continue;
            }
            error @ Err(..) => error?,
        };

        if packet.stream_index() != best_stream_index {
            continue;
        }

        codec_context.send_packet(&packet)?;

        while codec_context.receive_frame(&mut frame).is_ok() {
            let target_frame_time = frame
                .duration_in(time_base)
                .map(FrameDuration::to_duration)
                .unwrap_or_else(frame_time_fallback);

            let frame_start = Instant::now();
            let time_from_start = last_instant.duration_since(frame_loop_start);
            last_instant = frame_start;
            let _time_seconds = time_from_start.as_secs_f32();

            let data_planes = unsafe { [frame.data(0), frame.data(1), frame.data(2)] };

            video_pipeline.write_video_frame(&gpu_queue, data_planes);
            gpu_queue.submit([]);

            let surface_texture = wgpu_surface.get_current_texture().unwrap();
            let surface_view = surface_texture.texture.create_view(&Default::default());

            let mut encoder = device.create_command_encoder(&Default::default());

            video_pipeline.render(&mut encoder, &surface_view);

            _ = gpu_queue.submit([encoder.finish()]);

            surface_texture.present();

            display.roundtrip(queue.as_mut(), client_state.as_ref());

            let render_time = Instant::now().duration_since(frame_start);
            let sleep_time = target_frame_time.saturating_sub(render_time);

            if !sleep_time.is_zero() {
                thread::sleep(sleep_time);
            } else {
                tracing::warn!(?render_time, "frame took too long to prepare");
            }
        }
    }

    Ok(())
}
