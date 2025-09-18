use super::render::SceneRenderer;
use crate::{
    runtime::{
        gpu::Wgpu,
        wayland::{MonitorId, MonitorMap},
    },
    wallpaper::scene::{
        MainWorld, Monitor, Time,
        assets::{AssetHandle, Assets, RenderAssets},
        material::{Material, RenderMaterial, RenderMaterialHandle},
        render::{
            EntityMap, Extract, MainEntity, MonitorPlugged, MonitorUnplugged, RenderGpu,
            RenderPlugin, SceneExtract, SceneRender, SceneRenderStage,
        },
        transform::{GlobalTransform, ModelMatrix, Transform},
    },
};
use bevy_ecs::prelude::*;
use bytemuck::{Pod, Zeroable};
use derive_more::{Deref, DerefMut};
use for_sure::prelude::*;
use glam::{Mat4, Vec3};
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
    pub model: Mat4,
    pub time: f32,
    /// Padding for shaders, should be zeroed
    pub _padding: [u32; 3],
}

pub struct MeshPipeline {
    pub layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
}

impl MeshPipeline {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId) -> Self {
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
                bind_group_layouts: &[],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

        Self { layout, pipeline }
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

#[derive(Component, Default)]
#[require(Transform)]
pub struct Mesh;

#[derive(Component)]
#[require(Mesh)]
pub struct MeshMaterial<M: Material>(pub AssetHandle<M>);

#[derive(Component)]
#[require(ModelMatrix)]
pub struct RenderMesh {
    pub vertices: wgpu::Buffer,
}

impl RenderMesh {
    pub fn new(gpu: &Wgpu) -> Self {
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

        Self { vertices }
    }
}

#[derive(Component, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct AttachedMonitor(pub MonitorId);

#[derive(Deref, DerefMut)]
pub struct Meshes<'s>(pub QueryState<(Entity, &'s Mesh), Changed<Mesh>>);

impl FromWorld for Meshes<'_> {
    fn from_world(world: &mut World) -> Self {
        let main_world = world.resource::<MainWorld>();
        Self(QueryState::try_new(main_world).unwrap())
    }
}

pub fn extract_meshes(
    mut entity_map: ResMut<EntityMap>,
    monitor_id: Extract<Res<Monitor>>,
    meshes: Extract<Query<(Entity, &Mesh, &GlobalTransform), Changed<Mesh>>>,
    mut commands: Commands,
    gpu: Res<RenderGpu>,
) {
    let monitor_id = monitor_id.0;

    for (id, _mesh, transform) in &meshes {
        let render_id = commands
            .spawn((
                MainEntity(id),
                RenderMesh::new(&gpu),
                AttachedMonitor(monitor_id),
                ModelMatrix(transform.0.to_model()),
            ))
            .id();

        entity_map.insert(id, render_id);
    }
}

pub fn extract_meshes_material<M: Material>(
    mut entity_map: ResMut<EntityMap>,
    monitor_id: Extract<Res<Monitor>>,
    _materials: Res<RenderAssets<M::RenderAsset>>,
    meshes: Extract<Query<(Entity, &Mesh, &MeshMaterial<M>, &GlobalTransform), Changed<Mesh>>>,
    mut commands: Commands,
    gpu: Res<RenderGpu>,
) {
    let monitor_id = monitor_id.0;

    for (id, _mesh, &MeshMaterial(_material_id), transform) in &meshes {
        let render_id = commands
            .spawn((
                MainEntity(id),
                RenderMesh::new(&gpu),
                AttachedMonitor(monitor_id),
                ModelMatrix(transform.0.to_model()),
            ))
            .id();

        entity_map.insert(id, render_id);
    }
}

pub fn render_meshes(
    pipelines: Res<Pipelines>,
    materials: Res<Assets<RenderMaterial>>,
    meshes: Query<(
        &RenderMesh,
        &RenderMaterialHandle,
        &AttachedMonitor,
        &ModelMatrix,
    )>,
    mut render: ResMut<OngoingRender>,
    time: Res<Time>,
) {
    let meshes = meshes
        .iter()
        .sort::<&AttachedMonitor>()
        .chunk_by(|&(_, _, id, _)| id);

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

            for (mesh, &RenderMaterialHandle(material_id), _, &ModelMatrix(model)) in meshes {
                let material = materials.get(material_id).unwrap();

                pass.set_vertex_buffer(0, mesh.vertices.slice(..));

                pass.set_bind_group(0, &material.bind_group, &[]);

                pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    0,
                    bytemuck::bytes_of(&PushConst {
                        time: time.elapsed.as_secs_f32(),
                        model,
                        _padding: [0; 3],
                    }),
                );

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
