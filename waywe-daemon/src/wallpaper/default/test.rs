//! Test wallpaper implementation with animations.
//!
//! This module provides a test wallpaper with multiple meshes, videos, and animations
//! for testing the scene system. It includes both static images and videos with
//! rotating and moving elements.
//!
//! # Usage
//!
//! ```rust
//! use waywe_daemon::wallpaper::default::test::SceneTestWallpaper;
//! use waywe_daemon::wallpaper::scene::wallpaper::Wallpaper;
//!
//! let test_wallpaper = SceneTestWallpaper;
//!
//! // The wallpaper will be built with:
//! //! - DefaultPlugins for basic functionality
//! //! - Multiple meshes with images and videos
//! //! - Animation systems that respond to time and cursor position
//! ```

use crate::wallpaper::scene::{
    Monitor, Startup, Update,
    asset_server::AssetHandle,
    assets::Assets,
    cursor::Cursor,
    image::{Image, ImageMaterial},
    mesh::{Mesh, Mesh3d, MeshMaterial, Vertex},
    plugin::DefaultPlugins,
    time::Time,
    transform::Transform,
    video::{Video, VideoMaterial},
    wallpaper::{Wallpaper, WallpaperBuilder},
};
use bevy_ecs::prelude::*;
use glam::{Quat, Vec2, Vec3};
use smallvec::smallvec;

/// A test wallpaper with multiple meshes and animations.
///
/// This wallpaper implementation is designed for testing the scene system.
/// It includes multiple meshes with both images and videos, and animation
/// systems that respond to time and cursor position.
pub struct SceneTestWallpaper;

impl WallpaperBuilder for SceneTestWallpaper {
    /// Build the test wallpaper by setting up the scene.
    ///
    /// This adds the default plugins, initializes test assets, and adds
    /// systems for startup and updates.
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);

        wallpaper
            .main
            .add_systems(Update, rotate_meshes)
            .add_systems(Startup, (spawn_mesh, spawn_videos))
            .init_resource::<TestAssets>();
    }
}

/// Component that controls the time scale for animation.
#[derive(Component)]
pub struct TimeScale(pub f32);

/// Resource that holds pre-loaded test assets.
#[derive(Resource)]
pub struct TestAssets {
    /// A quad mesh for rendering.
    pub quad_mesh: AssetHandle<Mesh>,
    /// A triangle mesh for rendering.
    pub triangle_mesh: AssetHandle<Mesh>,
    /// A test image asset.
    pub image: AssetHandle<Image>,
    /// The aspect ratio of the test image.
    pub image_aspect_ratio: f32,
    /// A material for the test image.
    pub image_material: AssetHandle<ImageMaterial>,
    /// The first test video asset.
    pub video1: AssetHandle<Video>,
    /// A material for the first test video.
    pub video1_material: AssetHandle<VideoMaterial>,
    /// The aspect ratio of the first test video.
    pub video1_aspect_ratio: f32,
    /// The second test video asset.
    pub video2: AssetHandle<Video>,
    /// A material for the second test video.
    pub video2_material: AssetHandle<VideoMaterial>,
    /// The aspect ratio of the second test video.
    pub video2_aspect_ratio: f32,
}

impl FromWorld for TestAssets {
    /// Load test assets from the world.
    ///
    /// This creates meshes, loads images and videos, and sets up materials
    /// for use in the test wallpaper.
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        let quad_mesh = meshes.add(Mesh::rect(Vec2::ONE));
        let triangle_mesh = meshes.add(Mesh {
            vertices: smallvec![
                Vertex(Vec3::new(-0.5, -0.5, 0.0)),
                Vertex(Vec3::new(0.5, -0.5, 0.0)),
                Vertex(Vec3::new(0.0, 0.5, 0.0)),
            ],
        });

        let mut videos = world.resource_mut::<Assets<Video>>();

        let video1 = Video::new("target/test-video.mp4").unwrap();
        let video1_aspect_ratio = video1.frame_aspect_ratio();
        let video1_handle = videos.add(video1);

        let video2 = Video::new("target/test-video2.mp4").unwrap();
        let video2_aspect_ratio = video2.frame_aspect_ratio();
        let video2_handle = videos.add(video2);

        let mut images = world.resource_mut::<Assets<Image>>();

        // FIXME(hack3rmann): use local image
        const PATH: &str = "target/test-image.png";
        let image = ::image::ImageReader::open(PATH)
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();

        let image_aspect_ratio = image.height() as f32 / image.width() as f32;
        let image = images.add(Image { image });

        let mut image_materials = world.resource_mut::<Assets<ImageMaterial>>();
        let image_material = image_materials.add(ImageMaterial {
            image: image.clone(),
        });

        let mut video_materials = world.resource_mut::<Assets<VideoMaterial>>();
        let video1_material = video_materials.add(VideoMaterial {
            video: video1_handle.clone(),
        });
        let video2_material = video_materials.add(VideoMaterial {
            video: video2_handle.clone(),
        });

        Self {
            quad_mesh,
            triangle_mesh,
            image,
            image_aspect_ratio,
            image_material,
            video1: video1_handle,
            video1_aspect_ratio,
            video1_material,
            video2: video2_handle,
            video2_aspect_ratio,
            video2_material,
        }
    }
}

/// System that spawns video entities.
///
/// This system creates entities for playing videos with appropriate scaling
/// and time scaling factors for animation.
pub fn spawn_videos(mut commands: Commands, assets: Res<TestAssets>) {
    const SCALE: f32 = 0.6;

    commands.spawn((
        Mesh3d(assets.quad_mesh.clone()),
        MeshMaterial(assets.video1_material.clone()),
        Transform::default().scaled_by(Vec3::new(SCALE, assets.video1_aspect_ratio * SCALE, 1.0)),
        TimeScale(0.5),
    ));

    commands.spawn((
        Mesh3d(assets.quad_mesh.clone()),
        MeshMaterial(assets.video2_material.clone()),
        Transform::default().scaled_by(Vec3::new(SCALE, assets.video2_aspect_ratio * SCALE, 1.0)),
        TimeScale(0.3),
    ));

    commands.spawn((
        Mesh3d(assets.triangle_mesh.clone()),
        MeshMaterial(assets.video2_material.clone()),
        Transform::default().scaled_by(Vec3::new(SCALE, assets.video2_aspect_ratio * SCALE, 1.0)),
        TimeScale(0.8),
    ));
}

/// System that spawns image entities.
///
/// This system creates entities for displaying images with appropriate scaling
/// and time scaling factors for animation.
pub fn spawn_mesh(mut commands: Commands, assets: Res<TestAssets>) {
    const SCALE: f32 = 0.6;
    let aspect_scale = Vec3::new(SCALE, SCALE * assets.image_aspect_ratio, 1.0);

    commands.spawn((
        Mesh3d(assets.quad_mesh.clone()),
        Transform::default().scaled_by(aspect_scale),
        MeshMaterial(assets.image_material.clone()),
        TimeScale(1.0),
    ));

    commands.spawn((
        Mesh3d(assets.quad_mesh.clone()),
        Transform::default().scaled_by(aspect_scale),
        MeshMaterial(assets.image_material.clone()),
        TimeScale(std::f32::consts::FRAC_PI_2),
    ));
}

/// System that animates meshes over time.
///
/// This system rotates and moves meshes based on elapsed time and cursor position.
/// Each mesh can have a different time scale for unique animation behavior.
pub fn rotate_meshes(
    mut transforms: Query<(&mut Transform, &TimeScale), With<Mesh3d>>,
    time: Res<Time>,
    monitor: Res<Monitor>,
    cursor: Res<Cursor>,
) {
    let time = time.elapsed.as_secs_f32();

    let cursor_pos = 0.5 * cursor.position.as_vec2() / monitor.size.as_vec2() - Vec2::splat(0.5);

    for (mut transform, &TimeScale(time_scale)) in &mut transforms {
        transform.translation.x = 0.5 * (time_scale * time).cos() + 0.2 * cursor_pos.x;
        transform.translation.y = 0.5 * (time_scale * time).sin() + 0.2 * cursor_pos.y;

        transform.rotation = Quat::from_axis_angle(Vec3::X + time_scale * Vec3::Y, time);
    }
}
