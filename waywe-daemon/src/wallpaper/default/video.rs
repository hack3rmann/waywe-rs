use crate::wallpaper::scene::{
    FrameRateSetting, Monitor, Startup,
    assets::Assets,
    mesh::{Mesh, Mesh3d, MeshMaterial},
    plugin::DefaultPlugins,
    transform::Transform,
    video::{Video, VideoMaterial},
    wallpaper::{Wallpaper, WallpaperBuilder},
};
use bevy_ecs::prelude::*;
use derive_more::Deref;
use glam::{Vec2, Vec3};
use std::path::PathBuf;

pub struct VideoWallpaper {
    pub path: PathBuf,
}

#[derive(Resource, Deref)]
pub struct VideoPath(pub PathBuf);

impl WallpaperBuilder for VideoWallpaper {
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);

        wallpaper
            .main
            .insert_resource(VideoPath(self.path))
            .add_systems(Startup, setup);
    }

    fn frame_rate(&self) -> FrameRateSetting {
        FrameRateSetting::GuessFromScene
    }
}

pub fn setup(
    mut commands: Commands,
    path: Res<VideoPath>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut videos: ResMut<Assets<Video>>,
    mut materials: ResMut<Assets<VideoMaterial>>,
    monitor: Res<Monitor>,
) {
    let video = Video::new(&**path).unwrap();

    // Stretch the image to cover full screen
    let monitor_aspect_ratio = monitor.aspect_ratio();
    let aspect_ratio = video.frame_aspect_ratio();

    let scale = if aspect_ratio < 1.0 {
        Vec3::new(
            monitor_aspect_ratio / aspect_ratio,
            monitor_aspect_ratio,
            1.0,
        )
    } else {
        Vec3::new(1.0, aspect_ratio, 1.0)
    };

    let mesh = meshes.add(Mesh::rect(Vec2::ONE));
    let video = videos.add(video);
    let material = materials.add(VideoMaterial { video });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial(material),
        Transform::default().scaled_by(scale),
    ));
}
