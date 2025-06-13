use super::{DynWallpaper, Wallpaper};
use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures, gpu::Wgpu},
};
use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2};
use std::{collections::HashMap, mem, time::Instant};
use wgpu::util::DeviceExt as _;

pub struct TransitionWallpaper {
    pipeline: TransitionPipeline,
}

impl TransitionWallpaper {
    pub fn new(runtime: &mut Runtime, from: DynWallpaper, to: DynWallpaper) -> Self {
        Self {
            pipeline: TransitionPipeline::new(
                &mut runtime.wgpu,
                runtime.wayland.client_state.monitor_size(),
                from,
                to,
            ),
        }
    }

    pub fn try_resolve(self: Box<Self>) -> DynWallpaper {
        let Some(start) = self.pipeline.animation_start else {
            return self;
        };

        if start.elapsed().as_secs_f32() >= 2.0 {
            self.pipeline.to
        } else {
            self
        }
    }
}

impl Wallpaper for TransitionWallpaper {
    fn required_features() -> RuntimeFeatures
    where
        Self: Sized,
    {
        RuntimeFeatures::VIDEO | RuntimeFeatures::GPU
    }

    fn frame(
        &mut self,
        runtime: &Runtime,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError> {
        self.pipeline.render(runtime, encoder, surface_view)
    }
}

const SCREEN_TRIANGLE: [Vec2; 3] = [
    Vec2::new(-1.0, -1.0),
    Vec2::new(3.0, -1.0),
    Vec2::new(-1.0, 3.0),
];

pub struct TransitionPipeline {
    from: DynWallpaper,
    to: DynWallpaper,
    vertex_buffer: wgpu::Buffer,
    from_texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    animation_start: Option<Instant>,
}

impl TransitionPipeline {
    pub fn new(gpu: &mut Wgpu, screen_size: UVec2, from: DynWallpaper, to: DynWallpaper) -> Self {
        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("screen-triangle"),
                contents: bytemuck::bytes_of(&SCREEN_TRIANGLE),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let from_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("from-texture-target"),
            size: wgpu::Extent3d {
                width: screen_size.x,
                height: screen_size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: gpu.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let from_texture_view = from_texture.create_view(&Default::default());

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("image-bind-group-layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&from_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        const VERTEX_SHADER_NAME: &str = "shaders/white-vertex.glsl";
        const FRAGMENT_SHADER_NAME: &str = "shaders/transition.glsl";

        gpu.shader_cache
            .entry(VERTEX_SHADER_NAME)
            .or_insert_with(|| {
                gpu.device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Glsl {
                            shader: include_str!("../shaders/white-vertex.glsl").into(),
                            stage: wgpu::naga::ShaderStage::Vertex,
                            defines: Default::default(),
                        },
                    })
            });

        gpu.shader_cache
            .entry(FRAGMENT_SHADER_NAME)
            .or_insert_with(|| {
                gpu.device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Glsl {
                            shader: include_str!("../shaders/transition.glsl").into(),
                            stage: wgpu::naga::ShaderStage::Fragment,
                            defines: Default::default(),
                        },
                    })
            });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("image-pipeline-layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: 0..mem::size_of::<PushConst>() as u32,
                }],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &gpu.shader_cache[VERTEX_SHADER_NAME],
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &HashMap::new(),
                        zero_initialize_workgroup_memory: false,
                    },
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: mem::size_of_val(&SCREEN_TRIANGLE[0]) as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &gpu.shader_cache[FRAGMENT_SHADER_NAME],
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &HashMap::new(),
                        zero_initialize_workgroup_memory: false,
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        Self {
            from,
            to,
            vertex_buffer,
            from_texture_view,
            bind_group,
            pipeline,
            animation_start: None,
        }
    }

    pub fn render(
        &mut self,
        runtime: &Runtime,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError> {
        let animantion_time = self
            .animation_start
            .get_or_insert_with(Instant::now)
            .elapsed();

        let first_info = self.from.frame(runtime, encoder, &self.from_texture_view)?;
        let second_info = self.to.frame(runtime, encoder, surface_view)?;

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::bytes_of(&PushConst {
                time: animantion_time.as_secs_f32(),
            }),
        );
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..SCREEN_TRIANGLE.len() as u32, 0..1);

        Ok(first_info.best_or_60_fps(second_info))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PushConst {
    time: f32,
}
