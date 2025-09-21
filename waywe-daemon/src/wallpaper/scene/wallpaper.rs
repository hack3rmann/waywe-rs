use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{
        gpu::Wgpu,
        wayland::{MonitorId, Wayland},
    },
    wallpaper::scene::{
        DummyWorld, FrameRateSetting, MainWorld, Monitor, PostExtract, PostStartup, Startup, Time,
        Update, WallpaperConfig, WallpaperFlags, guess_framerate,
        plugin::PluginGroup,
        render::{
            EntityMap, MonitorPlugged, QueuedPlugEvents, Render, RenderGpu, SceneExtract,
            SceneRenderStage,
        },
        subapp::EcsApp,
        time::update_time,
    },
};
use bevy_ecs::prelude::*;
use derive_more::{Deref, DerefMut};
use std::{mem, sync::Arc};

#[derive(Debug)]
pub struct Wallpaper {
    pub main: EcsApp,
    pub render: EcsApp,
}

impl Wallpaper {
    fn make_render(gpu: Arc<Wgpu>, wayland: &Wayland) -> EcsApp {
        let mut render = EcsApp::default();

        let mut render_schedule = Schedule::new(Render);
        render_schedule.configure_sets(
            (
                SceneRenderStage::Update,
                SceneRenderStage::PreRender,
                SceneRenderStage::Render,
                SceneRenderStage::Present,
            )
                .chain(),
        );
        render_schedule.add_systems(update_time.in_set(SceneRenderStage::Update));

        let queued_plug_events = wayland
            .client_state
            .monitors
            .read()
            .unwrap()
            .keys()
            .map(|&id| MonitorPlugged { id })
            .collect();

        render
            .init_resource::<Time>()
            .init_resource::<EntityMap>()
            .insert_resource(RenderGpu(gpu))
            .add_schedule(Schedule::new(SceneExtract))
            .add_schedule(render_schedule)
            .insert_resource(QueuedPlugEvents(queued_plug_events));

        render
    }

    pub fn make_main(monitor: Monitor, config: WallpaperConfig) -> EcsApp {
        let mut main = EcsApp::default();
        let mut flags = WallpaperFlags::empty();

        if !matches!(config.framerate, FrameRateSetting::NoUpdate) {
            let mut update = Schedule::new(Update);
            update.add_systems(update_time);
            main.add_schedule(update);
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

    pub fn new(gpu: Arc<Wgpu>, wayland: &Wayland, monitor_id: MonitorId) -> Self {
        let monitor_size = wayland.client_state.monitor_size(monitor_id).unwrap();
        let monitor = Monitor {
            id: monitor_id,
            size: monitor_size,
        };

        Self {
            render: Self::make_render(gpu, wayland),
            // TODO(hack3rmann): allow custom config
            main: Self::make_main(monitor, WallpaperConfig::default()),
        }
    }

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

    pub fn add_plugins(&mut self, plugins: impl PluginGroup) -> &mut Self {
        plugins.add_to_app(self);
        self
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct PreparedWallpaper(pub Wallpaper);

impl PreparedWallpaper {
    pub fn prepare(mut wallpaper: Wallpaper) -> Self {
        wallpaper.main.world.run_schedule(Startup);
        Self(wallpaper)
    }

    pub fn frame(&mut self) -> Result<FrameInfo, FrameError> {
        if let Some(QueuedPlugEvents(events)) =
            self.render.world.remove_resource::<QueuedPlugEvents>()
        {
            for event in events {
                self.render.world.trigger(event);
            }
        }

        // TODO(hack3rmann): parallelize
        self.main.world.run_schedule(Update);
        self.run_extract();
        self.render.world.run_schedule(Render);

        let frame_info = match self.main.resource::<FrameRateSetting>() {
            FrameRateSetting::TargetFrameDuration(duration) => FrameInfo {
                target_frame_time: Some(*duration),
            },
            FrameRateSetting::NoUpdate => FrameInfo {
                target_frame_time: None,
            },
            FrameRateSetting::GuessFromScene => FrameInfo::new_60_fps(),
        };

        Ok(frame_info)
    }
}

pub struct WallpaperBuildConfig {
    pub monitor_id: MonitorId,
}

pub trait WallpaperBuilder {
    fn build(self, wallpaper: &mut Wallpaper);

    fn frame_rate(&self) -> FrameRateSetting {
        FrameRateSetting::CAP_TO_60_FPS
    }
}
