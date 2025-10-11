use crate::{
    event_loop::WallpaperTarget,
    wallpaper::{self, transition::RunningWallpapers},
};
use for_sure::prelude::*;
use smallvec::{SmallVec, smallvec};
use std::{collections::btree_map::Entry, path::PathBuf, sync::Arc};
use tracing::{debug, error};
use waywe_ipc::{
    WallpaperType,
    config::Config,
    profile::{Monitor, SetupProfile},
};
use waywe_runtime::{
    Runtime, RuntimeFeatures,
    app::App,
    effects::convolve::ConvolveConfig,
    event::{EventHandler, Handle, TryReplicate},
    frame::{FrameError, FrameInfo},
    wayland::{MonitorId, MonitorMap, WaylandEvent},
};
use waywe_scene::{cursor::CursorMoved, wallpaper::PreparedWallpaper};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallpaperState {
    #[default]
    Running,
    Paused,
}

impl WallpaperState {
    pub const fn inverted(self) -> Self {
        match self {
            Self::Running => Self::Paused,
            Self::Paused => Self::Running,
        }
    }

    pub const fn is_paused(self) -> bool {
        matches!(self, Self::Paused)
    }

    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }
}

#[derive(Default)]
pub struct WallpaperApp {
    pub wallpapers: MonitorMap<RunningWallpapers>,
    pub wallpaper_states: MonitorMap<WallpaperState>,
    pub config: Config,
    pub do_force_frame: bool,
}

impl WallpaperApp {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub fn set_wallpaper(
        &mut self,
        runtime: &Runtime,
        wallpaper: PreparedWallpaper,
        monitor_id: MonitorId,
    ) {
        match self.wallpapers.entry(monitor_id) {
            Entry::Vacant(entry) => {
                let size = {
                    let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                    monitors[&monitor_id].size.unwrap()
                };
                let mut wallpapers =
                    RunningWallpapers::new(monitor_id, size, self.config.animation.clone());

                wallpapers.add_effect(ConvolveConfig {
                    kernel: Arc::new([0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0]),
                });

                wallpapers.enqueue_wallpaper(&runtime.wgpu, wallpaper);
                entry.insert(wallpapers);
            }
            Entry::Occupied(mut occupied_entry) => occupied_entry
                .get_mut()
                .enqueue_wallpaper(&runtime.wgpu, wallpaper),
        }

        self.wallpaper_states
            .insert(monitor_id, WallpaperState::Running);
    }
}

pub struct WallpaperPreparedEvent {
    pub wallpaper: PreparedWallpaper,
    pub monitor_id: MonitorId,
}

impl TryReplicate for WallpaperPreparedEvent {}

#[derive(Clone)]
pub struct NewWallpaperEvent {
    pub path: PathBuf,
    pub ty: WallpaperType,
    pub target: WallpaperTarget,
}

#[derive(Clone)]
pub struct WallpaperPauseEvent {
    pub target: WallpaperTarget,
}

impl App for WallpaperApp {
    fn populate_handler(&mut self, handler: &mut EventHandler<Self>) {
        handler
            .add_event::<WaylandEvent>()
            .add_event::<NewWallpaperEvent>()
            .add_event::<WallpaperPreparedEvent>()
            .add_event::<WallpaperPauseEvent>();
    }

    async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        if Almost::is_nil(&runtime.wgpu) {
            return Err(FrameError::NoWorkToDo);
        }

        // FIXME(hack3rmann): multiple monitors
        let mut result = Err(FrameError::NoWorkToDo);

        for (&monitor_id, wallpapers) in self.wallpapers.iter_mut() {
            if let Some(&state) = self.wallpaper_states.get(&monitor_id)
                && state.is_paused()
            {
                continue;
            }

            let surface = {
                let surfaces = runtime.wgpu.surfaces.read().unwrap();
                surfaces[&monitor_id].surface.get_current_texture().unwrap()
            };

            let mut encoder = runtime
                .wgpu
                .device
                .create_command_encoder(&Default::default());

            result = wallpapers.render(&runtime.wgpu, &surface.texture, &mut encoder);

            runtime.wgpu.queue.submit([encoder.finish()]);
            surface.present();
        }

        if let Err(FrameError::NoWorkToDo) = &result {
            runtime.control_flow.idle();
        }

        self.do_force_frame = false;

        result
    }
}

impl Handle<WallpaperPauseEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WallpaperPauseEvent) {
        let WallpaperPauseEvent { target } = event;

        let monitor_ids: SmallVec<[MonitorId; 4]> = match target {
            WallpaperTarget::ForAll => {
                let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                monitors.keys().copied().collect()
            }
            WallpaperTarget::ForMonitor(id) => smallvec![id],
        };

        for monitor_id in monitor_ids {
            let Some(state) = self.wallpaper_states.get_mut(&monitor_id) else {
                continue;
            };

            *state = state.inverted();
        }
    }
}

impl Handle<WallpaperPreparedEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WallpaperPreparedEvent) {
        let WallpaperPreparedEvent {
            wallpaper,
            monitor_id,
        } = event;

        runtime.control_flow.busy();
        self.set_wallpaper(runtime, wallpaper, monitor_id);
    }
}

impl Handle<WaylandEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WaylandEvent) {
        match event {
            WaylandEvent::ResizeRequested { monitor_id, size } => {
                if Almost::is_value(&runtime.wgpu) {
                    runtime.wgpu.resize_surface(monitor_id, size);
                }

                self.do_force_frame = true;
            }
            WaylandEvent::MonitorPlugged { id: monitor_id } => {
                if Almost::is_value(&runtime.wgpu) {
                    runtime.wgpu.register_surface(&runtime.wayland, monitor_id);
                }

                let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                let monitor = &monitors[&monitor_id];
                let monitor_name = Arc::clone(monitor.name.as_ref().unwrap());

                debug!(?monitor_id, ?monitor_name, "new monitor detected");

                if let Ok(mut profile) = SetupProfile::read()
                    && let Some(info) = profile.monitors.remove(&monitor_name)
                {
                    let event = NewWallpaperEvent {
                        path: info.path,
                        ty: info.wallpaper_type,
                        target: WallpaperTarget::ForMonitor(monitor_id),
                    };

                    runtime.task_pool.emitter.emit(event).unwrap();
                }

                runtime.control_flow.busy();
            }
            WaylandEvent::MonitorUnplugged { id: monitor_id } => {
                debug!(?monitor_id, "unplugged a monitor");

                _ = self.wallpapers.remove(&monitor_id);
                _ = self.wallpaper_states.remove(&monitor_id);

                runtime.wgpu.unregister_surface(monitor_id);
            }
            WaylandEvent::CursorMoved { position } => {
                let event = CursorMoved { position };

                for wallpaper in self
                    .wallpapers
                    .values_mut()
                    .flat_map(RunningWallpapers::wallpapers_mut)
                {
                    wallpaper.wallpaper.wallpaper.main.world.trigger(event);
                }
            }
        }
    }
}

impl Handle<NewWallpaperEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: NewWallpaperEvent) {
        let NewWallpaperEvent { path, ty, target } = event;

        // FIXME(hack3rmann): remove runtime features
        runtime.enable(RuntimeFeatures::GPU).await;

        let monitor_ids: SmallVec<[MonitorId; 4]> = match target {
            WallpaperTarget::ForAll => {
                let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                monitors.keys().copied().collect()
            }
            WallpaperTarget::ForMonitor(id) => smallvec![id],
        };

        for monitor_id in monitor_ids {
            let path = path.clone();
            let gpu = Arc::clone(&runtime.wgpu);
            let wayland = Arc::clone(&runtime.wayland);

            let monitors = runtime.wayland.client_state.monitors.read().unwrap();
            let monitor = &monitors[&monitor_id];
            let monitor_name = Arc::clone(monitor.name.as_ref().unwrap());
            let monitor_profile = Monitor {
                wallpaper_type: ty,
                path: path.clone(),
            };

            if let Err(error) = SetupProfile::default()
                .with(monitor_name, monitor_profile)
                .store()
            {
                error!(?error, "failed to save setup profile");
            }

            runtime.task_pool.spawn(move |mut emitter| {
                let event = WallpaperPreparedEvent {
                    wallpaper: wallpaper::create(gpu, wayland, &path, ty, monitor_id),
                    monitor_id,
                };

                emitter.emit(event).unwrap();
            });
        }
    }
}
