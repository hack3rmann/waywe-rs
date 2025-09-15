use super::render::SceneRenderer;
use crate::{
    runtime::{
        gpu::Wgpu,
        wayland::{MonitorId, MonitorMap},
    },
    wallpaper::scene::{
        MainWorld, Monitor, Time,
        render::{
            MainEntity, MonitorPlugged, MonitorUnplugged, RenderGpu, RenderPlugin, SceneExtract,
            SceneRender, SceneRenderStage,
        },
    },
};
use bevy_ecs::prelude::*;
use bytemuck::{Pod, Zeroable};
use derive_more::{Deref, DerefMut};
use for_sure::prelude::*;
use glam::Vec3;
use itertools::Itertools;
use std::mem;

pub struct RenderMeshPlugin;

impl RenderPlugin for RenderMeshPlugin {
    fn init(self, renderer: &mut SceneRenderer) {
        renderer.world.add_observer(add_pipeline);
        renderer.world.add_observer(remove_pipeline);
        renderer.world.init_resource::<Pipelines>();
        renderer.world.init_resource::<OngoingRender>();
        renderer.add_systems(SceneExtract, extract_meshes);
        renderer.add_systems(
            SceneRender,
            (
                prepare_render.in_set(SceneRenderStage::PreRender),
                render_meshes.in_set(SceneRenderStage::Render),
                finish_render.in_set(SceneRenderStage::Present),
            ),
        );
    }
}

#[repr(transparent)]
#[derive(Default, PartialEq, Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex(pub Vec3);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct PushConst {
    pub time: f32,
}

pub struct MeshPipeline {
    pub layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl MeshPipeline {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId) -> Self {
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("image-bind-group-layout"),
                    entries: &[],
                });

        const VERTEX_SHADER_NAME: &str = "shaders/test-vertex.glsl";
        const FRAGMENT_SHADER_NAME: &str = "shaders/test-fragment.glsl";

        gpu.use_shader(
            VERTEX_SHADER_NAME,
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Glsl {
                    shader: include_str!("../../shaders/test-vertex.glsl").into(),
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
                    shader: include_str!("../../shaders/test-fragment.glsl").into(),
                    stage: wgpu::naga::ShaderStage::Fragment,
                    defines: Default::default(),
                },
            },
        );

        let layout = gpu
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
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &gpu.shader_cache.get(VERTEX_SHADER_NAME).unwrap(),
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
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
                        format: gpu.surfaces.read().unwrap()[&monitor_id].format,
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
            layout,
            pipeline,
            bind_group_layout,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Pipelines(pub MonitorMap<MeshPipeline>);

pub fn add_pipeline(
    plugged: Trigger<MonitorPlugged>,
    mut pipelines: ResMut<Pipelines>,
    gpu: Res<RenderGpu>,
) {
    let pipeline = MeshPipeline::new(&gpu, plugged.id);
    pipelines.insert(plugged.id, pipeline);
}

pub fn remove_pipeline(unplugged: Trigger<MonitorUnplugged>, mut pipelines: ResMut<Pipelines>) {
    _ = pipelines.remove(&unplugged.id);
}

#[derive(Component)]
pub struct Mesh;

#[derive(Component)]
pub struct RenderMesh {
    pub vertices: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl RenderMesh {
    pub fn new(gpu: &Wgpu, pipeline: &MeshPipeline) -> Self {
        use wgpu::util::DeviceExt as _;

        let vertices = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("test-mesh-buffer"),
                usage: wgpu::BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&[
                    Vertex(Vec3::new(-0.5, -0.5, 0.0)),
                    Vertex(Vec3::new(0.5, -0.5, 0.0)),
                    Vertex(Vec3::new(0.0, 0.5, 0.0)),
                ]),
            });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("test-mesh-bind-group"),
            layout: &pipeline.bind_group_layout,
            entries: &[],
        });

        Self {
            vertices,
            bind_group,
        }
    }
}

#[derive(Component, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct AttachedMonitor(pub MonitorId);

pub fn extract_meshes(
    mut commands: Commands,
    main_world: Res<MainWorld>,
    gpu: Res<RenderGpu>,
    pipelines: Res<Pipelines>,
) {
    let mut meshes = QueryState::<(Entity, &Mesh), Changed<Mesh>>::try_new(&main_world).unwrap();
    let monitor_id = main_world.resource::<Monitor>().0;
    let Some(pipeline) = pipelines.get(&monitor_id) else {
        return;
    };

    for (id, _mesh) in meshes.iter(&main_world) {
        commands.spawn((
            MainEntity(id),
            RenderMesh::new(&gpu, pipeline),
            AttachedMonitor(monitor_id),
        ));
    }
}

pub fn render_meshes(
    pipelines: Res<Pipelines>,
    meshes: Query<(&RenderMesh, &AttachedMonitor, &MainEntity)>,
    mut render: ResMut<OngoingRender>,
    time: Res<Time>,
) {
    let meshes = meshes
        .iter()
        .sort::<&AttachedMonitor>()
        .chunk_by(|&(_, id, _)| id);

    for (&AttachedMonitor(monitor_id), meshes) in &meshes {
        let pipeline = &pipelines[&monitor_id];
        let target_surface = render.outputs.remove(&monitor_id).unwrap();

        {
            let mut pass = render
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("test-mesh-render"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target_surface,
                        depth_slice: None,
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

            pass.set_pipeline(&pipeline.pipeline);
            pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                0,
                bytemuck::bytes_of(&PushConst {
                    time: time.elapsed.as_secs_f32(),
                }),
            );

            for (mesh, _, &MainEntity(_main_entity)) in meshes {
                pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                pass.set_bind_group(0, &mesh.bind_group, &[]);

                let n_vertices = mesh.vertices.size() / mem::size_of::<Vertex>() as u64;
                pass.draw(0..n_vertices as u32, 0..1);
            }
        }

        _ = render.outputs.insert(monitor_id, target_surface);
    }
}

#[derive(Resource, Default)]
pub struct OngoingRender {
    pub encoder: Almost<wgpu::CommandEncoder>,
    pub surfaces: MonitorMap<wgpu::SurfaceTexture>,
    pub outputs: MonitorMap<wgpu::TextureView>,
}

pub fn prepare_render(mut render: ResMut<OngoingRender>, gpu: Res<RenderGpu>) {
    render.encoder = Value(gpu.device.create_command_encoder(&Default::default()));
    render.surfaces = gpu
        .surfaces
        .read()
        .unwrap()
        .iter()
        .map(|(&id, surface)| (id, surface.surface.get_current_texture().unwrap()))
        .collect();
    render.outputs = render
        .surfaces
        .iter()
        .map(|(&id, surface)| (id, surface.texture.create_view(&Default::default())))
        .collect();
}

pub fn finish_render(mut render: ResMut<OngoingRender>, gpu: Res<RenderGpu>) {
    render.outputs.clear();

    let encoder = Almost::unwrap(Almost::take(&mut render.encoder));
    gpu.queue.submit([encoder.finish()]);

    for surface in mem::take(&mut render.surfaces).into_values() {
        surface.present();
    }
}
