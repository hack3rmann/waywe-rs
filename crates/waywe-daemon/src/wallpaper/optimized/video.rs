use crate::wallpaper::{Wallpaper, WallpaperConfig, optimized::image::FullscreenVertex};
use for_sure::prelude::*;
use glam::Vec2;
use std::{mem, path::PathBuf, time::Duration};
use video::{BackendError, FrameDuration};
use waywe_runtime::{frame::FrameInfo, gpu::Wgpu};
use waywe_scene::video::{RenderVideo, Video};
use waywe_spirv_derive::ShaderDescriptor;
use wgpu::util::DeviceExt;

pub const LABEL: &str = "default-video";

pub struct VideoWallpaper {
    pub video: Video,
    pub rendered_video: Almost<RenderVideo>,
    pub pipeline: VideoPipeline,
    pub config: WallpaperConfig,
}

impl VideoWallpaper {
    pub fn new(
        path: impl Into<PathBuf>,
        gpu: &Wgpu,
        config: WallpaperConfig,
    ) -> Result<Self, BackendError> {
        Ok(Self {
            video: Video::new(path)?,
            rendered_video: Nil,
            pipeline: VideoPipeline::new(gpu, config),
            config,
        })
    }

    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL),
            layout: &self.pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.rendered_video.texture_y_plane,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.rendered_video.texture_uv_plane,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.pipeline.sampler),
                },
            ],
        })
    }
}

impl Wallpaper for VideoWallpaper {
    fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig) {
        if self.config.surface_format != config.surface_format {
            self.pipeline.pipeline = VideoPipeline::create_pipeline(
                gpu,
                &self.pipeline.pipeline_layout,
                config.surface_format,
            );
        }

        self.config = config;
    }

    fn frame(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo {
        if self.video.n_frames_since_update == 0 || Almost::is_nil(&self.rendered_video) {
            self.rendered_video = Value(RenderVideo::export_from(
                &self.video,
                &gpu.adapter,
                &gpu.device,
            ));
        }

        let bind_group = self.create_bind_group(&gpu.device);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let size = Vec2::new(
            self.config.surface_size.x as f32,
            self.config.surface_size.y as f32,
        );

        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_vertex_buffer(0, self.pipeline.vertex_buffer.slice(..));
        pass.set_immediates(0, bytemuck::bytes_of(&size));
        pass.set_bind_group(0, &bind_group, &[]);

        pass.draw(0..SCREEN_TRIANGLE.len() as u32, 0..1);

        let duration = self
            .video
            .frame
            .duration_in(self.video.time_base)
            .map(FrameDuration::to_duration)
            .unwrap_or(self.video.frame_time_fallback);

        FrameInfo {
            target_frame_time: Some(duration),
        }
    }

    fn advance_time(&mut self, delta: Duration) {
        self.video.advance_by(delta);
    }
}

const SCREEN_TRIANGLE: [Vec2; 3] = [
    Vec2::new(-1.0, -1.0),
    Vec2::new(3.0, -1.0),
    Vec2::new(-1.0, 3.0),
];

pub struct VideoPipeline {
    pub config: WallpaperConfig,
    pub vertex_buffer: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}

impl VideoPipeline {
    pub fn new(gpu: &Wgpu, config: WallpaperConfig) -> Self {
        gpu.require_shader::<FullscreenVertex>();
        gpu.require_shader::<VideoFragment>();

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("screen-triangle"),
                contents: bytemuck::bytes_of(&SCREEN_TRIANGLE),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("waywe-bind-group-layout"),
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
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: mem::size_of::<Vec2>() as u32,
            });

        let pipeline = Self::create_pipeline(gpu, &pipeline_layout, config.surface_format);

        Self {
            sampler,
            config,
            vertex_buffer,
            pipeline,
            pipeline_layout,
            bind_group_layout,
        }
    }

    fn create_pipeline(
        gpu: &Wgpu,
        layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        gpu.require_shader::<FullscreenVertex>();
        gpu.require_shader::<VideoFragment>();

        gpu.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &gpu.shader_cache.get::<FullscreenVertex>().unwrap(),
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
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
                    module: &gpu.shader_cache.get::<VideoFragment>().unwrap(),
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
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
                multiview_mask: None,
                cache: None,
            })
    }
}

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/waywe-daemon/src/shaders/video.glsl",
    stage = "fragment",
    label = "default-video"
)]
pub struct VideoFragment;
