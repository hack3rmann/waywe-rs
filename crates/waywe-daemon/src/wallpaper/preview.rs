use crate::wallpaper::{Wallpaper, optimized::OptimizedWallpaper};
use std::mem;
use waywe_runtime::{WallpaperConfig, gpu::Wgpu};

pub fn preview_wallpaper(
    gpu: &Wgpu,
    wallpaper: &mut OptimizedWallpaper,
    config: WallpaperConfig,
) -> Vec<u8> {
    let surface = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("wallpaper-preview-surface-texture"),
        size: wgpu::Extent3d {
            width: config.surface_size.x,
            height: config.surface_size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.surface_format,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("wallpaper-preview-stage-buffer"),
        size: mem::size_of::<u32>() as u64
            * config.surface_size.x as u64
            * config.surface_size.y as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let surface_view = surface.create_view(&Default::default());
    let mut encoder = gpu.device.create_command_encoder(&Default::default());

    let _frame_info = wallpaper.frame(gpu, &surface_view, &mut encoder);

    encoder.copy_texture_to_buffer(
        surface.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 1,
                bytes_per_row: Some(mem::size_of::<u32>() as u32 * config.surface_size.x),
                rows_per_image: Some(config.surface_size.y),
            },
        },
        surface.size(),
    );

    encoder.map_buffer_on_submit(&buffer, wgpu::MapMode::Read, .., Result::unwrap);

    let index = gpu.queue.submit([encoder.finish()]);
    let _status = gpu
        .device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(index),
            timeout: None,
        })
        .unwrap();

    buffer.get_mapped_range(..).unwrap().to_vec()
}
