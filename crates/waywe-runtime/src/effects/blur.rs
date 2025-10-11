use crate::{
    effects::{AppliedEffect, Effect, config::EffectConfig},
    gpu::Wgpu,
    shaders::ShaderDescriptor,
    wayland::MonitorId,
};
use std::mem;

const LABEL: &str = "blur";

pub struct BlurConfig;

impl EffectConfig for BlurConfig {
    fn build_effect(&self, gpu: &Wgpu, monitor_id: MonitorId) -> Box<dyn Effect> {
        Box::new(Blur::new(gpu, monitor_id))
    }
}

pub struct DownsamplePipeline {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::ComputePipeline,
    pub downsampled: wgpu::TextureView,
}

impl DownsamplePipeline {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId) -> Self {
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

        let downsampled_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(LABEL),
            size: wgpu::Extent3d {
                width: size.width / 2,
                height: size.height / 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let downsampled = downsampled_texture.create_view(&Default::default());

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(LABEL),
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
                push_constant_ranges: &[],
            });

        gpu.require_shader::<DownsampleShader>();

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(LABEL),
                layout: Some(&pipeline_layout),
                module: &gpu.shader_cache.get::<DownsampleShader>().unwrap(),
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
                cache: None,
            });

        Self {
            pipeline,
            bind_group_layout,
            downsampled,
        }
    }
}

pub struct BlurPipeline {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::ComputePipeline,
    pub blurred: wgpu::TextureView,
}

impl BlurPipeline {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId) -> Self {
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

        let blurred_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(LABEL),
            size: wgpu::Extent3d {
                width: size.width / 2,
                height: size.height / 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let blurred = blurred_texture.create_view(&Default::default());

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(LABEL),
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
                    range: 0..mem::size_of::<u32>() as u32,
                }],
            });

        gpu.require_shader::<BlurShader>();

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(LABEL),
                layout: Some(&pipeline_layout),
                module: &gpu.shader_cache.get::<BlurShader>().unwrap(),
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
                cache: None,
            });

        Self {
            bind_group_layout,
            pipeline,
            blurred,
        }
    }
}

pub struct UpsamplePipeline {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::ComputePipeline,
    pub sampler: wgpu::Sampler,
    pub result: wgpu::TextureView,
}

impl UpsamplePipeline {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId) -> Self {
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

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(LABEL),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let result_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(LABEL),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let result = result_texture.create_view(&Default::default());

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(LABEL),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
                push_constant_ranges: &[],
            });

        gpu.require_shader::<UpsampleShader>();

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(LABEL),
                layout: Some(&pipeline_layout),
                module: &gpu.shader_cache.get::<UpsampleShader>().unwrap(),
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
                cache: None,
            });

        Self {
            sampler,
            bind_group_layout,
            pipeline,
            result,
        }
    }
}

pub struct Blur {
    pub downsample: DownsamplePipeline,
    pub blur: BlurPipeline,
    pub upsample: UpsamplePipeline,
}

impl Blur {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId) -> Self {
        Self {
            downsample: DownsamplePipeline::new(gpu, monitor_id),
            blur: BlurPipeline::new(gpu, monitor_id),
            upsample: UpsamplePipeline::new(gpu, monitor_id),
        }
    }
}

impl Effect for Blur {
    fn apply(
        &mut self,
        gpu: &Wgpu,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
    ) -> AppliedEffect {
        const WORKGROUP_SIZE: u32 = 8;
        let width = self.downsample.downsampled.texture().size().width / WORKGROUP_SIZE;
        let height = self.downsample.downsampled.texture().size().height / WORKGROUP_SIZE;

        {
            let downsample_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(LABEL),
                layout: &self.downsample.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(input),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&self.downsample.downsampled),
                    },
                ],
            });

            let mut pass = encoder.begin_compute_pass(&Default::default());

            pass.set_bind_group(0, &downsample_bind_group, &[]);
            pass.set_pipeline(&self.downsample.pipeline);

            pass.dispatch_workgroups(width, height, 1);
        }

        for _ in 0..10 {
            // Blur for X axis
            {
                let blur_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(LABEL),
                    layout: &self.downsample.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.downsample.downsampled,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&self.blur.blurred),
                        },
                    ],
                });

                let mut pass = encoder.begin_compute_pass(&Default::default());

                pass.set_bind_group(0, &blur_bind_group, &[]);
                pass.set_pipeline(&self.blur.pipeline);
                pass.set_push_constants(0, bytemuck::bytes_of(&0_u32));

                pass.dispatch_workgroups(width, height, 1);
            }

            // Blur for Y axis
            {
                let blur_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(LABEL),
                    layout: &self.downsample.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.blur.blurred),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                &self.downsample.downsampled,
                            ),
                        },
                    ],
                });

                let mut pass = encoder.begin_compute_pass(&Default::default());

                pass.set_bind_group(0, &blur_bind_group, &[]);
                pass.set_pipeline(&self.blur.pipeline);
                pass.set_push_constants(0, bytemuck::bytes_of(&1_u32));

                pass.dispatch_workgroups(width, height, 1);
            }
        }

        {
            let upsample_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(LABEL),
                layout: &self.upsample.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.downsample.downsampled),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.upsample.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&self.upsample.result),
                    },
                ],
            });

            let mut pass = encoder.begin_compute_pass(&Default::default());

            pass.set_bind_group(0, &upsample_bind_group, &[]);
            pass.set_pipeline(&self.upsample.pipeline);

            let width = input.texture().size().width / WORKGROUP_SIZE;
            let height = input.texture().size().height / WORKGROUP_SIZE;
            pass.dispatch_workgroups(width, height, 1);
        }

        AppliedEffect::WithOutput(self.upsample.result.clone())
    }
}

pub struct DownsampleShader;

impl ShaderDescriptor for DownsampleShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::include_wgsl!("../shaders/downsample.wgsl")
    }
}

pub struct BlurShader;

impl ShaderDescriptor for BlurShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::include_wgsl!("../shaders/blur-axis.wgsl")
    }
}

pub struct UpsampleShader;

impl ShaderDescriptor for UpsampleShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::include_wgsl!("../shaders/upsample.wgsl")
    }
}
