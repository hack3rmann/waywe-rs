use crate::wallpaper::scene::{
    FrameRateSetting, Startup,
    assets::Assets,
    image::{Image, ImageMaterial},
    mesh::{Mesh, Mesh3d, MeshMaterial},
    plugin::DefaultPlugins,
    transform::Transform,
    wallpaper::{Wallpaper, WallpaperBuilder},
};
use bevy_ecs::prelude::*;
use derive_more::Deref;
use glam::{Vec2, Vec3};
use std::path::PathBuf;

pub struct ImageWallpaper {
    pub path: PathBuf,
}

#[derive(Resource, Deref)]
pub struct ImagePath(pub PathBuf);

impl WallpaperBuilder for ImageWallpaper {
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);

        wallpaper
            .main
            .insert_resource(ImagePath(self.path))
            .add_systems(Startup, setup);
    }

    fn frame_rate(&self) -> FrameRateSetting {
        FrameRateSetting::NoUpdate
    }
}

pub fn setup(
    mut commands: Commands,
    path: Res<ImagePath>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ImageMaterial>>,
) {
    let image = ::image::ImageReader::open(&**path)
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();

    // TODO(hack3rmann): scale with respect to monitor's aspect ratio
    let aspect_ratio = image.height() as f32 / image.width() as f32;

    let mesh = meshes.add(Mesh::rect(Vec2::ONE));
    let image = images.add(Image { image });
    let material = materials.add(ImageMaterial { image });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial(material),
        Transform::default().scaled_by(Vec3::new(1.0, aspect_ratio, 1.0)),
    ));
}
