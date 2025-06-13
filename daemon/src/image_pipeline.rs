use crate::runtime::gpu::Wgpu;
use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2};
use image::{ImageBuffer, Rgba};
use std::{collections::HashMap, mem};
use wgpu::util::DeviceExt as _;

const SCREEN_TRIANGLE: [Vec2; 3] = [
    Vec2::new(-1.0, -1.0),
    Vec2::new(3.0, -1.0),
    Vec2::new(-1.0, 3.0),
];

pub type Color = u32;

pub const COLOR_WHITE: Color = u32::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct PushConst {
    pub resolution: Vec2,
    pub transparency_color: u32,
}

pub struct ImagePipeline {
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    screen_size: UVec2,
    transparency_color: Color,
}

impl ImagePipeline {
    pub fn new(
        gpu: &mut Wgpu,
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        transparency_color: Color,
        monitor_size: UVec2,
    ) -> Self {
        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("screen-triangle"),
                contents: bytemuck::bytes_of(&SCREEN_TRIANGLE),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let texture = gpu.device.create_texture_with_data(
            &gpu.queue,
            &wgpu::TextureDescriptor {
                label: Some("image-texture"),
                size: wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            image,
        );

        let texture_view = texture.create_view(&Default::default());

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
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        const VERTEX_SHADER_NAME: &str = "shaders/white-vertex.glsl";
        const FRAGMENT_SHADER_NAME: &str = "shaders/image.glsl";

        gpu.shader_cache
            .entry(VERTEX_SHADER_NAME)
            .or_insert_with(|| {
                gpu.device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Glsl {
                            shader: include_str!("shaders/white-vertex.glsl").into(),
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
                            shader: include_str!("shaders/image.glsl").into(),
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
            vertex_buffer,
            bind_group,
            pipeline,
            screen_size: monitor_size,
            transparency_color,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, surface_view: &wgpu::TextureView) {
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
                resolution: Vec2::new(self.screen_size.x as f32, self.screen_size.y as f32),
                transparency_color: self.transparency_color,
            }),
        );
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..SCREEN_TRIANGLE.len() as u32, 0..1);
    }
}
