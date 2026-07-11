use crate::wallpaper::{Wallpaper, optimized::OptimizedWallpaper};
use static_assertions::assert_impl_all;
use std::mem;
use waywe_runtime::{WallpaperConfig, gpu::Wgpu};

pub struct PreviewPipeline {
    pub surface: wgpu::Texture,
    pub stage_buffer: wgpu::Buffer,
    pub config: WallpaperConfig,
}
assert_impl_all!(PreviewPipeline: Send, Sync);

impl PreviewPipeline {
    pub fn new(gpu: &Wgpu, config: WallpaperConfig) -> Self {
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

        let stage_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wallpaper-preview-stage-buffer"),
            size: mem::size_of::<u32>() as u64
                * config.surface_size.x as u64
                * config.surface_size.y as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface,
            stage_buffer,
            config,
        }
    }

    pub fn render_async(
        &self,
        gpu: &Wgpu,
        wallpaper: &mut OptimizedWallpaper,
        on_success: impl FnOnce(wgpu::Buffer) + Send + 'static,
    ) {
        let surface_view = self.surface.create_view(&Default::default());
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        let _frame_info = wallpaper.frame(gpu, &surface_view, &mut encoder);

        encoder.copy_texture_to_buffer(
            self.surface.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &self.stage_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(mem::size_of::<u32>() as u32 * self.config.surface_size.x),
                    rows_per_image: Some(self.config.surface_size.y),
                },
            },
            self.surface.size(),
        );

        let buffer = self.stage_buffer.clone();

        encoder.map_buffer_on_submit(&self.stage_buffer, wgpu::MapMode::Read, .., move |result| {
            result.unwrap();
            on_success(buffer);
        });

        gpu.queue.submit([encoder.finish()]);
    }
}
