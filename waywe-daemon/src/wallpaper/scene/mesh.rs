//! Mesh rendering system for geometric shapes.
//!
//! This module provides components and systems for rendering geometric
//! shapes (meshes) with materials.
//!
//! # Core Types
//!
//! - [`Mesh`]: Geometric shape data
//! - [`Mesh3d`]: Component to render a mesh
//! - [`MeshMaterial`]: Material to apply to a mesh
//! - [`RenderMesh`]: GPU-ready mesh data
//!
//! # Components
//!
//! - [`Vertex`]: Position data for mesh vertices
//! - [`PushConst`]: Uniform data passed to shaders
//!
//! # Plugins
//!
//! - [`MeshPlugin`]: Adds mesh rendering functionality to a wallpaper

use super::wallpaper::Wallpaper;
use crate::{
    runtime::{gpu::Wgpu, wayland::MonitorId},
    wallpaper::scene::{
        Monitor,
        asset_server::{AssetHandle, AssetId},
        assets::{
            Asset, Assets, AssetsPlugin, RenderAsset, RenderAssets, RenderAssetsPlugin,
            extract_new_render_assets,
        },
        extract::Extract,
        image::{ImageMaterial, extract_image_materials},
        material::{Material, MaterialAssetMap, RenderMaterial, RenderMaterialHandle},
        plugin::Plugin,
        render::{EntityMap, MainEntity, Render, RenderGpu, SceneExtract, SceneRenderStage},
        time::Time,
        transform::{GlobalTransform, ModelMatrix, Transform},
        video::{VideoMaterial, extract_video_materials},
    },
};
use bevy_ecs::{
    prelude::*,
    system::{SystemParamItem, lifetimeless::SRes},
};
use bytemuck::{Pod, Zeroable};
use derive_more::{Deref, DerefMut};
use for_sure::prelude::*;
use glam::{Mat4, Vec2, Vec3};
use itertools::Itertools;
use smallvec::{SmallVec, smallvec};
use std::{collections::HashMap, mem};

/// Plugin for mesh rendering functionality.
///
/// Adds systems and resources for rendering geometric shapes.
pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins((
            AssetsPlugin::<Mesh>::new(),
            RenderAssetsPlugin::<RenderMesh>::extract_new(),
        ));

        wallpaper
            .render
            .init_resource::<MeshPipelines>()
            .init_resource::<OngoingRender>()
            .add_systems(
                SceneExtract,
                (
                    extact_objects::<ImageMaterial>
                        .after(extract_image_materials)
                        .after(extract_new_render_assets::<RenderMesh>),
                    extact_objects::<VideoMaterial>
                        .after(extract_video_materials)
                        .after(extract_new_render_assets::<RenderMesh>),
                ),
            )
            .add_systems(
                Render,
                (
                    prepare_render.in_set(SceneRenderStage::PreRender),
                    render_meshes.in_set(SceneRenderStage::Render),
                    finish_render.in_set(SceneRenderStage::Present),
                ),
            );
    }
}

/// Vertex position data.
#[repr(transparent)]
#[derive(Default, PartialEq, Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex(pub Vec3);

/// Push constants passed to shaders.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct PushConst {
    /// Model-view-projection matrix.
    pub mvp: Mat4,
    /// Current time for animations.
    pub time: f32,
    /// Padding for shaders, should be zeroed.
    pub _padding: [u32; 3],
}

/// Collection of mesh pipelines for a specific material.
#[derive(Default, Resource, Debug, Deref, DerefMut)]
pub struct MeshPipelines(pub HashMap<AssetId, MeshPipeline>);

/// Render pipeline for meshes.
#[derive(Debug)]
pub struct MeshPipeline {
    /// Pipeline layout.
    pub layout: wgpu::PipelineLayout,
    /// Render pipeline.
    pub pipeline: wgpu::RenderPipeline,
}

impl MeshPipeline {
    /// Create a new mesh pipeline for a specific material and monitor.
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId, material: &RenderMaterial) -> Self {
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("image-pipeline-layout"),
                bind_group_layouts: &[&material.bind_group_layout],
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
                    module: &material.shader.vertex,
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
                    module: &material.shader.fragment,
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

/// Geometric mesh asset.
#[derive(Default)]
pub struct Mesh {
    /// Vertices defining the mesh geometry.
    pub vertices: SmallVec<[Vertex; 12]>,
}

impl Asset for Mesh {}

impl Mesh {
    /// Create a rectangular mesh.
    pub fn rect(sizes: Vec2) -> Self {
        Self {
            vertices: smallvec![
                Vertex(Vec3::new(-sizes.x, -sizes.y, 0.0)),
                Vertex(Vec3::new(sizes.x, -sizes.y, 0.0)),
                Vertex(Vec3::new(sizes.x, sizes.y, 0.0)),
                Vertex(Vec3::new(-sizes.x, -sizes.y, 0.0)),
                Vertex(Vec3::new(sizes.x, sizes.y, 0.0)),
                Vertex(Vec3::new(-sizes.x, sizes.y, 0.0)),
            ],
        }
    }
}

/// Component to render a mesh.
#[derive(Clone, Debug, Component)]
#[require(Transform)]
pub struct Mesh3d(pub AssetHandle<Mesh>);

/// Component to apply a material to a mesh.
#[derive(Component)]
pub struct MeshMaterial<M: Material>(pub AssetHandle<M>);

/// GPU-ready mesh data.
#[derive(Clone, Debug)]
pub struct RenderMesh {
    /// Vertex buffer.
    pub vertices: wgpu::Buffer,
}

impl RenderAsset for RenderMesh {
    type Asset = Mesh;
    type Param = SRes<RenderGpu>;

    fn extract(mesh: &Self::Asset, gpu: &mut SystemParamItem<'_, '_, Self::Param>) -> Self {
        Self::new(mesh, gpu)
    }
}

impl RenderMesh {
    /// Create a new render mesh from mesh data.
    pub fn new(mesh: &Mesh, gpu: &Wgpu) -> Self {
        use wgpu::util::DeviceExt as _;

        let vertices = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("test-mesh-buffer"),
                usage: wgpu::BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&mesh.vertices),
            });

        Self { vertices }
    }
}

/// Handle to a render mesh component.
#[derive(Component)]
#[require(ModelMatrix)]
pub struct RenderMeshHandle(pub AssetHandle<Mesh>);

/// System to extract mesh objects for rendering.
pub fn extact_objects<M: Material>(
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
    monitor: Res<Monitor>,
    asset_map: Res<MaterialAssetMap>,
    mesh_query: Extract<
        Query<(Entity, &Mesh3d, &MeshMaterial<M>, &GlobalTransform), Changed<Mesh3d>>,
    >,
    gpu: Res<RenderGpu>,
    materials: Res<Assets<RenderMaterial>>,
    mut pipelines: ResMut<MeshPipelines>,
) {
    for (id, Mesh3d(mesh), MeshMaterial(material), transform) in &mesh_query {
        let render_material = asset_map.get(material.id()).unwrap();

        let render_id = commands
            .spawn((
                MainEntity(id),
                RenderMeshHandle(mesh.clone()),
                RenderMaterialHandle(render_material.clone()),
                ModelMatrix(transform.0.to_model()),
            ))
            .id();

        let material = materials.get(render_material.id()).unwrap();

        pipelines
            .entry(render_material.id())
            .or_insert_with(|| MeshPipeline::new(&gpu, monitor.id, material));

        entity_map.insert(id, render_id);
    }
}

/// System to render meshes.
pub fn render_meshes(
    pipelines: Res<MeshPipelines>,
    materials: Res<Assets<RenderMaterial>>,
    meshes: Res<RenderAssets<RenderMesh>>,
    mesh_handles: Query<(&RenderMeshHandle, &ModelMatrix, &RenderMaterialHandle)>,
    mut render: ResMut<OngoingRender>,
    time: Res<Time>,
    monitor: Res<Monitor>,
) {
    let mesh_handles = mesh_handles
        .iter()
        .sort::<&RenderMaterialHandle>()
        .chunk_by(|&(_, _, handle)| handle);

    for (RenderMaterialHandle(material), mesh_handles) in &mesh_handles {
        let target_surface = Almost::unwrap(Almost::take(&mut render.output));
        let aspect_ratio = monitor.aspect_ratio();
        let camera_view =
            Mat4::orthographic_rh(-1.0, 1.0, -aspect_ratio, aspect_ratio, -10.0, 10.0);

        let pipeline = pipelines.get(&material.id()).unwrap();
        let material = materials.get(material.id()).unwrap();

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
            pass.set_bind_group(0, &material.bind_group, &[]);

            for (RenderMeshHandle(mesh_handle), &ModelMatrix(model), _) in mesh_handles {
                let mesh = meshes.get(mesh_handle.id()).unwrap();
                let n_vertices = mesh.vertices.size() / mem::size_of::<Vertex>() as u64;

                pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    0,
                    bytemuck::bytes_of(&PushConst {
                        time: time.elapsed.as_secs_f32(),
                        mvp: camera_view * model,
                        _padding: [0; 3],
                    }),
                );

                pass.draw(0..n_vertices as u32, 0..1);
            }
        }

        render.output = Value(target_surface);
    }
}

/// Resource tracking ongoing render operations.
#[derive(Resource, Default)]
pub struct OngoingRender {
    /// Command encoder for building render commands.
    pub encoder: Almost<wgpu::CommandEncoder>,
    /// Current surface texture
    pub surface: Almost<wgpu::SurfaceTexture>,
    /// Surface texture view
    pub output: Almost<wgpu::TextureView>,
}

/// System to prepare for rendering.
pub fn prepare_render(
    monitor: Res<Monitor>,
    mut render: ResMut<OngoingRender>,
    gpu: Res<RenderGpu>,
) {
    render.encoder = Value(gpu.device.create_command_encoder(&Default::default()));

    let surfaces = gpu.surfaces.read().unwrap();

    render.surface = Value(surfaces[&monitor.id].surface.get_current_texture().unwrap());
    render.output = Value(render.surface.texture.create_view(&Default::default()));
}

/// System to finish rendering and present frames.
pub fn finish_render(mut render: ResMut<OngoingRender>, gpu: Res<RenderGpu>) {
    render.output = Nil;

    let encoder = Almost::unwrap(Almost::take(&mut render.encoder));
    gpu.queue.submit([encoder.finish()]);

    Almost::unwrap(Almost::take(&mut render.surface)).present();
}
