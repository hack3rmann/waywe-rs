use crate::wallpaper::{Wallpaper, optimized::OptimizedWallpaper};
use static_assertions::assert_impl_all;
use std::{mem, num::NonZeroU64};
use waywe_runtime::{WallpaperConfig, gpu::Wgpu};
use waywe_spirv_derive::ShaderDescriptor;

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/waywe-daemon/src/shaders/copy-texture-to-buffer.glsl",
    stage = "compute"
)]
pub struct CopyTextureToBufferShader;

pub struct PreviewPipeline {
    pub bind_group: wgpu::BindGroup,
    pub copy_pipeline: wgpu::ComputePipeline,
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
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let stage_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wallpaper-preview-stage-buffer"),
            size: mem::size_of::<u32>() as u64
                * config.surface_size.x as u64
                * config.surface_size.y as u64,
            usage: wgpu::BufferUsages::MAP_READ
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let sampler = gpu.device.create_sampler(&Default::default());

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("copy-texture-to-buffer-bind-group-layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: NonZeroU64::new(stage_buffer.size()),
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("copy-texture-to-buffer-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &surface.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: stage_buffer.as_entire_binding(),
                },
            ],
        });

        gpu.require_shader::<CopyTextureToBufferShader>();

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("copy-texture-to-buffer-pipeline-layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

        let copy_pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("copy-texture-to-buffer-pipeline"),
                layout: Some(&pipeline_layout),
                module: &gpu.shader_cache.get::<CopyTextureToBufferShader>().unwrap(),
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
                cache: None,
            });

        Self {
            bind_group,
            copy_pipeline,
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

        {
            let mut pass = encoder.begin_compute_pass(&Default::default());

            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_pipeline(&self.copy_pipeline);

            pass.dispatch_workgroups(self.surface.width(), self.surface.height(), 1);
        }

        let buffer = self.stage_buffer.clone();

        encoder.map_buffer_on_submit(&self.stage_buffer, wgpu::MapMode::Read, .., move |result| {
            result.unwrap();
            on_success(buffer);
        });

        gpu.queue.submit([encoder.finish()]);
    }
}
