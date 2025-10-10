use crate::effects::{AppliedEffect, EFFECTS_TEXTURE_USAGES, Effect};
use bytemuck::{Pod, Zeroable};
use std::{mem, num::NonZeroU64};
use waywe_runtime::{gpu::Wgpu, shaders::ShaderDescriptor, wayland::MonitorId};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const LABEL: &str = "convolve";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct PushConst {
    pub kernel_size: u32,
}

pub struct Convolve {
    pub output: wgpu::TextureView,
    pub pipeline: wgpu::ComputePipeline,
    pub kernel: wgpu::Buffer,
}

impl Convolve {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId, kernel_data: &[f32]) -> Self {
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

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some(LABEL),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: EFFECTS_TEXTURE_USAGES,
            view_formats: &[],
        };

        let output_texture = gpu.device.create_texture(&texture_descriptor);
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
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::ReadOnly,
                                format,
                                view_dimension: wgpu::TextureViewDimension::D2,
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
        }
    }
}

impl Effect for Convolve {
    fn apply(
        &mut self,
        _gpu: &Wgpu,
        _encoder: &mut wgpu::CommandEncoder,
        _surface: &wgpu::TextureView,
    ) -> AppliedEffect {
        AppliedEffect::WithOutput(self.output.clone())
    }
}

pub struct ConvolveShader;

impl ShaderDescriptor for ConvolveShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::include_wgsl!("../shaders/convolve.wgsl")
    }
}
