use bytemuck::{Pod, Zeroable};
use for_sure::prelude::*;
use glam::{UVec2, Vec2};
use smallvec::SmallVec;
use std::{
    collections::VecDeque,
    mem,
    time::{Duration, Instant},
};
use waywe_ipc::config::{AnimationConfig, AnimationDirection};
use waywe_runtime::{
    frame::{FrameError, FrameInfo},
    gpu::Wgpu,
    shaders::ShaderDescriptor,
    wayland::MonitorId,
};
use waywe_scene::wallpaper::PreparedWallpaper;
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
}

pub struct FullScreenVertexShader;

impl ShaderDescriptor for FullScreenVertexShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../shaders/fullscreen-vertex.glsl").into(),
                stage: wgpu::naga::ShaderStage::Vertex,
                defines: Default::default(),
            },
        }
    }
}

pub struct FullScreenFragmentShader;

impl ShaderDescriptor for FullScreenFragmentShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../shaders/transition.glsl").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        }
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
        gpu.require_shader::<FullScreenVertexShader>();
        gpu.require_shader::<FullScreenFragmentShader>();

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

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &gpu.shader_cache.get::<FullScreenVertexShader>().unwrap(),
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
                    module: &gpu.shader_cache.get::<FullScreenFragmentShader>().unwrap(),
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
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
    pub direction: AnimationDirection,
    pub centre: Vec2,
}

impl OngoingTransition {
    pub fn new(aspect_ratio: f32, config: &AnimationConfig) -> Self {
        let corners = [
            Vec2::new(-1.0 / aspect_ratio, -1.0),
            Vec2::new(1.0 / aspect_ratio, -1.0),
            Vec2::new(1.0 / aspect_ratio, 1.0),
            Vec2::new(-1.0 / aspect_ratio, 1.0),
        ];

        let stretched_centre = config.center_position.get();
        let centre = Vec2::new(stretched_centre.x / aspect_ratio, stretched_centre.y);

        let scale = centre
            .distance(corners[0])
            .max(centre.distance(corners[1]))
            .max(centre.distance(corners[2]))
            .max(centre.distance(corners[3]));

        Self {
            done_fraction: 0.0,
            scale,
            start_time: Instant::now(),
            animation_duration: Duration::from_millis(config.duration_milliseconds),
            direction: config.direction,
            centre,
        }
    }

    pub fn centre(&self) -> Vec2 {
        self.centre
    }

    pub fn update(&mut self) {
        let total = self.start_time.elapsed().as_secs_f32() / self.animation_duration.as_secs_f32();
        self.done_fraction = total.min(1.0);
    }

    pub fn is_finished(&self) -> bool {
        self.done_fraction >= 1.0
    }

    pub fn amount(&self) -> f32 {
        self.amount_with_easing(|t| t)
    }

    #[inline]
    pub fn amount_with_easing(&self, ease: impl FnOnce(f32) -> f32) -> f32 {
        let t = match self.direction {
            AnimationDirection::Out => self.done_fraction,
            AnimationDirection::In => 1.0 - self.done_fraction,
        };
        ease(t) * self.scale
    }

    pub fn direction(&self) -> f32 {
        match self.direction {
            AnimationDirection::Out => 1.0,
            AnimationDirection::In => -1.0,
        }
    }
}

pub struct RunningWallpapers {
    pub monitor_id: MonitorId,
    pub aspect_ratio: f32,
    pub executing: VecDeque<PreparedWallpaper>,
    pub ongoing_transitions: SmallVec<[OngoingTransition; 8]>,
    pub transition_pipeline: Almost<WallpaperTransitionPipeline>,
    pub textures: Almost<WallpaperTransitionState>,
    pub config: AnimationConfig,
}

impl RunningWallpapers {
    pub const fn new(monitor_id: MonitorId, monitor_size: UVec2, config: AnimationConfig) -> Self {
        Self {
            monitor_id,
            aspect_ratio: monitor_size.y as f32 / monitor_size.x as f32,
            executing: VecDeque::new(),
            ongoing_transitions: SmallVec::new_const(),
            transition_pipeline: Nil,
            textures: Nil,
            config,
        }
    }

    pub fn enqueue_wallpaper(&mut self, wallpaper: PreparedWallpaper) {
        self.executing.push_back(wallpaper);

        if self.executing.len() >= 2 {
            self.ongoing_transitions
                .push(OngoingTransition::new(self.aspect_ratio, &self.config));
        }
    }

    pub fn remove_finished(&mut self) {
        let n_unfinished = self
            .ongoing_transitions
            .iter()
            .take_while(|t| t.is_finished())
            .count();

        _ = self.ongoing_transitions.drain(..n_unfinished);
        _ = self.executing.drain(..n_unfinished);
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

        if self.executing.is_empty() {
            return Err(FrameError::NoWorkToDo);
        } else if self.executing.len() == 1 {
            let Some(wallpaper) = self.executing.front_mut() else {
                unreachable!()
            };
            return Ok(wallpaper.frame(surface_view, encoder));
        }

        let mut wallpapers = self.executing.iter_mut();

        let Some(first) = wallpapers.next() else {
            unreachable!()
        };

        let mut frame_result = first.frame(self.textures.from.clone(), encoder);

        for (wallpaper, transition) in wallpapers.zip(&mut self.ongoing_transitions) {
            transition.update();

            let frame_info = wallpaper.frame(self.textures.to.clone(), encoder);
            frame_result = frame_result.min_or_60_fps(frame_info);

            let state = AnimationState {
                centre: transition.centre(),
                radius: transition.amount_with_easing(self.config.easing.get()),
                direction: transition.direction(),
            };

            self.transition_pipeline
                .render(&self.textures, &surface_view, encoder, &state);

            // TODO(hack3rmann): we can avoid copying the texture by swapping bind groups
            // with third intermediate texture
            encoder.copy_texture_to_texture(
                surface.as_image_copy(),
                self.textures.state.as_image_copy(),
                surface.size(),
            );
        }

        Ok(frame_result)
    }

    pub fn wallpapers_mut(&mut self) -> &mut [PreparedWallpaper] {
        self.executing.make_contiguous()
    }
}
