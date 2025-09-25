use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{gpu::Wgpu, wayland::MonitorId},
    wallpaper::scene::wallpaper::PreparedWallpaper,
};
use bytemuck::{Pod, Zeroable};
use for_sure::prelude::*;
use glam::{UVec2, Vec2};
use smallvec::SmallVec;
use std::{
    mem,
    time::{Duration, Instant},
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const SCREEN_TRIANGLE: [Vec2; 3] = [
    Vec2::new(-1.0, -1.0),
    Vec2::new(3.0, -1.0),
    Vec2::new(-1.0, 3.0),
];

pub struct WallpaperTransitionState {
    pub state: wgpu::Texture,
    pub from: wgpu::TextureView,
    pub to: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

impl WallpaperTransitionState {
    pub fn new(gpu: &Wgpu, pipeline: &WallpaperTransitionPipeline) -> Self {
        let surfaces = gpu.surfaces.read().unwrap();
        let surface_config = &surfaces[&pipeline.monitor_id].config;

        let state = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("transition"),
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let from = state.create_view(&wgpu::TextureViewDescriptor {
            label: Some("transition-from"),
            base_array_layer: 0,
            array_layer_count: Some(1),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        let to = state.create_view(&wgpu::TextureViewDescriptor {
            label: Some("transition-from"),
            base_array_layer: 1,
            array_layer_count: Some(1),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transition-binds"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&from),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&to),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                },
            ],
        });

        Self {
            state,
            from,
            to,
            bind_group,
        }
    }

    pub fn swap_textures(&mut self) {
        mem::swap(&mut self.from, &mut self.to);
    }
}

pub struct WallpaperTransitionPipeline {
    pub monitor_id: MonitorId,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub vertices: wgpu::Buffer,
}

impl WallpaperTransitionPipeline {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId) -> Self {
        let vertices = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("fullscreen-triangle"),
            contents: bytemuck::cast_slice(&SCREEN_TRIANGLE),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("waywe-transition-binds"),
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

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("waywe-transition"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: 0..mem::size_of::<AnimationState>() as u32,
                }],
            });

        let surface_format = {
            let surfaces = gpu.surfaces.read().unwrap();
            surfaces[&monitor_id].format
        };

        const VERTEX_SHADER_NAME: &str = "shaders/white-vertex.glsl";
        const FRAGMENT_SHADER_NAME: &str = "shaders/transition.glsl";

        gpu.use_shader(
            VERTEX_SHADER_NAME,
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Glsl {
                    shader: include_str!("../shaders/fullscreen-vertex.glsl").into(),
                    stage: wgpu::naga::ShaderStage::Vertex,
                    defines: Default::default(),
                },
            },
        );

        gpu.use_shader(
            FRAGMENT_SHADER_NAME,
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Glsl {
                    shader: include_str!("../shaders/transition.glsl").into(),
                    stage: wgpu::naga::ShaderStage::Fragment,
                    defines: &[],
                },
            },
        );

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &gpu.shader_cache.get(VERTEX_SHADER_NAME).unwrap(),
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
                    module: &gpu.shader_cache.get(FRAGMENT_SHADER_NAME).unwrap(),
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
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

        let sampler = gpu.device.create_sampler(&Default::default());

        Self {
            monitor_id,
            pipeline,
            bind_group_layout,
            sampler,
            vertices,
        }
    }

    pub fn render(
        &self,
        state: &WallpaperTransitionState,
        surface_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        animation_state: &AnimationState,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("transition-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &state.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::bytes_of(animation_state),
        );

        pass.draw(0..SCREEN_TRIANGLE.len() as u32, 0..1);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default, Pod, Zeroable)]
pub struct AnimationState {
    pub centre: Vec2,
    pub radius: f32,
    pub direction: f32,
}

pub struct OngoingTransition {
    /// Amount of work done in 0..=1 (normalized time)
    pub done_fraction: f32,
    pub scale: f32,
    pub start_time: Instant,
    pub animation_duration: Duration,
}

impl OngoingTransition {
    pub fn new(scale: f32) -> Self {
        Self {
            done_fraction: 0.0,
            scale,
            start_time: Instant::now(),
            // TODO(hack3rmann): use config
            animation_duration: Duration::new(1, 0),
        }
    }

    pub fn update(&mut self) {
        let total = self.start_time.elapsed().as_secs_f32() / self.animation_duration.as_secs_f32();
        self.done_fraction = total.min(1.0);
    }

    pub fn is_finished(&self) -> bool {
        self.done_fraction >= 1.0
    }

    pub fn amount(&self) -> f32 {
        self.done_fraction * self.scale
    }
}

pub struct RunningWallpapers {
    pub monitor_id: MonitorId,
    pub aspect_ratio: f32,
    pub executing: Vec<PreparedWallpaper>,
    pub ongoing_transitions: SmallVec<[OngoingTransition; 8]>,
    pub transition_pipeline: Almost<WallpaperTransitionPipeline>,
    pub textures: Almost<WallpaperTransitionState>,
}

impl RunningWallpapers {
    pub const fn new(monitor_id: MonitorId, monitor_size: UVec2) -> Self {
        Self {
            monitor_id,
            aspect_ratio: monitor_size.y as f32 / monitor_size.x as f32,
            executing: vec![],
            ongoing_transitions: SmallVec::new_const(),
            transition_pipeline: Nil,
            textures: Nil,
        }
    }

    pub fn enqueue_wallpaper(&mut self, wallpaper: PreparedWallpaper) {
        self.executing.push(wallpaper);

        let scale = Vec2::new(1.0 / self.aspect_ratio, 1.0).length();

        if self.executing.len() >= 2 {
            self.ongoing_transitions.push(OngoingTransition::new(scale));
        }
    }

    pub fn remove_finished(&mut self) {
        while !self.ongoing_transitions.is_empty() {
            if !self.ongoing_transitions[0].is_finished() {
                break;
            }

            _ = self.ongoing_transitions.remove(0);
            _ = self.executing.remove(0);
        }
    }

    pub fn is_transitioning(&self) -> bool {
        self.executing.len() >= 2
    }

    pub fn init_transitions(&mut self, gpu: &Wgpu) {
        if self.is_transitioning() && Almost::is_nil(&self.transition_pipeline) {
            self.transition_pipeline =
                Value(WallpaperTransitionPipeline::new(gpu, self.monitor_id));
            self.textures = Value(WallpaperTransitionState::new(
                gpu,
                &self.transition_pipeline,
            ));
        }
    }

    pub fn render(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::Texture,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<FrameInfo, FrameError> {
        self.init_transitions(gpu);
        self.remove_finished();

        let surface_view = surface.create_view(&Default::default());

        match self.executing.as_mut_slice() {
            [] => return Err(FrameError::NoWorkToDo),
            [wallpaper] => return wallpaper.frame(surface_view, encoder),
            _ => {}
        }

        let ([first], tail) = self.executing.split_at_mut(1) else {
            unreachable!()
        };

        let mut result = first.frame(self.textures.from.clone(), encoder);

        for (wallpaper, transition) in tail.iter_mut().zip(&mut self.ongoing_transitions) {
            transition.update();
            result = wallpaper.frame(self.textures.to.clone(), encoder);

            self.transition_pipeline.render(
                &self.textures,
                &surface_view,
                encoder,
                &AnimationState {
                    centre: Vec2::ZERO,
                    radius: transition.amount(),
                    direction: 1.0,
                },
            );

            // TODO(hack3rmann): we can avoid copying the texture by swapping bind groups
            // with third intermediate texture
            encoder.copy_texture_to_texture(
                surface.as_image_copy(),
                self.textures.state.as_image_copy(),
                surface.size(),
            );
        }

        result
    }

    pub fn wallpapers_mut(&mut self) -> &mut [PreparedWallpaper] {
        &mut self.executing
    }
}
