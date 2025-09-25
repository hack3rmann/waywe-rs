//! Main wallpaper management and rendering.
//!
//! This module provides the core [`Wallpaper`] struct that manages the
//! two ECS worlds (main and render) and coordinates the rendering loop.
//!
//! # Architecture
//!
//! The wallpaper system uses a dual-world architecture:
//!
//! 1. **Main World**: Runs logic updates on a separate thread
//! 2. **Render World**: Handles GPU rendering and presentation
//!
//! Each frame, the main world updates logic, then data is extracted to
//! the render world, which then performs rendering.
//!
//! # Usage
//!
//! ```rust
//! use waywe_daemon::wallpaper::scene::{
//!     wallpaper::Wallpaper,
//!     plugin::DefaultPlugins,
//! };
//!
//! // Create a new wallpaper
//! // let mut wallpaper = Wallpaper::new(gpu, wayland, monitor_id);
//!
//! // Add plugins for functionality
//! // wallpaper.add_plugins(DefaultPlugins);
//!
//! // Prepare for rendering
//! // let mut prepared = PreparedWallpaper::prepare(wallpaper);
//!
//! // Run the frame loop
//! // loop {
//! //     prepared.frame()?;
//! // }
//! ```

use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{
        gpu::Wgpu,
        wayland::{MonitorId, Wayland},
    },
    wallpaper::scene::{
        DummyWorld, FrameRateSetting, MainWorld, Monitor, PostExtract, PostStartup, PostUpdate,
        PreUpdate, Startup, Time, Update, WallpaperConfig, WallpaperFlags, guess_framerate,
        mesh::{CommandEncoder, SurfaceView},
        plugin::PluginGroup,
        render::{EntityMap, Render, RenderGpu, RenderStage, SceneExtract},
        subapp::EcsApp,
        time::update_time,
    },
};
use bevy_ecs::prelude::*;
use std::{mem, sync::Arc, thread};

/// Main wallpaper controller.
///
/// Manages the dual ECS world architecture and coordinates rendering.
pub struct Wallpaper {
    /// Main world for logic updates.
    pub main: EcsApp,
    /// Render world for GPU operations.
    pub render: EcsApp,
}

impl Wallpaper {
    /// Create the render world with appropriate systems and resources.
    fn make_render(gpu: Arc<Wgpu>, monitor: Monitor) -> EcsApp {
        let mut render = EcsApp::default();

        let mut render_schedule = Schedule::new(Render);
        render_schedule.configure_sets(
            (
                RenderStage::Update,
                RenderStage::PrepareRender,
                RenderStage::ClearPass,
                RenderStage::Render,
                RenderStage::Finish,
            )
                .chain(),
        );
        render_schedule.add_systems(update_time.in_set(RenderStage::Update));

        render
            .init_resource::<Time>()
            .init_resource::<EntityMap>()
            .init_resource::<FrameRateSetting>()
            .insert_resource(RenderGpu(gpu))
            .insert_resource(monitor)
            .add_schedule(Schedule::new(SceneExtract))
            .add_schedule(render_schedule);

        render
    }

    /// Create the main world with appropriate systems and resources.
    pub fn make_main(monitor: Monitor, config: WallpaperConfig) -> EcsApp {
        let mut main = EcsApp::default();
        let mut flags = WallpaperFlags::empty();

        if !matches!(config.framerate, FrameRateSetting::NoUpdate) {
            let mut update = Schedule::new(Update);
            update.add_systems(update_time);
            main.add_schedule(update)
                .add_schedule(Schedule::new(PreUpdate))
                .add_schedule(Schedule::new(PostUpdate));
        } else {
            flags |= WallpaperFlags::NO_UPDATE;
        }

        main.insert_resource(config.framerate)
            .insert_resource(monitor)
            .insert_resource(flags)
            .init_resource::<Time>()
            .init_resource::<DummyWorld>()
            .add_systems(PostStartup, guess_framerate)
            .add_schedule(Schedule::new(Startup))
            .add_schedule(Schedule::new(PostExtract));

        main
    }

    /// Create a new wallpaper for a specific monitor.
    pub fn new(gpu: Arc<Wgpu>, wayland: &Wayland, monitor_id: MonitorId) -> Self {
        let monitor_size = wayland.client_state.monitor_size(monitor_id).unwrap();
        let monitor = Monitor {
            id: monitor_id,
            size: monitor_size,
        };

        Self {
            render: Self::make_render(gpu, monitor),
            // TODO(hack3rmann): allow custom config
            main: Self::make_main(monitor, WallpaperConfig::default()),
        }
    }

    /// Extract data from the main world to the render world.
    ///
    /// This is called each frame to synchronize the two worlds.
    pub fn run_extract(&mut self) {
        let DummyWorld(temp_world) = self
            .main
            .world
            .remove_resource::<DummyWorld>()
            .unwrap_or_default();
        let main_world = mem::replace(&mut self.main.world, temp_world);
        self.render.insert_resource(MainWorld(main_world));

        self.render.world.run_schedule(SceneExtract);

        let MainWorld(main_world) = self.render.world.remove_resource::<MainWorld>().unwrap();
        let temp_world = mem::replace(&mut self.main.world, main_world);
        self.main.world.insert_resource(DummyWorld(temp_world));

        self.main.world.run_schedule(PostExtract);
    }

    /// Add plugins to extend wallpaper functionality.
    pub fn add_plugins(&mut self, plugins: impl PluginGroup) -> &mut Self {
        plugins.add_to_app(self);
        self
    }
}

/// Runs render schedules. `render` should correspond to the rendering app
fn run_render(
    render: &mut EcsApp,
    surface_view: wgpu::TextureView,
    encoder: &mut wgpu::CommandEncoder,
) {
    render.insert_resource(SurfaceView(surface_view));
    // Safety: we never access the encoder between insert and remove
    render.insert_resource(unsafe { CommandEncoder::new(encoder) });

    render.world.run_schedule(Render);

    _ = render.world.remove_resource::<SurfaceView>();
    render
        .world
        .remove_resource::<CommandEncoder>()
        // Safety: remove the encoder
        .expect("world should let go the encoder for the safety");
}

/// Runs update schedules. `main` should correspond to the main app
fn run_update(main: &mut EcsApp) {
    main.world.run_schedule(PreUpdate);
    main.world.run_schedule(Update);
    main.world.run_schedule(PostUpdate);
}

/// A wallpaper that has been prepared for rendering.
///
/// This wrapper handles the frame loop and synchronization between
/// the main and render worlds.
pub struct PreparedWallpaper {
    first_time: bool,
    /// The wallpaper being managed.
    pub wallpaper: Wallpaper,
}

impl PreparedWallpaper {
    /// Prepare a wallpaper for rendering.
    ///
    /// This runs the startup schedule and prepares the wallpaper for the frame loop.
    pub fn prepare(mut wallpaper: Wallpaper) -> Self {
        wallpaper.main.world.run_schedule(Startup);

        Self {
            first_time: true,
            wallpaper,
        }
    }

    /// Run one frame of the wallpaper.
    ///
    /// This updates logic, extracts data to the render world, and renders the frame.
    pub fn frame(
        &mut self,
        surface_view: wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<FrameInfo, FrameError> {
        if self.first_time {
            run_update(&mut self.wallpaper.main);
            self.wallpaper.run_extract();
            run_render(&mut self.wallpaper.render, surface_view, encoder);

            self.first_time = false;
        } else {
            thread::scope(|s| {
                let handle = s.spawn(|| run_update(&mut self.wallpaper.main));
                run_render(&mut self.wallpaper.render, surface_view, encoder);
                handle.join().unwrap();
            });

            self.wallpaper.run_extract();
        }

        Ok(match self.wallpaper.main.resource::<FrameRateSetting>() {
            FrameRateSetting::TargetFrameDuration(duration) => FrameInfo {
                target_frame_time: Some(*duration),
            },
            FrameRateSetting::GuessFromScene => FrameInfo::new_60_fps(),
            FrameRateSetting::NoUpdate => FrameInfo {
                target_frame_time: None,
            },
        })
    }
}

/// Configuration for building a wallpaper.
pub struct WallpaperBuildConfig {
    /// The monitor ID to render to.
    pub monitor_id: MonitorId,
}

/// Trait for building custom wallpapers.
pub trait WallpaperBuilder {
    /// Build the wallpaper by adding entities, components, and systems.
    fn build(self, wallpaper: &mut Wallpaper);
}
