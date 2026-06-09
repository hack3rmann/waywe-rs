use super::wallpaper::Wallpaper;
use crate::{
    Monitor,
    gpu::Gpu,
    mesh::{CommandEncoder, SurfaceView},
    plugin::Plugin,
    render::{Render, RenderGpu, RenderSet},
};
use bevy_ecs::prelude::*;
use derive_more::Deref;
use glam::Vec3;
use waywe_runtime::shaders::ShaderDescriptor;

pub struct ClearScreenPlugin;

impl Plugin for ClearScreenPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper
            .render
            .init_resource::<ClearColor>()
            .init_resource::<ClearPipeline>()
            .add_systems(
                Render,
                (
                    run_clear_pass.in_set(RenderSet::ClearPass),
                    resize_clear_pass
                        .in_set(RenderSet::Reconfigure)
                        .run_if(resource_changed::<Monitor>),
                ),
            );
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct ClearColor(pub Vec3);

impl ClearColor {
    pub const fn to_wgpu(self) -> wgpu::Color {
        wgpu::Color {
            r: self.0.x as f64,
            g: self.0.y as f64,
            b: self.0.z as f64,
            a: 1.0,
        }
    }
}

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/waywe-scene/src/shaders/noop-vertex.glsl",
    stage = "vertex"
)]
pub struct NoOpVertexShader;

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/waywe-scene/src/shaders/noop-fragment.glsl",
    stage = "fragment"
)]
pub struct NoOpFragmentShader;

#[derive(Resource, Deref)]
pub struct ClearPipeline(pub wgpu::RenderPipeline);

impl ClearPipeline {
    pub fn new(gpu: &Gpu, monitor: Monitor) -> Self {
        gpu.require_shader::<NoOpVertexShader>();
        gpu.require_shader::<NoOpFragmentShader>();

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image-pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &gpu.shader_cache.get::<NoOpVertexShader>().unwrap(),
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &gpu.shader_cache.get::<NoOpFragmentShader>().unwrap(),
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    targets: &[Some(wgpu::ColorTargetState {
                        format: monitor.surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::empty(),
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            });

        Self(pipeline)
    }
}

impl FromWorld for ClearPipeline {
    fn from_world(world: &mut World) -> Self {
        let gpu = world.resource::<RenderGpu>();
        let monitor = world.resource::<Monitor>();
        Self::new(gpu, *monitor)
    }
}

pub fn run_clear_pass(
    mut encoder: ResMut<CommandEncoder>,
    surface: Res<SurfaceView>,
    pipeline: Res<ClearPipeline>,
    color: Res<ClearColor>,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("clear-pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &surface,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color.to_wgpu()),
                // NOTE(hack3rmann): must be `Store` so wgpu marks the surface
                // initialized. `Discard` leaves HAL-imported textures in an
                // uninitialized state, and the next `Load` pass panics with
                // `NoValidTextureClearMode`.
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    pass.set_pipeline(&pipeline);
    pass.draw(0..0, 0..0);
}

pub fn resize_clear_pass(
    gpu: Res<RenderGpu>,
    monitor: Res<Monitor>,
    mut pipeline: ResMut<ClearPipeline>,
) {
    *pipeline = ClearPipeline::new(&gpu, *monitor);
}
