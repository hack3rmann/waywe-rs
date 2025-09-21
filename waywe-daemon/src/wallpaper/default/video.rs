//! Video wallpaper implementation.
//!
//! This module provides an implementation for playing a video as a wallpaper.
//! The video is automatically scaled to fit the screen while maintaining its aspect ratio.
//!
//! # Usage
//!
//! ```rust
//! use waywe_daemon::wallpaper::default::video::VideoWallpaper;
//! use waywe_daemon::wallpaper::scene::wallpaper::Wallpaper;
//! use std::path::PathBuf;
//!
//! let video_wallpaper = VideoWallpaper {
//!     path: PathBuf::from("path/to/video.mp4"),
//! };
//!
//! // The wallpaper will be built with:
//! //! - Frame rate guessed from the video content
//! //! - DefaultPlugins for basic functionality
//! //! - A quad mesh with the video as a texture
//! ```

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
#[derive(Resource, Deref)]
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
            .add_systems(Startup, setup);
    }

    /// Get the frame rate setting for this wallpaper.
    ///
    /// Video wallpapers use [`FrameRateSetting::GuessFromScene`] to automatically
    /// determine the appropriate frame rate based on the video content.
    fn frame_rate(&self) -> FrameRateSetting {
        FrameRateSetting::GuessFromScene
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
