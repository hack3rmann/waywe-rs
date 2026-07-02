use crate::{
    event_loop::WallpaperTarget,
    wallpaper::{
        self, Wallpaper, WallpaperConfig, optimized::OptimizedWallpaper,
        package_registry::PackageRegistry, transition::RunningWallpapers,
    },
};
use glam::UVec2;
use smallvec::{SmallVec, smallvec};
use std::{
    collections::{BTreeMap, btree_map::Entry},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use tracing::{debug, error};
use waywe_ipc::{
    WallpaperType,
    command::{DaemonResponse, PauseMode},
    config::Config,
    ipc::server::{ClientId, IpcResponse},
    profile::{Monitor, SetupProfile},
};
use waywe_runtime::{
    Runtime,
    app::App,
    event::{EventHandler, Handle, PostEventActions, TryReplicate},
    frame::{FrameError, FrameInfo},
    gpu::SurfaceResult,
    wayland::{MonitorId, MonitorMap, MonitorName, WaylandEvent},
};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallpaperStateKind {
    #[default]
    Running,
    Paused {
        needs_redraw: bool,
    },
}

impl WallpaperStateKind {
    pub const fn inverted(self) -> Self {
        match self {
            Self::Running => Self::Paused { needs_redraw: true },
            Self::Paused { needs_redraw: _ } => Self::Running,
        }
    }

    pub const fn paused(self) -> Self {
        match self {
            Self::Running => Self::Paused { needs_redraw: true },
            Self::Paused { needs_redraw } => Self::Paused { needs_redraw },
        }
    }

    pub const fn resumed(self) -> Self {
        Self::Running
    }

    pub const fn altered(self, mode: PauseMode) -> Self {
        match mode {
            PauseMode::Toggle => self.inverted(),
            PauseMode::On => self.paused(),
            PauseMode::Off => self.resumed(),
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WallpaperState {
    pub kind: WallpaperStateKind,
    pub is_active: bool,
}

impl WallpaperState {
    pub const ACTIVE_RUNNING: Self = Self {
        kind: WallpaperStateKind::Running,
        is_active: true,
    };

    pub const fn needs_redraw(self) -> bool {
        if !self.is_active {
            return false;
        }

        match self.kind {
            WallpaperStateKind::Running => true,
            WallpaperStateKind::Paused { needs_redraw } => needs_redraw,
        }
    }

    pub const fn redraw_completed(mut self) -> Self {
        if let WallpaperStateKind::Paused { needs_redraw } = &mut self.kind {
            *needs_redraw = false;
        }

        self
    }
}

#[derive(Default)]
pub struct WallpaperApp {
    pub wallpapers: MonitorMap<RunningWallpapers>,
    pub wallpaper_states: BTreeMap<MonitorName, WallpaperState>,
    pub config: Config,
    pub package_registry: PackageRegistry,
    pub last_instant: Option<Instant>,
}

impl WallpaperApp {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }
}

pub struct WallpaperPreparedEvent {
    pub wallpaper: OptimizedWallpaper,
    pub monitor_id: MonitorId,
}

impl TryReplicate for WallpaperPreparedEvent {}

#[derive(Clone, Debug)]
pub struct NewWallpaperEvent {
    pub path: PathBuf,
    pub ty: WallpaperType,
    pub target: WallpaperTarget,
}

#[derive(Clone, Debug)]
pub struct WallpaperPreviewEvent {
    pub path: PathBuf,
    pub ty: WallpaperType,
    pub size: UVec2,
    pub sender_id: ClientId,
}

#[derive(Clone, Debug)]
pub struct WallpaperPauseEvent {
    pub target: WallpaperTarget,
    pub mode: PauseMode,
}

impl App for WallpaperApp {
    fn populate_handler(&mut self, handler: &mut EventHandler<Self>) {
        handler
            .add_event::<WaylandEvent>()
            .add_event::<NewWallpaperEvent>()
            .add_event::<WallpaperPreparedEvent>()
            .add_event::<WallpaperPauseEvent>()
            .add_event::<WallpaperPreviewEvent>();
    }

    async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        let mut results: SmallVec<[_; 4]> = smallvec![];

        let time_delta = self
            .last_instant
            .as_ref()
            .map(Instant::elapsed)
            .unwrap_or_default();
        self.last_instant = Some(Instant::now());

        for (&monitor_id, wallpapers) in self.wallpapers.iter_mut() {
            let monitor_name = {
                let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                monitors[&monitor_id].name.clone()
            };

            if let Some(state) = self.wallpaper_states.get(&monitor_name)
                && !state.needs_redraw()
            {
                results.push(Err(FrameError::NoWorkToDo));
                continue;
            }

            let (needs_reconfigure, surface) = match runtime
                .wgpu
                .get_current_surface(&runtime.wayland, monitor_id)
            {
                SurfaceResult::Ok(texture) => (false, texture),
                SurfaceResult::Reconfigure(texture) => (true, texture),
                SurfaceResult::Skip => {
                    results.push(Err(FrameError::NoWorkToDo));
                    continue;
                }
                SurfaceResult::Err => {
                    tracing::error!(?monitor_id, "failed to get_current_texture on surface");
                    results.push(Err(FrameError::NoWorkToDo));
                    continue;
                }
            };

            let mut encoder = runtime
                .wgpu
                .device
                .create_command_encoder(&Default::default());

            wallpapers.advance_time(time_delta);
            let result = wallpapers.render(&runtime.wgpu, &surface.texture, &mut encoder);

            results.push(result);

            runtime.wgpu.queue.submit([encoder.finish()]);
            runtime.wgpu.queue.present(surface);

            if let Some(state) = self.wallpaper_states.get_mut(&monitor_name) {
                *state = state.redraw_completed();
            }

            if needs_reconfigure {
                runtime.wgpu.reconfigure_surface(monitor_id);
            }
        }

        let no_work = results
            .iter()
            .all(|res| *res == Err(FrameError::NoWorkToDo));

        let going_sleep = results.iter().all(|res| {
            matches!(
                res,
                Err(FrameError::NoWorkToDo)
                    | Ok(FrameInfo {
                        target_frame_time: None
                    })
            )
        });

        if going_sleep {
            self.last_instant = None;
        }

        if no_work {
            return Err(FrameError::NoWorkToDo);
        }

        let results = results.iter().flatten().cloned();
        let frame_info = results
            .reduce(|acc, elem| acc.min_or_60_fps(elem))
            .expect("at least one Ok FrameInfo");

        Ok(frame_info)
    }
}

impl Handle<WallpaperPauseEvent> for WallpaperApp {
    async fn handle(
        &mut self,
        runtime: &mut Runtime,
        event: WallpaperPauseEvent,
    ) -> PostEventActions {
        let WallpaperPauseEvent { target, mode } = event;

        match target {
            WallpaperTarget::ForAll => {
                for state in self.wallpaper_states.values_mut() {
                    state.kind = state.kind.altered(mode);
                }
            }
            WallpaperTarget::ForMonitor(id) => {
                let name = runtime.wayland.client_state.monitor_name(id).unwrap();
                let state = self.wallpaper_states.get_mut(&name).unwrap();

                state.kind = state.kind.altered(mode);
            }
        }

        PostEventActions::REDRAW
    }
}

impl Handle<WallpaperPreparedEvent> for WallpaperApp {
    async fn handle(
        &mut self,
        runtime: &mut Runtime,
        event: WallpaperPreparedEvent,
    ) -> PostEventActions {
        // FIXME(hack3rmann): monitor unplug + plug can occur while preparing a wallpaper,
        // which can cause monitor reindexing. We should use monitor_name here instead
        let WallpaperPreparedEvent {
            mut wallpaper,
            monitor_id,
        } = event;

        // NOTE(hack3rmann): wallpaper may be prepared after monitor is disconnected
        let Some(config) = runtime.wallpaper_config(monitor_id) else {
            return PostEventActions::empty();
        };

        // NOTE(hack3rmann): monitor could be unplugged when this event arrives
        let Some(run) = self.wallpapers.get_mut(&monitor_id) else {
            return PostEventActions::empty();
        };

        // NOTE(hack3rmann): we may get outdated wallpaper configuration if resize event comes
        // before WallpaperPreparedEvent and after NewWallpaperEvent
        wallpaper.configure(&runtime.wgpu, config);
        run.enqueue_wallpaper(&runtime.wgpu, wallpaper);

        let monitor_name = {
            let monitors = runtime.wayland.client_state.monitors.read().unwrap();
            monitors[&monitor_id].name.clone()
        };

        match self.wallpaper_states.entry(monitor_name) {
            Entry::Vacant(entry) => {
                entry.insert(WallpaperState::ACTIVE_RUNNING);
            }
            Entry::Occupied(entry) => {
                let state = entry.into_mut();
                state.is_active = true;

                if let WallpaperStateKind::Paused { needs_redraw } = &mut state.kind {
                    *needs_redraw = true;
                }
            }
        }

        PostEventActions::REDRAW
    }
}

impl Handle<WaylandEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WaylandEvent) -> PostEventActions {
        match event {
            WaylandEvent::ResizeRequested { monitor_id, size } => {
                runtime.wgpu.resize_surface(monitor_id, size);

                let monitor_name = runtime
                    .wayland
                    .client_state
                    .monitor_name(monitor_id)
                    .unwrap();

                if let Some(WallpaperState {
                    kind: WallpaperStateKind::Paused { needs_redraw },
                    ..
                }) = self.wallpaper_states.get_mut(&monitor_name)
                {
                    *needs_redraw = true;
                }

                let Some(wall) = self.wallpapers.get_mut(&monitor_id) else {
                    return PostEventActions::empty();
                };
                let surface_format = {
                    let surfaces = runtime.wgpu.surfaces.read().unwrap();
                    surfaces[&monitor_id].format
                };

                let config = WallpaperConfig {
                    surface_size: size,
                    surface_format,
                };

                wall.configure(&runtime.wgpu, config);

                PostEventActions::REDRAW
            }
            WaylandEvent::MonitorPlugged {
                id: monitor_id,
                name: monitor_name,
            } => {
                runtime.wgpu.register_surface(&runtime.wayland, monitor_id);

                debug!(?monitor_id, ?monitor_name, "new monitor detected");

                if let Ok(mut profile) = SetupProfile::read()
                    && let Some(info) = profile.monitors.remove(monitor_name.as_str())
                {
                    let event = NewWallpaperEvent {
                        path: info.path,
                        ty: info.wallpaper_type,
                        target: WallpaperTarget::ForMonitor(monitor_id),
                    };

                    runtime.task_pool.emitter.emit(event).unwrap();
                }

                let mut run = RunningWallpapers::new(
                    runtime
                        .wallpaper_config(monitor_id)
                        .unwrap_or_else(|| panic!("no config for {monitor_id:?}")),
                    self.config.animation.clone(),
                );

                run.effects_builder.add_builtins(&self.config.effects);

                self.wallpapers.insert(monitor_id, run);

                PostEventActions::empty()
            }
            WaylandEvent::MonitorUnplugged {
                id: monitor_id,
                name,
            } => {
                debug!(?monitor_id, "unplugged a monitor");

                _ = self.wallpapers.remove(&monitor_id);

                if let Some(state) = self.wallpaper_states.get_mut(&name) {
                    state.is_active = false;
                }

                runtime.wgpu.unregister_surface(monitor_id);

                PostEventActions::empty()
            }
            WaylandEvent::CursorMoved { position: _ } => PostEventActions::empty(),
        }
    }
}

impl Handle<NewWallpaperEvent> for WallpaperApp {
    async fn handle(
        &mut self,
        runtime: &mut Runtime,
        event: NewWallpaperEvent,
    ) -> PostEventActions {
        let NewWallpaperEvent { path, ty, target } = event;

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

            let monitor_name = {
                let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                monitors[&monitor_id].name.clone()
            };

            let monitor_profile = Monitor {
                wallpaper_type: ty,
                path: path.clone(),
            };

            if let Err(error) = SetupProfile::default()
                .with(monitor_name.to_string(), monitor_profile)
                .store()
            {
                error!(?error, "failed to save setup profile");
            }

            let config = runtime
                .wallpaper_config(monitor_id)
                .unwrap_or_else(|| panic!("no config for {monitor_id:?}"));
            let packages = self.package_registry.clone();

            runtime
                .task_pool
                .spawn_event(async move || WallpaperPreparedEvent {
                    wallpaper: wallpaper::create(gpu, &path, ty, config, packages).await,
                    monitor_id,
                })
                .await;
        }

        PostEventActions::empty()
    }
}

impl Handle<WallpaperPreviewEvent> for WallpaperApp {
    async fn handle(
        &mut self,
        runtime: &mut Runtime,
        event: WallpaperPreviewEvent,
    ) -> PostEventActions {
        let response = IpcResponse {
            body: DaemonResponse::Preview {
                width: event.size.x,
                height: event.size.y,
                rgba: vec![],
            },
            destination_id: event.sender_id,
        };
        runtime.ipc_sender.send(response).unwrap();

        debug!(?event);

        PostEventActions::empty()
    }
}
