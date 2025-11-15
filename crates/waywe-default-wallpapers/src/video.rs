//! Video wallpaper implementation.
//!
//! This module provides an implementation for playing a video as a wallpaper.
//! The video is automatically scaled to fit the screen while maintaining its aspect ratio.

use derive_more::Deref;
use std::path::PathBuf;
use waywe_ecs::{prelude::*, uuid::TypeUuid};
use waywe_scene::{
    FrameRateSetting, Monitor, Startup,
    assets::Assets,
    glam::{Vec2, Vec3},
    mesh::{Mesh, Mesh3d, MeshMaterial},
    plugin::DefaultPlugins,
    transform::Transform,
    video::{Video, VideoMaterial},
    wallpaper::{Wallpaper, WallpaperBuilder},
};

/// A wallpaper that plays a video.
///
/// This wallpaper implementation loads a video from a file path and plays
/// it as a fullscreen wallpaper. The video is automatically scaled to fit
/// the screen while maintaining its aspect ratio.
pub struct VideoWallpaper {
    /// Path to the video file to play.
    pub path: PathBuf,
}

/// Resource that holds the video path during initialization.
#[derive(Resource, Deref, TypeUuid)]
pub struct VideoPath(pub PathBuf);

impl WallpaperBuilder for VideoWallpaper {
    /// Build the video wallpaper by setting up the scene.
    ///
    /// This adds the default plugins, inserts the video path as a resource,
    /// and adds the setup system to the startup schedule.
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);

        wallpaper
            .main
            .insert_resource(VideoPath(self.path))
            .insert_resource(FrameRateSetting::GuessFromScene)
            .add_systems(Startup, setup);
    }
}

/// System that sets up the video wallpaper scene.
///
/// This system loads the video, creates a mesh to display it on, and spawns
/// an entity with the appropriate components to render the video.
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
