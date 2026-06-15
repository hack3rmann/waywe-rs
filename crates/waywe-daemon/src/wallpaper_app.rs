use crate::{
    event_loop::WallpaperTarget,
    wallpaper::{
        self, Wallpaper, WallpaperConfig, optimized::OptimizedWallpaper,
        package_registry::PackageRegistry, transition::RunningWallpapers,
    },
};
use smallvec::{SmallVec, smallvec};
use std::{
    collections::{BTreeMap, btree_map::Entry},
    panic::{self, AssertUnwindSafe},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use tracing::{debug, error};
use waywe_ipc::{
    WallpaperType,
    config::Config,
    profile::{Monitor, SetupProfile},
};
use waywe_runtime::{
    ControlFlow, Runtime,
    app::App,
    event::{EventHandler, Handle, TryReplicate},
    frame::{FrameError, FrameInfo},
    gpu::SurfaceResult,
    wayland::{MonitorId, MonitorMap, WaylandEvent},
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
    pub wallpaper_states: BTreeMap<Arc<str>, WallpaperState>,
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

    pub fn set_wallpaper(
        &mut self,
        runtime: &Runtime,
        wallpaper: OptimizedWallpaper,
        monitor_id: MonitorId,
    ) {
        match self.wallpapers.entry(monitor_id) {
            Entry::Vacant(entry) => {
                let mut wallpapers = RunningWallpapers::new(
                    runtime.wallpaper_config(monitor_id),
                    self.config.animation.clone(),
                );

                wallpapers
                    .effects_builder
                    .add_builtins(&self.config.effects);
                wallpapers.enqueue_wallpaper(&runtime.wgpu, wallpaper);

                entry.insert(wallpapers);
            }
            Entry::Occupied(mut occupied_entry) => occupied_entry
                .get_mut()
                .enqueue_wallpaper(&runtime.wgpu, wallpaper),
        }

        let monitor_name = {
            let monitors = runtime.wayland.client_state.monitors.read().unwrap();
            Arc::clone(&monitors[&monitor_id].name)
        };

        if let Entry::Vacant(entry) = self.wallpaper_states.entry(monitor_name) {
            entry.insert(WallpaperState::ACTIVE_RUNNING);
        }
    }
}

pub struct WallpaperPreparedEvent {
    pub wallpaper: OptimizedWallpaper,
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
                Arc::clone(&monitors[&monitor_id].name)
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

            // BUG(hack3rmann): if we panic between `surface.get_current_texture` and `surface.present`,
            // `wgpu` fails to destroy `SwapchainAcquireSemaphore`, which potentially leads to
            // a segmentation fault. This is a `wgpu` bug, see
            // <https://github.com/gfx-rs/wgpu/issues/8243>
            let unwind_result = panic::catch_unwind(AssertUnwindSafe(|| {
                wallpapers.advance_time(time_delta);
                wallpapers.render(&runtime.wgpu, &surface.texture, &mut encoder)
            }));

            runtime.wgpu.queue.submit([encoder.finish()]);
            surface.present();

            let result = match unwind_result {
                Ok(res) => res,
                Err(payload) => panic::resume_unwind(payload),
            };

            results.push(result);

            if let Some(state) = self.wallpaper_states.get_mut(&monitor_name) {
                *state = state.redraw_completed();
            }

            if needs_reconfigure {
                runtime.wgpu.reconfigure_surface(monitor_id);
            }
        }

        if results
            .iter()
            .all(|res| *res == Err(FrameError::NoWorkToDo))
        {
            runtime.control_flow = ControlFlow::Idle;
            self.last_instant = None;
            return Err(FrameError::NoWorkToDo);
        }

        runtime.control_flow = ControlFlow::Busy;

        let results = results.iter().flatten().cloned();
        let frame_info = results
            .reduce(|acc, elem| acc.min_or_60_fps(elem))
            .expect("at least one Ok FrameInfo");

        Ok(frame_info)
    }
}

impl Handle<WallpaperPauseEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WallpaperPauseEvent) {
        let WallpaperPauseEvent { target } = event;

        match target {
            WallpaperTarget::ForAll => {
                for state in self.wallpaper_states.values_mut() {
                    state.kind = state.kind.inverted();
                }
            }
            WallpaperTarget::ForMonitor(id) => {
                let name = runtime.wayland.client_state.monitor_name(id).unwrap();
                let state = self.wallpaper_states.get_mut(&name).unwrap();

                state.kind = state.kind.inverted();
            }
        }
    }
}

impl Handle<WallpaperPreparedEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WallpaperPreparedEvent) {
        let WallpaperPreparedEvent {
            mut wallpaper,
            monitor_id,
        } = event;

        // NOTE(hack3rmann): we may get outdated wallpaper configuration if resize event comes
        // before WallpaperPreparedEvent and after NewWallpaperEvent
        wallpaper.configure(&runtime.wgpu, runtime.wallpaper_config(monitor_id));

        runtime.control_flow.busy();
        self.set_wallpaper(runtime, wallpaper, monitor_id);
    }
}

impl Handle<WaylandEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WaylandEvent) {
        match event {
            WaylandEvent::ResizeRequested { monitor_id, size } => {
                runtime.wgpu.resize_surface(monitor_id, size);

                let Some(monitor_name) = runtime.wayland.client_state.monitor_name(monitor_id)
                else {
                    return;
                };

                if let Some(WallpaperState {
                    kind: WallpaperStateKind::Paused { needs_redraw },
                    ..
                }) = self.wallpaper_states.get_mut(&monitor_name)
                {
                    *needs_redraw = true;
                }

                let Some(wall) = self.wallpapers.get_mut(&monitor_id) else {
                    return;
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
            }
            WaylandEvent::MonitorPlugged {
                id: monitor_id,
                name: monitor_name,
            } => {
                runtime.wgpu.register_surface(&runtime.wayland, monitor_id);

                if let Some(state) = self.wallpaper_states.get_mut(&monitor_name) {
                    state.is_active = true;

                    if let WallpaperStateKind::Paused { needs_redraw } = &mut state.kind {
                        *needs_redraw = true;
                    }
                }

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
            }
            WaylandEvent::CursorMoved { position: _ } => {}
        }
    }
}

impl Handle<NewWallpaperEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: NewWallpaperEvent) {
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

            let monitors = runtime.wayland.client_state.monitors.read().unwrap();
            let monitor = &monitors[&monitor_id];
            let monitor_name = Arc::clone(&monitor.name);
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

            let config = runtime.wallpaper_config(monitor_id);
            let packages = self.package_registry.clone();

            runtime.task_pool.spawn(move |mut emitter| {
                let event = WallpaperPreparedEvent {
                    wallpaper: wallpaper::create(gpu, &path, ty, config, packages),
                    monitor_id,
                };

                emitter.emit(event).unwrap();
            });
        }
    }
}
