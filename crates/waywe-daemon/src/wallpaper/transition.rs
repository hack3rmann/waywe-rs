use crate::wallpaper::{Wallpaper, WallpaperConfig, optimized::OptimizedWallpaper};
use bytemuck::{Pod, Zeroable};
use for_sure::prelude::*;
use glam::Vec2;
use smallvec::SmallVec;
use std::{collections::VecDeque, f32::consts::PI, mem, time::Duration};
use waywe_ipc::config::{
    Angle, Animation, AnimationConfig, AnimationDirection, AnimationStyle, CenterPosition,
    Interpolation,
};
use waywe_runtime::{
    effects::{Effects, config::EffectsBuilder},
    frame::{FrameError, FrameInfo},
    gpu::Wgpu,
};
use waywe_spirv_derive::ShaderDescriptor;
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
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("transition"),
            size: wgpu::Extent3d {
                width: pipeline.config.surface_size.x,
                height: pipeline.config.surface_size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: pipeline.config.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[pipeline.config.surface_format.remove_srgb_suffix()],
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

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/waywe-daemon/src/shaders/fullscreen-vertex.glsl",
    stage = "vertex"
)]
pub struct FullScreenVertexShader;

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/waywe-daemon/src/shaders/transition-circle.glsl",
    stage = "fragment"
)]
pub struct TransitionCircleFragmentShader;

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/waywe-daemon/src/shaders/transition-slide.glsl",
    stage = "fragment"
)]
pub struct TransitionSlideFragmentShader;

pub struct WallpaperTransitionPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub vertices: wgpu::Buffer,
    pub pipeline_cache: wgpu::PipelineCache,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub config: WallpaperConfig,
    pub animation_style: AnimationStyle,
}

impl WallpaperTransitionPipeline {
    pub fn create_pipeline(
        gpu: &Wgpu,
        animation_style: AnimationStyle,
        layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        pipeline_cache: &wgpu::PipelineCache,
    ) -> wgpu::RenderPipeline {
        gpu.require_shader::<FullScreenVertexShader>();

        let fragment_shader = match animation_style {
            AnimationStyle::Circle => {
                gpu.require_shader::<TransitionCircleFragmentShader>();
                gpu.shader_cache
                    .get::<TransitionCircleFragmentShader>()
                    .unwrap()
            }
            AnimationStyle::Slide => {
                gpu.require_shader::<TransitionSlideFragmentShader>();
                gpu.shader_cache
                    .get::<TransitionSlideFragmentShader>()
                    .unwrap()
            }
        };

        gpu.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image-pipeline"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &gpu.shader_cache.get::<FullScreenVertexShader>().unwrap(),
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: mem::size_of_val(&SCREEN_TRIANGLE[0]) as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    })],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
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
                cache: Some(pipeline_cache),
            })
    }

    pub fn new(gpu: &Wgpu, config: WallpaperConfig, animation_style: AnimationStyle) -> Self {
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
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: mem::size_of::<AnimationState>() as u32,
            });

        // Safety: data is None
        let pipeline_cache = unsafe {
            gpu.device
                .create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                    data: None,
                    label: None,
                    fallback: false,
                })
        };

        let pipeline = Self::create_pipeline(
            gpu,
            animation_style,
            &pipeline_layout,
            config.surface_format,
            &pipeline_cache,
        );

        let sampler = gpu.device.create_sampler(&Default::default());

        Self {
            pipeline_cache,
            pipeline,
            pipeline_layout,
            bind_group_layout,
            config,
            animation_style,
            sampler,
            vertices,
        }
    }

    pub fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig) {
        if self.config.surface_format != config.surface_format {
            self.pipeline = Self::create_pipeline(
                gpu,
                self.animation_style,
                &self.pipeline_layout,
                config.surface_format,
                &self.pipeline_cache,
            );
        }

        self.config = config;
    }

    pub fn switch_shader(&mut self, gpu: &Wgpu, animation_style: AnimationStyle) {
        self.pipeline = Self::create_pipeline(
            gpu,
            animation_style,
            &self.pipeline_layout,
            self.config.surface_format,
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
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &state.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_immediates(0, animation_state.bytes());

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

    pub fn advance_time(&mut self, delta: Duration) {
        match self {
            Self::Circular(circular) => circular.advance_time(delta),
            Self::Slide(slide) => slide.advance_time(delta),
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
    pub animation_progress: Duration,
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
            animation_progress: Duration::ZERO,
            animation_duration: duraition,
            normal,
            position,
        }
    }

    pub fn advance_time(&mut self, delta: Duration) {
        self.animation_progress += delta;
        let total = self.animation_progress.as_secs_f32() / self.animation_duration.as_secs_f32();
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
    pub animation_progress: Duration,
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
            animation_progress: Duration::ZERO,
            animation_duration: duration,
            direction,
            centre,
        }
    }

    pub fn centre(&self) -> Vec2 {
        self.centre
    }

    pub fn advance_time(&mut self, delta: Duration) {
        self.animation_progress += delta;
        let total = self.animation_progress.as_secs_f32() / self.animation_duration.as_secs_f32();
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
    pub executing: VecDeque<EffectWallpaper>,
    pub ongoing_transitions: SmallVec<[OngoingTransition; 8]>,
    pub transition_pipeline: Almost<WallpaperTransitionPipeline>,
    pub textures: Almost<WallpaperTransitionState>,
    pub config: AnimationConfig,
    pub effects_builder: EffectsBuilder,
    pub wallpaper_config: WallpaperConfig,
}

impl RunningWallpapers {
    pub const fn new(wallpaper_config: WallpaperConfig, config: AnimationConfig) -> Self {
        Self {
            executing: VecDeque::new(),
            ongoing_transitions: SmallVec::new_const(),
            transition_pipeline: Nil,
            textures: Nil,
            config,
            effects_builder: EffectsBuilder::new(),
            wallpaper_config,
        }
    }

    pub fn enqueue_wallpaper(&mut self, gpu: &Wgpu, wallpaper: OptimizedWallpaper) {
        self.executing.push_back(EffectWallpaper {
            wallpaper,
            effects: self.effects_builder.build(gpu, self.wallpaper_config),
        });

        if self.executing.len() >= 2 {
            self.ongoing_transitions.push(OngoingTransition::new(
                self.wallpaper_config.aspect_ratio(),
                &self.config,
            ));
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

        if self.executing.len() <= 1 {
            self.transition_pipeline = Nil;
            self.textures = Nil;
        }
    }

    pub fn is_transitioning(&self) -> bool {
        self.executing.len() >= 2
    }

    pub fn init_transitions(&mut self, gpu: &Wgpu) {
        if self.is_transitioning() && Almost::is_nil(&self.transition_pipeline) {
            let pipeline = WallpaperTransitionPipeline::new(
                gpu,
                self.wallpaper_config,
                self.config.animation.style(),
            );

            self.textures = Value(WallpaperTransitionState::new(gpu, &pipeline));
            self.transition_pipeline = Value(pipeline);
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
            let frame_info = wallpaper.frame(gpu, &self.textures.to, encoder);
            frame_result = frame_result.min_or_60_fps(frame_info);

            let state = transition.state(self.config.easing);

            if transition.animation_style() != self.config.animation.style() {
                self.transition_pipeline
                    .switch_shader(gpu, transition.animation_style());
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

impl Wallpaper for RunningWallpapers {
    fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig) {
        if self.wallpaper_config == config {
            return;
        }

        if let Value(pipeline) = &mut self.transition_pipeline {
            pipeline.configure(gpu, config);
            *self.textures = WallpaperTransitionState::new(gpu, pipeline);
        }

        for effect in &mut self.executing {
            effect.wallpaper.configure(gpu, config);
            effect.effects = self.effects_builder.build(gpu, config);
        }

        self.wallpaper_config = config;
    }

    fn frame(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo {
        self.render(gpu, surface.texture(), encoder).unwrap()
    }

    fn advance_time(&mut self, delta: Duration) {
        for transition in &mut self.ongoing_transitions {
            transition.advance_time(delta);
        }

        for effected in &mut self.executing {
            effected.wallpaper.advance_time(delta);
        }
    }
}
