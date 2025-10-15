use crate::{
    Monitor,
    asset_server::{AssetHandle, AssetId},
    assets::{Assets, AssetsExtract, RefAssets},
    extract::Extract,
    image::{Image, ImageMaterial},
    material::{MaterialSet, RenderMaterial, RenderMaterialId},
    mesh::{Mesh, MeshPipeline, RenderMeshId},
    plugin::Plugin,
    render::{EntityMap, MainEntity, RenderGpu, SceneExtract},
    transform::{GlobalTransform, ModelMatrix, PreExtractTransform, Transform},
    video::{Video, VideoMaterial},
    wallpaper::Wallpaper,
};
use derive_more::Deref;
use glam::{UVec2, Vec2, Vec3};
use waywe_ecs::{prelude::*, system::SystemParam};

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.main.init_resource::<QuadMesh>();

        wallpaper.render.add_systems(
            SceneExtract,
            (
                extract_sprites
                    .after(AssetsExtract::MainToRender)
                    .after(MaterialSet::ExtractRender),
                despawn_sprites,
            ),
        );
    }
}

#[derive(Resource, Deref)]
pub struct QuadMesh(pub AssetHandle<Mesh>);

impl FromWorld for QuadMesh {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self(meshes.add(Mesh::rect(Vec2::ONE)))
    }
}

#[derive(Clone, Debug)]
pub enum SpriteTexture {
    Image(AssetHandle<ImageMaterial>),
    Video(AssetHandle<VideoMaterial>),
}

impl SpriteTexture {
    pub fn material_id(&self) -> AssetId {
        match self {
            SpriteTexture::Image(h) => h.id(),
            SpriteTexture::Video(h) => h.id(),
        }
    }
}

impl From<AssetHandle<ImageMaterial>> for SpriteTexture {
    fn from(value: AssetHandle<ImageMaterial>) -> Self {
        Self::Image(value)
    }
}

impl From<AssetHandle<VideoMaterial>> for SpriteTexture {
    fn from(value: AssetHandle<VideoMaterial>) -> Self {
        Self::Video(value)
    }
}

#[derive(SystemParam)]
pub struct SpriteAssets<'w> {
    image_materials: Res<'w, Assets<ImageMaterial>>,
    video_materials: Res<'w, Assets<VideoMaterial>>,
    images: Res<'w, Assets<Image>>,
    videos: Res<'w, Assets<Video>>,
}

#[derive(Component)]
#[require(Transform)]
pub struct Sprite {
    pub texture: SpriteTexture,
}

pub fn extract_sprites(
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
    monitor: Res<Monitor>,
    sprites: Extract<Query<(Entity, &Sprite, &GlobalTransform), Changed<Sprite>>>,
    gpu: Res<RenderGpu>,
    materials: Res<RefAssets<RenderMaterial>>,
    mut pipelines: ResMut<RefAssets<MeshPipeline>>,
    quad_mesh: Extract<Res<QuadMesh>>,
    assets: Extract<SpriteAssets>,
) {
    for (id, sprite, &GlobalTransform(transform)) in &sprites {
        let material_id = sprite.texture.material_id();

        let size = match &sprite.texture {
            SpriteTexture::Image(image) => {
                let image_id = assets.image_materials.get(image.id()).unwrap().image.id();
                let image = &assets.images.get(image_id).unwrap().image;
                UVec2::new(image.width(), image.height())
            }
            SpriteTexture::Video(video) => {
                let video_id = assets.video_materials.get(video.id()).unwrap().video.id();
                let video = assets.videos.get(video_id).unwrap();
                video.frame_size()
            }
        };

        let aspect_ratio = size.x as f32 / size.y as f32;
        let scale = Vec3::new(aspect_ratio, 1.0, 1.0);

        let render_id = commands
            .spawn((
                MainEntity(id),
                RenderMeshId(quad_mesh.id()),
                RenderMaterialId(material_id),
                ModelMatrix(transform.scaled_by(scale).to_model()),
                PreExtractTransform(Transform::default().scaled_by(scale)),
            ))
            .id();

        let render_material = materials.get(material_id).unwrap();

        pipelines.insert_with(material_id, || {
            MeshPipeline::new(&gpu, monitor.id, render_material)
        });

        entity_map.insert(id, render_id);
    }
}

pub fn despawn_sprites(
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
    mut sprites: Extract<RemovedComponents<Sprite>>,
) {
    for id in sprites.read() {
        let Some(render_id) = entity_map.remove(&id) else {
            continue;
        };

        let Ok(mut render_entity) = commands.get_entity(render_id) else {
            continue;
        };

        render_entity.despawn();
    }
}
