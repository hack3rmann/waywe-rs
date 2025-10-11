use crate::{
    effects::{AppliedEffect, EFFECTS_TEXTURE_DESC, Effect, config::EffectConfig},
    gpu::Wgpu,
    shaders::ShaderDescriptor,
    wayland::MonitorId,
};
use bytemuck::{Pod, Zeroable};
use std::{mem, num::NonZeroU64, sync::Arc};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const LABEL: &str = "convolve";

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ConvolveConfig {
    pub kernel: Arc<[f32]>,
}

impl EffectConfig for ConvolveConfig {
    fn build_effect(&self, gpu: &Wgpu, monitor_id: MonitorId) -> Box<dyn Effect> {
        Box::new(Convolve::new(gpu, monitor_id, &self.kernel))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct PushConst {
    pub kernel_size: u32,
}

pub struct Convolve {
    pub output: wgpu::TextureView,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::ComputePipeline,
    pub kernel: wgpu::Buffer,
    pub kernel_size: u32,
}

impl Convolve {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId, kernel_data: &[f32]) -> Self {
        let kernel_size = kernel_data.len().isqrt() as u32;
        assert_eq!(
            kernel_size as u64 * kernel_size as u64,
            kernel_data.len() as u64
        );

        let (size, format) = {
            let surfaces = gpu.surfaces.read().unwrap();
            let surface = &surfaces[&monitor_id];

            (
                wgpu::Extent3d {
                    width: surface.config.width,
                    height: surface.config.height,
                    depth_or_array_layers: 1,
                },
                surface.format.remove_srgb_suffix(),
            )
        };

        let output_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(LABEL),
            size,
            format,
            ..EFFECTS_TEXTURE_DESC
        });
        let output = output_texture.create_view(&Default::default());

        let kernel = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(LABEL),
            contents: bytemuck::cast_slice(kernel_data),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(LABEL),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: NonZeroU64::new(
                                    mem::size_of_val(kernel_data) as u64
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(LABEL),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: 0..mem::size_of::<PushConst>() as u32,
                }],
            });

        gpu.require_shader::<ConvolveShader>();

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(LABEL),
                layout: Some(&pipeline_layout),
                module: &gpu.shader_cache.get::<ConvolveShader>().unwrap(),
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
                cache: None,
            });

        Self {
            output,
            pipeline,
            kernel,
            bind_group_layout,
            kernel_size,
        }
    }
}

impl Effect for Convolve {
    fn apply(
        &mut self,
        gpu: &Wgpu,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
    ) -> AppliedEffect {
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.kernel.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(input),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.output),
                },
            ],
        });

        {
            let mut pass = encoder.begin_compute_pass(&Default::default());

            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_pipeline(&self.pipeline);
            pass.set_push_constants(
                0,
                bytemuck::bytes_of(&PushConst {
                    kernel_size: self.kernel_size,
                }),
            );

            const WORKGROUP_SIZE: u32 = 8;
            let width = input.texture().size().width / WORKGROUP_SIZE;
            let height = input.texture().size().width / WORKGROUP_SIZE;

            pass.dispatch_workgroups(width, height, 1);
        }

        AppliedEffect::WithOutput(self.output.clone())
    }
}

pub struct ConvolveShader;

impl ShaderDescriptor for ConvolveShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::include_wgsl!("../shaders/convolve.wgsl")
    }
}
