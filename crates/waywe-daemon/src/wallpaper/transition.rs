use crate::wallpaper::optimized::OptimizedWallpaper;
use bytemuck::{Pod, Zeroable};
use for_sure::prelude::*;
use glam::{UVec2, Vec2};
use smallvec::SmallVec;
use std::{
    collections::VecDeque,
    f32::consts::PI,
    mem,
    time::{Duration, Instant},
};
use waywe_ipc::config::{
    Angle, Animation, AnimationConfig, AnimationDirection, AnimationStyle, CenterPosition,
    Interpolation,
};
use waywe_runtime::{
    effects::{Effects, config::EffectsBuilder},
    frame::{FrameError, FrameInfo},
    gpu::Wgpu,
    shaders::ShaderDescriptor,
    wayland::MonitorId,
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const SCREEN_TRIANGLE: [Vec2; 3] = [
    Vec2::new(-1.0, -1.0),
    Vec2::new(3.0, -1.0),
    Vec2::new(-1.0, 3.0),
];

pub struct WallpaperTransitionState {
    pub from: wgpu::TextureView,
    pub to: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

impl WallpaperTransitionState {
    pub fn new(gpu: &Wgpu, pipeline: &WallpaperTransitionPipeline) -> Self {
        let surfaces = gpu.surfaces.read().unwrap();
        let surface_config = &surfaces[&pipeline.monitor_id].config;

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("transition"),
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[surface_config.format.remove_srgb_suffix()],
        };

        let from_texture = gpu.device.create_texture(&texture_desc);
        let to_texture = gpu.device.create_texture(&texture_desc);

        let from = from_texture.create_view(&Default::default());
        let to = to_texture.create_view(&Default::default());

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

pub struct TransitionCircleFragmentShader;

impl ShaderDescriptor for TransitionCircleFragmentShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../shaders/transition-circle.glsl").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        }
    }
}

pub struct TransitionSlideFragmentShader;

impl ShaderDescriptor for TransitionSlideFragmentShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../shaders/transition-slide.glsl").into(),
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
    pub pipeline_cache: wgpu::PipelineCache,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub surface_format: wgpu::TextureFormat,
}

impl WallpaperTransitionPipeline {
    pub fn create_pipeline<V: ShaderDescriptor, F: ShaderDescriptor>(
        gpu: &Wgpu,
        layout: &wgpu::PipelineLayout,
        surface_format: wgpu::TextureFormat,
        pipeline_cache: &wgpu::PipelineCache,
    ) -> wgpu::RenderPipeline {
        gpu.require_shader::<V>();
        gpu.require_shader::<F>();

        gpu.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image-pipeline"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &gpu.shader_cache.get::<V>().unwrap(),
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
                    module: &gpu.shader_cache.get::<F>().unwrap(),
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
                cache: Some(pipeline_cache),
            })
    }

    pub fn new(gpu: &Wgpu, monitor_id: MonitorId, animation_style: AnimationStyle) -> Self {
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

        // Safety: data is None
        let pipeline_cache = unsafe {
            gpu.device
                .create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                    data: None,
                    label: None,
                    fallback: false,
                })
        };

        let pipeline = match animation_style {
            AnimationStyle::Slide => Self::create_pipeline::<
                FullScreenVertexShader,
                TransitionSlideFragmentShader,
            >(
                gpu, &pipeline_layout, surface_format, &pipeline_cache
            ),

            AnimationStyle::Circle => Self::create_pipeline::<
                FullScreenVertexShader,
                TransitionCircleFragmentShader,
            >(
                gpu, &pipeline_layout, surface_format, &pipeline_cache
            ),
        };

        let sampler = gpu.device.create_sampler(&Default::default());

        Self {
            monitor_id,
            pipeline_cache,
            pipeline,
            pipeline_layout,
            bind_group_layout,
            surface_format,
            sampler,
            vertices,
        }
    }

    pub fn switch_shader<F: ShaderDescriptor>(&mut self, gpu: &Wgpu) {
        self.pipeline = Self::create_pipeline::<FullScreenVertexShader, F>(
            gpu,
            &self.pipeline_layout,
            self.surface_format,
            &self.pipeline_cache,
        );
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
        pass.set_push_constants(wgpu::ShaderStages::FRAGMENT, 0, animation_state.bytes());

        pass.draw(0..SCREEN_TRIANGLE.len() as u32, 0..1);
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum AnimationState {
    Circle(CircleAnimationState),
    Slide(SlideAnimationState),
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::Circle(CircleAnimationState::default())
    }
}

impl AnimationState {
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Circle(circle) => bytemuck::bytes_of(circle),
            Self::Slide(slide) => bytemuck::bytes_of(slide),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default, Pod, Zeroable)]
pub struct CircleAnimationState {
    pub centre: Vec2,
    pub radius: f32,
    pub direction: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default, Pod, Zeroable)]
pub struct SlideAnimationState {
    pub position: Vec2,
    pub normal: Vec2,
}

fn corners_with_aspect_ratio(aspect_ratio: f32) -> [Vec2; 4] {
    [
        Vec2::new(-1.0 / aspect_ratio, -1.0),
        Vec2::new(1.0 / aspect_ratio, -1.0),
        Vec2::new(1.0 / aspect_ratio, 1.0),
        Vec2::new(-1.0 / aspect_ratio, 1.0),
    ]
}

#[derive(Clone, Debug, PartialEq)]
pub enum OngoingTransition {
    Slide(SlideTransition),
    Circular(CircularTransition),
}

impl OngoingTransition {
    pub fn new(aspect_ratio: f32, config: &AnimationConfig) -> Self {
        let duration = Duration::from_millis(config.duration_milliseconds);

        match config.animation {
            Animation::Circle {
                center_position,
                direction,
            } => Self::Circular(CircularTransition::new(
                aspect_ratio,
                center_position,
                direction,
                duration,
            )),
            Animation::Slide { angle } => {
                Self::Slide(SlideTransition::new(aspect_ratio, angle, duration))
            }
        }
    }

    pub fn update(&mut self) {
        match self {
            Self::Circular(circular) => circular.update(),
            Self::Slide(slide) => slide.update(),
        }
    }

    pub fn is_finished(&self) -> bool {
        match self {
            Self::Circular(circular) => circular.is_finished(),
            Self::Slide(slide) => slide.is_finished(),
        }
    }

    pub fn state(&self, ease: Interpolation) -> AnimationState {
        match self {
            Self::Circular(circular) => AnimationState::Circle(circular.state(ease)),
            Self::Slide(slide) => AnimationState::Slide(slide.state(ease)),
        }
    }

    pub fn animation_style(&self) -> AnimationStyle {
        match self {
            Self::Circular(..) => AnimationStyle::Circle,
            Self::Slide(..) => AnimationStyle::Slide,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SlideTransition {
    /// Amount of work done in 0..=1 (normalized time)
    pub done_fraction: f32,
    pub scale: f32,
    pub start_time: Instant,
    pub animation_duration: Duration,
    pub position: Vec2,
    pub normal: Vec2,
}

impl SlideTransition {
    pub fn new(aspect_ratio: f32, angle: Angle, duraition: Duration) -> Self {
        let corners = corners_with_aspect_ratio(aspect_ratio);

        let angle = angle.get_radians();

        let normal = Vec2::new(angle.cos(), angle.sin());

        let scale = 2.0 * (angle.sin().abs() + angle.cos().abs() / aspect_ratio);
        let corner_idx = ((2.0 * angle / PI) as i32).rem_euclid(4);

        let position = corners[corner_idx as usize];

        Self {
            done_fraction: 0.0,
            scale,
            start_time: Instant::now(),
            animation_duration: duraition,
            normal,
            position,
        }
    }

    pub fn update(&mut self) {
        let total = self.start_time.elapsed().as_secs_f32() / self.animation_duration.as_secs_f32();
        self.done_fraction = total.min(1.0);
    }

    pub fn is_finished(&self) -> bool {
        self.done_fraction >= 1.0
    }

    #[inline]
    pub fn amount_with_easing(&self, ease: Interpolation) -> f32 {
        ease.get(self.done_fraction) * self.scale
    }

    pub fn state(&self, ease: Interpolation) -> SlideAnimationState {
        SlideAnimationState {
            normal: self.normal,
            position: self.position + self.amount_with_easing(ease) * self.normal,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct CircularTransition {
    /// Amount of work done in 0..=1 (normalized time)
    pub done_fraction: f32,
    pub scale: f32,
    pub start_time: Instant,
    pub animation_duration: Duration,
    pub direction: AnimationDirection,
    pub centre: Vec2,
}

impl CircularTransition {
    pub fn new(
        aspect_ratio: f32,
        center_position: CenterPosition,
        direction: AnimationDirection,
        duration: Duration,
    ) -> Self {
        let corners = corners_with_aspect_ratio(aspect_ratio);

        let stretched_centre = center_position.get();
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
            animation_duration: duration,
            direction,
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

    #[inline]
    pub fn amount_with_easing(&self, ease: Interpolation) -> f32 {
        let t = match self.direction {
            AnimationDirection::Out => self.done_fraction,
            AnimationDirection::In => 1.0 - self.done_fraction,
        };
        ease.get(t) * self.scale
    }

    pub fn direction(&self) -> f32 {
        match self.direction {
            AnimationDirection::Out => 1.0,
            AnimationDirection::In => -1.0,
        }
    }

    pub fn state(&self, ease: Interpolation) -> CircleAnimationState {
        CircleAnimationState {
            centre: self.centre(),
            radius: self.amount_with_easing(ease),
            direction: self.direction(),
        }
    }
}

pub struct EffectWallpaper {
    pub wallpaper: OptimizedWallpaper,
    pub effects: Effects,
}

impl EffectWallpaper {
    pub const fn new(wallpaper: OptimizedWallpaper) -> Self {
        Self {
            wallpaper,
            effects: Effects::new(),
        }
    }

    pub fn frame(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo {
        let info = self.wallpaper.frame(gpu, surface, encoder);
        self.effects.render(gpu, surface, encoder);
        info
    }
}

pub struct RunningWallpapers {
    pub monitor_id: MonitorId,
    pub aspect_ratio: f32,
    pub executing: VecDeque<EffectWallpaper>,
    pub ongoing_transitions: SmallVec<[OngoingTransition; 8]>,
    pub transition_pipeline: Almost<WallpaperTransitionPipeline>,
    pub textures: Almost<WallpaperTransitionState>,
    pub config: AnimationConfig,
    pub effects_builder: EffectsBuilder,
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
            effects_builder: EffectsBuilder::new(monitor_id),
        }
    }

    pub fn enqueue_wallpaper(&mut self, gpu: &Wgpu, wallpaper: OptimizedWallpaper) {
        self.executing.push_back(EffectWallpaper {
            wallpaper,
            effects: self.effects_builder.build(gpu),
        });

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
            self.transition_pipeline = Value(WallpaperTransitionPipeline::new(
                gpu,
                self.monitor_id,
                self.config.animation.style(),
            ));
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
            return Ok(wallpaper.frame(gpu, &surface_view, encoder));
        }

        let mut wallpapers = self.executing.iter_mut();

        let Some(first) = wallpapers.next() else {
            unreachable!()
        };

        let mut frame_result = first.frame(gpu, &self.textures.from, encoder);

        for (wallpaper, transition) in wallpapers.zip(&mut self.ongoing_transitions) {
            transition.update();

            let frame_info = wallpaper.frame(gpu, &self.textures.to, encoder);
            frame_result = frame_result.min_or_60_fps(frame_info);

            let state = transition.state(self.config.easing);

            if transition.animation_style() != self.config.animation.style() {
                match transition.animation_style() {
                    AnimationStyle::Circle => self
                        .transition_pipeline
                        .switch_shader::<TransitionCircleFragmentShader>(gpu),
                    AnimationStyle::Slide => self
                        .transition_pipeline
                        .switch_shader::<TransitionSlideFragmentShader>(gpu),
                }
            }

            self.transition_pipeline
                .render(&self.textures, &surface_view, encoder, &state);

            // TODO(hack3rmann): we can avoid copying the texture by swapping bind groups
            // with third intermediate texture
            encoder.copy_texture_to_texture(
                surface.as_image_copy(),
                self.textures.from.texture().as_image_copy(),
                surface.size(),
            );
        }

        Ok(FrameInfo {
            target_frame_time: Some(frame_result.target_frame_time.unwrap_or(FrameInfo::MAX_FPS)),
        })
    }

    pub fn wallpapers_mut(&mut self) -> &mut [EffectWallpaper] {
        self.executing.make_contiguous()
    }
}
