use glam::UVec2;
use std::sync::Arc;
use waywe_runtime::testing::{
    DEFAULT_FORMAT, DEFAULT_SIZE, OffscreenSurface, Rect, assert_avg_color,
};
use waywe_scene::{
    Monitor,
    gpu::Gpu,
    render::RenderGpu,
    wallpaper::{PreparedWallpaper, Wallpaper, WallpaperBuilder},
};

pub fn monitor() -> Monitor {
    Monitor {
        size: DEFAULT_SIZE,
        surface_format: DEFAULT_FORMAT,
    }
}

pub fn render_scene(builder: impl WallpaperBuilder) -> image::RgbaImage {
    let gpu = Arc::new(pollster::block_on(Gpu::new()));
    let mut wallpaper = Wallpaper::new(gpu, monitor(), ".".into());
    builder.build(&mut wallpaper);
    let mut prepared = PreparedWallpaper::prepare(wallpaper);

    let render_gpu = prepared
        .wallpaper
        .render
        .world
        .resource::<RenderGpu>()
        .0
        .clone();

    let surface = OffscreenSurface::new(&render_gpu.device, DEFAULT_SIZE, DEFAULT_FORMAT);
    let texture = surface.texture_clone();
    clear_texture(
        &render_gpu.device,
        &render_gpu.queue,
        &texture,
        wgpu::Color::BLACK,
    );

    let view = texture.create_view(&Default::default());
    let mut encoder = render_gpu
        .device
        .create_command_encoder(&Default::default());
    prepared.frame(view, &mut encoder);
    render_gpu.queue.submit([encoder.finish()]);
    render_gpu
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();

    readback(&render_gpu.device, &render_gpu.queue, &texture)
}

fn clear_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    color: wgpu::Color,
) {
    let view = texture.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scene-test-clear"),
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
    queue.submit([encoder.finish()]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
}

fn readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> image::RgbaImage {
    let width = texture.width();
    let height = texture.height();
    let format = texture.format().remove_srgb_suffix();
    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scene-readback"),
        size: padded as u64 * height as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&Default::default());
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
                bytes_per_row: Some(padded),
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
    encoder.map_buffer_on_submit(&buffer, wgpu::MapMode::Read, .., move |r| {
        tx.send(r).unwrap();
    });
    queue.submit([encoder.finish()]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();

    let view = buffer.get_mapped_range(..).unwrap();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        let start = (row * padded) as usize;
        let end = start + unpadded as usize;
        let row_bytes = &view[start..end];
        match format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {
                pixels.extend_from_slice(row_bytes);
            }
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                for c in row_bytes.chunks_exact(4) {
                    pixels.extend_from_slice(&[c[2], c[1], c[0], c[3]]);
                }
            }
            other => panic!("unsupported format {other:?}"),
        }
    }
    drop(view);
    buffer.unmap();
    image::RgbaImage::from_raw(width, height, pixels).unwrap()
}

pub fn assert_full_avg(image: &image::RgbaImage, rgba: [u8; 4], tol: u8, name: &str) {
    assert_avg_color(
        image,
        Rect::full(UVec2::new(image.width(), image.height())),
        rgba,
        tol,
        name,
    );
}
