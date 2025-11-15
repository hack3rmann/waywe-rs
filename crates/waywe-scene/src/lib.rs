//! Entity-Component-System (ECS) framework for creating dynamic wallpapers.
//!
//! This module provides a complete ECS-based scene system for creating and rendering
//! dynamic wallpapers. It's built on top of Bevy's ECS architecture and provides
//! specialized components and systems for wallpaper rendering.
//!
//! # Architecture Overview
//!
//! The scene system consists of two main ECS worlds:
//!
//! 1. **Main World**: Runs logic updates on a separate thread
//! 2. **Render World**: Handles GPU rendering and presentation
//!
//! These worlds are synchronized through an extraction process that transfers
//! relevant data from the main world to the render world each frame.
//!
//! # Core Components
//!
//! The scene system provides several core components for building wallpapers:
//!
//! - [`Transform`](transform::Transform) and [`GlobalTransform`](transform::GlobalTransform): Position, scale, and rotate entities
//! - [`Image`](image::Image), [`ImageMaterial`](image::ImageMaterial): Load and display images
//! - [`Mesh`](mesh::Mesh): Define geometry for rendering
//! - [`Video`]: Play video content as textures
//! - [`Material`](material::Material): Define how surfaces are rendered
//!
//! # Creating a Wallpaper
//!
//! To create a wallpaper, you typically:
//!
//! 1. Create a [`Wallpaper`](wallpaper::Wallpaper) instance with [`Wallpaper::new`](wallpaper::Wallpaper::new)
//! 2. Add plugins for the functionality you need using [`Wallpaper::add_plugins`](wallpaper::Wallpaper::add_plugins)
//! 3. Spawn entities with components in the main world
//! 4. Run the frame loop with [`PreparedWallpaper::frame`](wallpaper::PreparedWallpaper::frame)
//!
//! # Custom Plugins
//!
//! You can create custom functionality by implementing the [`Plugin`](plugin::Plugin) trait:
//!
//! ```rust
//! use waywe_scene::{
//!     Update,
//!     plugin::Plugin,
//!     wallpaper::Wallpaper,
//!     ecs::prelude::Res,
//!     time::Time,
//! };
//!
//! struct MyCustomPlugin;
//!
//! impl Plugin for MyCustomPlugin {
//!     fn build(&self, wallpaper: &mut Wallpaper) {
//!         // Add your systems and resources here
//!         wallpaper.main.add_systems(Update, my_system);
//!     }
//! }
//!
//! fn my_system(time: Res<Time>) {
//!     println!("Time: {}", time.elapsed.as_secs_f32());
//! }
//! ```
//!
//! # Asset Management
//!
//! The scene system includes a robust asset management system:
//!
//! - [`Assets`]: Store and manage assets in the main world
//! - [`RenderAssets`](assets::RenderAssets): GPU-ready versions of assets in the render world
//! - [`AssetHandle`](assets::AssetHandle): Type-safe references to assets
//!
//! Assets are automatically extracted from the main world to the render world
//! during the frame loop.
//!
//! # Scheduling
//!
//! The scene system uses several schedules for different phases:
//!
//! - [`Startup`]: One-time initialization systems
//! - [`Update`]: Main logic update systems
//! - [`SceneExtract`](render::SceneExtract): Extract data from main to render world
//! - [`Render`](render::Render): GPU rendering systems
//!
//! Systems can be added to specific schedules to control when they run.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod asset_server;
pub mod assets;
pub mod clear_screen;
pub mod cursor;
pub mod extract;
pub mod image;
pub mod material;
pub mod mesh;
pub mod plugin;
pub mod render;
pub mod sprite;
pub mod subapp;
pub mod time;
pub mod transform;
pub mod video;
pub mod wallpaper;

use crate::{assets::Assets, time::Time, video::Video};
use bitflags::bitflags;
use derive_more::{Deref, DerefMut};
use glam::UVec2;
use std::time::Duration;
use waywe_ecs::{prelude::*, schedule::ScheduleLabel};
use waywe_runtime::{frame::FrameInfo, wayland::MonitorId};
use waywe_uuid::TypeUuid;

pub use glam;
pub use waywe_ecs as ecs;

pub mod prelude {
    pub use crate::{
        FrameRateSetting, Monitor, Startup, Update,
        asset_server::{AssetHandle, AssetServer},
        assets::Assets,
        cursor::Cursor,
        image::{Image, ImageMaterial},
        mesh::{Mesh, Mesh3d, MeshMaterial, Vertex},
        plugin::DefaultPlugins,
        sprite::Sprite,
        time::Time,
        transform::Transform,
        video::{Video, VideoMaterial},
        wallpaper::{Wallpaper, WallpaperBuilder},
    };
    pub use glam::*;
    pub use waywe_ecs::prelude::*;
}

/// Frame rate configuration for the wallpaper.
///
/// Controls how frequently the wallpaper updates and renders.
#[derive(Clone, Copy, Resource, TypeUuid, Debug, PartialEq)]
#[uuid = "23cc01a5-4dce-408e-97bd-370068188598"]
pub enum FrameRateSetting {
    /// Target a specific frame duration (e.g., 16.67ms for 60 FPS).
    TargetFrameDuration(Duration),
    /// Disable updates entirely (static wallpaper).
    NoUpdate,
    /// Automatically determine frame rate based on scene content.
    GuessFromScene,
}

impl FrameRateSetting {
    /// Cap frame rate to 60 FPS.
    pub const CAP_TO_60_FPS: Self = Self::TargetFrameDuration(FrameInfo::MAX_FPS);
}

impl Default for FrameRateSetting {
    fn default() -> Self {
        Self::CAP_TO_60_FPS
    }
}

/// Schedule label for the main update loop.
#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreUpdate;

/// Schedule label for the main update loop.
#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Update;

/// Schedule label for the main update loop.
#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PostUpdate;

/// Schedule label for one-time startup systems.
#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Startup;

/// Schedule label for systems that run after startup.
#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PostStartup;

/// Schedule label for systems that run after extraction.
#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PostExtract;

/// Wrapper around the main ECS world.
///
/// This resource holds the main world that runs logic updates.
#[derive(Resource, TypeUuid, Deref, DerefMut)]
#[uuid = "404f0fbf-1987-4fe0-a031-979f7b108deb"]
pub struct MainWorld(pub World);

/// Wrapper around a temporary world used during extraction.
///
/// This resource holds a temporary world used to store the main world
/// during the extraction process.
#[derive(Resource, TypeUuid, Default)]
#[uuid = "625a064a-e500-422c-a54f-3a2b91c2ef38"]
pub struct DummyWorld(pub World);

/// Information about the monitor this wallpaper is rendering to.
#[derive(Resource, TypeUuid, Clone, Copy)]
#[uuid = "3c383d8f-51c5-4f3e-b23c-7bfb58641d38"]
pub struct Monitor {
    /// Unique identifier for the monitor.
    pub id: MonitorId,
    /// Size of the monitor in pixels.
    pub size: UVec2,
}

impl Monitor {
    /// Calculate the aspect ratio of the monitor (height/width).
    pub const fn aspect_ratio(self) -> f32 {
        self.size.y as f32 / self.size.x as f32
    }
}

bitflags! {
    /// Flags controlling wallpaper behavior.
    #[derive(Clone, Copy, Debug, PartialEq, Default, Eq, PartialOrd, Ord, Hash, Resource, TypeUuid)]
    #[uuid = "b2985935-0c27-4150-b9af-d5a333aa7ca6"]
    pub struct WallpaperFlags: u32 {
        /// Startup systems have completed.
        const STARTUP_DONE = 1;
        /// Updates are disabled.
        const NO_UPDATE = 2;
    }
}

/// Configuration for a wallpaper instance.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WallpaperConfig {
    /// Frame rate settings for this wallpaper.
    pub framerate: FrameRateSetting,
}

/// System that automatically determines the appropriate frame rate based on video assets.
///
/// This system runs during [`PostStartup`] and sets the frame rate to match the
/// fastest video in the scene when [`FrameRateSetting::GuessFromScene`] is used.
pub fn guess_framerate(videos: Res<Assets<Video>>, mut setting: ResMut<FrameRateSetting>) {
    if !matches!(&*setting, FrameRateSetting::GuessFromScene) {
        return;
    }

    let min_duration = videos
        .iter()
        .map(|(_, video)| video.frame_time_fallback)
        .min()
        .unwrap_or(FrameInfo::MAX_FPS);

    *setting = FrameRateSetting::TargetFrameDuration(min_duration);
}
