//! Static image wallpaper implementation.
//!
//! This module provides an implementation for displaying a static image as a wallpaper.
//! The image is automatically scaled to fit the screen while maintaining its aspect ratio.
//!
//! # Usage
//!
//! ```rust
//! use waywe_daemon::wallpaper::default::image::ImageWallpaper;
//! use waywe_daemon::wallpaper::scene::wallpaper::Wallpaper;
//! use std::path::PathBuf;
//!
//! let image_wallpaper = ImageWallpaper {
//!     path: PathBuf::from("path/to/image.png"),
//! };
//!
//! // The wallpaper will be built with:
//! // - No update schedule (static wallpaper)
//! // - DefaultPlugins for basic functionality
//! // - A quad mesh with the image as a texture
//! ```

use bevy_ecs::prelude::*;
use derive_more::Deref;
use glam::{Vec2, Vec3};
use std::path::PathBuf;
use waywe_scene::{
    FrameRateSetting, Monitor, Startup,
    assets::Assets,
    image::{Image, ImageMaterial},
    mesh::{Mesh, Mesh3d, MeshMaterial},
    plugin::DefaultPlugins,
    transform::Transform,
    wallpaper::{Wallpaper, WallpaperBuilder},
};

/// A wallpaper that displays a static image.
///
/// This wallpaper implementation loads an image from a file path and displays
/// it as a fullscreen wallpaper. The image is automatically scaled to fit
/// the screen while maintaining its aspect ratio.
pub struct ImageWallpaper {
    /// Path to the image file to display.
    pub path: PathBuf,
}

/// Resource that holds the image path during initialization.
#[derive(Resource, Deref)]
pub struct ImagePath(pub PathBuf);

impl WallpaperBuilder for ImageWallpaper {
    /// Build the image wallpaper by setting up the scene.
    ///
    /// This adds the default plugins, inserts the image path as a resource,
    /// and adds the setup system to the startup schedule.
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);

        wallpaper
            .main
            .insert_resource(FrameRateSetting::NoUpdate)
            .insert_resource(ImagePath(self.path))
            .add_systems(Startup, setup);
    }
}

/// System that sets up the image wallpaper scene.
///
/// This system loads the image, creates a mesh to display it on, and spawns
/// an entity with the appropriate components to render the image.
pub fn setup(
    mut commands: Commands,
    path: Res<ImagePath>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ImageMaterial>>,
    monitor: Res<Monitor>,
) {
    let image = ::image::ImageReader::open(&**path)
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();

    // Stretch the image to cover full screen
    let monitor_aspect_ratio = monitor.aspect_ratio();
    let aspect_ratio = image.height() as f32 / image.width() as f32;

    // FIXME(hack3rmann): scaling issue with wallhaven-eyw5qw
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
    let image = images.add(Image { image });
    let material = materials.add(ImageMaterial { image });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial(material),
        Transform::default().scaled_by(scale),
    ));
}
