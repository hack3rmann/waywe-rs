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

use crate::{
    Monitor,
    asset_server::{AssetHandle, AssetId},
    assets::{
        Asset, AssetsExtract, AssetsPlugin, RefAssets, RefAssetsPlugin,
        RefAssetsRefDependencyPlugin, RenderAsset, RenderAssets, RenderAssetsPlugin,
    },
    effects::EFFECTS_TEXTURE_USAGES,
    extract::Extract,
    image::ImageMaterial,
    material::{Material, MaterialSet, RenderMaterial, RenderMaterialId},
    plugin::Plugin,
    render::{EntityMap, MainEntity, Render, RenderGpu, RenderSet, SceneExtract},
    time::Time,
    transform::{GlobalTransform, ModelMatrix, Transform},
    video::VideoMaterial,
    wallpaper::Wallpaper,
};
use bevy_ecs::{
    prelude::*,
    system::{SystemParamItem, lifetimeless::SRes},
};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3};
use itertools::Itertools;
use smallvec::{SmallVec, smallvec};
use std::{
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
use waywe_runtime::{gpu::Wgpu, wayland::MonitorId};

/// Plugin for mesh rendering functionality.
///
/// Adds systems and resources for rendering geometric shapes.
pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins((
            AssetsPlugin::<Mesh>::new(),
            RenderAssetsPlugin::<RenderMesh>::extract_new(),
            RefAssetsPlugin::<MeshPipeline>::new(),
            RefAssetsRefDependencyPlugin::<MeshPipeline, RenderMaterial>::new(),
        ));

        wallpaper
            .render
            .add_systems(
                SceneExtract,
                (
                    // TODO(hack3rmann): maybe pack it into a generic plugin
                    extract_objects::<ImageMaterial>
                        .after(AssetsExtract::MainToRender)
                        .after(MaterialSet::ExtractRender),
                    extract_objects::<VideoMaterial>
                        .after(AssetsExtract::MainToRender)
                        .after(MaterialSet::ExtractRender),
                    despawn_removed_entities,
                ),
            )
            .add_systems(Render, render_meshes.in_set(RenderSet::Render));
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

/// Render pipeline for meshes.
#[derive(Debug)]
pub struct MeshPipeline {
    /// Pipeline layout.
    pub layout: wgpu::PipelineLayout,
    /// Render pipeline.
    pub pipeline: wgpu::RenderPipeline,
}

impl Asset for MeshPipeline {}

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
    pub n_vertices: u64,
}

impl RenderAsset for RenderMesh {
    type Asset = Mesh;
    type Param = SRes<RenderGpu>;

    const REPLACE_ON_UPDATE: bool = false;

    fn extract(mesh: &Self::Asset, gpu: &mut SystemParamItem<'_, '_, Self::Param>) -> Self {
        Self::new(mesh, gpu)
    }

    fn update(&mut self, source: &Self::Asset, gpu: &mut SystemParamItem<'_, '_, Self::Param>) {
        self.update_buffer(source, gpu);
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
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&mesh.vertices),
            });

        Self {
            vertices,
            n_vertices: mesh.vertices.len() as u64,
        }
    }

    pub fn update_buffer(&mut self, mesh: &Mesh, gpu: &Wgpu) {
        // Not enough capacity
        if self.vertices.size() < mem::size_of_val(mesh.vertices.as_slice()) as u64 {
            *self = Self::new(mesh, gpu);
            return;
        }

        gpu.queue
            .write_buffer(&self.vertices, 0, bytemuck::cast_slice(&mesh.vertices));
        let index = gpu.queue.submit([]);
        gpu.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: None,
            })
            .unwrap();

        self.n_vertices = mesh.vertices.len() as u64;
    }

    pub fn buffer_slice(&self) -> wgpu::BufferSlice<'_> {
        self.vertices
            .slice(..self.n_vertices * mem::size_of::<Vertex>() as u64)
    }
}

/// Handle to a render mesh component.
#[derive(Component, Clone, Copy, Debug)]
#[require(ModelMatrix)]
pub struct RenderMeshId(pub AssetId);

/// System to extract mesh objects for rendering.
pub fn extract_objects<M: Material>(
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
    monitor: Res<Monitor>,
    mesh_query: Extract<
        Query<(Entity, &Mesh3d, &MeshMaterial<M>, &GlobalTransform), Changed<Mesh3d>>,
    >,
    gpu: Res<RenderGpu>,
    materials: Res<RefAssets<RenderMaterial>>,
    mut pipelines: ResMut<RefAssets<MeshPipeline>>,
) {
    for (id, Mesh3d(mesh), MeshMaterial(material), transform) in &mesh_query {
        let render_id = commands
            .spawn((
                MainEntity(id),
                RenderMeshId(mesh.id()),
                RenderMaterialId(material.id()),
                ModelMatrix(transform.0.to_model()),
            ))
            .id();

        let render_material = materials.get(material.id()).unwrap();

        pipelines.insert_with(material.id(), || {
            MeshPipeline::new(&gpu, monitor.id, render_material)
        });

        entity_map.insert(id, render_id);
    }
}

pub fn despawn_removed_entities(
    mut commands: Commands,
    mut despawned: Extract<RemovedComponents<Mesh3d>>,
    mut entity_map: ResMut<EntityMap>,
) {
    for id in despawned.read() {
        let Some(render_id) = entity_map.remove(&id) else {
            continue;
        };

        let Ok(mut render_entity) = commands.get_entity(render_id) else {
            continue;
        };

        render_entity.despawn();
    }
}

/// System to render meshes.
pub fn render_meshes(
    mut mesh_handles_cache: Local<Vec<(RenderMeshId, ModelMatrix)>>,
    pipelines: Res<RefAssets<MeshPipeline>>,
    materials: Res<RefAssets<RenderMaterial>>,
    meshes: Res<RenderAssets<RenderMesh>>,
    mesh_handles: Query<(&RenderMeshId, &ModelMatrix, &RenderMaterialId)>,
    mut encoder: ResMut<CommandEncoder>,
    time: Res<Time>,
    monitor: Res<Monitor>,
    surface_view: Res<SurfaceView>,
) {
    let mesh_handles = mesh_handles
        .iter()
        .sort::<&RenderMaterialId>()
        .chunk_by(|&(_, _, handle)| handle);

    for (&RenderMaterialId(material_id), mesh_handles) in &mesh_handles {
        mesh_handles_cache.clear();
        mesh_handles_cache.extend(mesh_handles.map(|(&id, &model, _)| (id, model)));
        mesh_handles_cache.sort_unstable_by_key(|&(RenderMeshId(id), _)| id);

        let mesh_handles = mesh_handles_cache
            .iter()
            .chunk_by(|&(RenderMeshId(id), _)| id);

        let aspect_ratio = monitor.aspect_ratio();
        let camera_view =
            Mat4::orthographic_rh(-1.0, 1.0, -aspect_ratio, aspect_ratio, -10.0, 10.0);

        let pipeline = pipelines.get(material_id).unwrap();
        let material = materials.get(material_id).unwrap();

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("test-mesh-render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
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

        for (&mesh_id, model_matrices) in &mesh_handles {
            let mesh = meshes.get(mesh_id).unwrap();
            pass.set_vertex_buffer(0, mesh.buffer_slice());

            for &(_, ModelMatrix(model)) in model_matrices {
                pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    0,
                    bytemuck::bytes_of(&PushConst {
                        time: time.elapsed.as_secs_f32(),
                        mvp: camera_view * model,
                        _padding: [0; 3],
                    }),
                );

                pass.draw(0..mesh.n_vertices as u32, 0..1);
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct SurfaceView {
    pub surface: Option<wgpu::TextureView>,
    pub effect: Option<wgpu::TextureView>,
}

impl SurfaceView {
    pub fn init_effect(&mut self, gpu: &Wgpu, monitor_id: MonitorId) {
        let (size, format) = {
            let surfaces = gpu.surfaces.read().unwrap();
            let surface = &surfaces[&monitor_id];

            (
                wgpu::Extent3d {
                    width: surface.config.width,
                    height: surface.config.height,
                    depth_or_array_layers: 1,
                },
                surface.format,
            )
        };

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("effect"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: EFFECTS_TEXTURE_USAGES,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());

        self.effect = Some(view);
    }
}

impl Deref for SurfaceView {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        self.effect
            .as_ref()
            .unwrap_or_else(|| self.surface.as_ref().unwrap())
    }
}

#[derive(Resource)]
pub struct CommandEncoder(NonNull<wgpu::CommandEncoder>);

unsafe impl Send for CommandEncoder {}
unsafe impl Sync for CommandEncoder {}

impl CommandEncoder {
    /// # Safety
    ///
    /// - `encoder` must not be accessed while this struct exist
    pub const unsafe fn new(encoder: &mut wgpu::CommandEncoder) -> Self {
        Self(NonNull::from_mut(encoder))
    }

    pub const fn get(&self) -> &wgpu::CommandEncoder {
        unsafe { self.0.as_ref() }
    }

    pub const fn get_mut(&mut self) -> &mut wgpu::CommandEncoder {
        unsafe { self.0.as_mut() }
    }
}

impl Deref for CommandEncoder {
    type Target = wgpu::CommandEncoder;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl DerefMut for CommandEncoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}
