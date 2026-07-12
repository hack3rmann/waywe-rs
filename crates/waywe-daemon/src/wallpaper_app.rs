use crate::{
    event_loop::WallpaperTarget,
    wallpaper::{
        self, Wallpaper, WallpaperConfig, optimized::OptimizedWallpaper,
        package_registry::PackageRegistry, preview::PreviewPipeline, transition::RunningWallpapers,
    },
};
use calloop::channel::Sender;
use display_error_chain::ErrorChainExt;
use glam::UVec2;
use smallvec::{SmallVec, smallvec};
use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry},
    io::ErrorKind,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{debug, error};
use waywe_ipc::{
    WallpaperType,
    command::{DaemonError, DaemonResponse, DaemonResult, PauseMode},
    config::Config,
    ipc::server::{ClientId, IpcResponse},
    profile::{Monitor, SetupProfile, SetupProfileError},
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
    pub wallpaper_paths: BTreeMap<MonitorName, PathBuf>,
    pub config: Arc<Config>,
    pub package_registry: PackageRegistry,
    pub last_instant: Option<Instant>,
}

impl WallpaperApp {
    pub fn from_config(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            ..Default::default()
        }
    }
}

pub struct WallpaperPreparedEvent {
    pub path: PathBuf,
    pub wallpaper: OptimizedWallpaper,
    pub monitor_id: MonitorId,
    pub sender_id: Option<ClientId>,
}

impl TryReplicate for WallpaperPreparedEvent {}

#[derive(Clone, Debug)]
pub struct NewWallpaperEvent {
    pub path: PathBuf,
    pub ty: WallpaperType,
    pub target: WallpaperTarget,
    pub sender_id: Option<ClientId>,
}

#[derive(Clone, Debug)]
pub struct WallpaperPreviewEvent {
    pub path: PathBuf,
    pub ty: WallpaperType,
    pub size: UVec2,
    pub sender_id: ClientId,
    pub time: Duration,
}

#[derive(Clone, Debug)]
pub struct WallpaperPauseEvent {
    pub target: WallpaperTarget,
    pub mode: PauseMode,
    pub sender_id: ClientId,
}

#[derive(Clone, Debug)]
pub struct CurrentWallpaperEvent {
    pub target: WallpaperTarget,
    pub sender_id: ClientId,
}

#[derive(Clone, Debug)]
pub struct ConfigReloadEvent {
    pub path: Option<PathBuf>,
    pub config: Option<Config>,
    pub sender_id: Option<ClientId>,
}

impl App for WallpaperApp {
    fn populate_handler(&mut self, handler: &mut EventHandler<Self>) {
        handler
            .add_event::<WaylandEvent>()
            .add_event::<NewWallpaperEvent>()
            .add_event::<WallpaperPreparedEvent>()
            .add_event::<WallpaperPauseEvent>()
            .add_event::<WallpaperPreviewEvent>()
            .add_event::<CurrentWallpaperEvent>()
            .add_event::<ConfigReloadEvent>();
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
        let WallpaperPauseEvent {
            target,
            mode,
            sender_id,
        } = event;

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

        runtime
            .ipc
            .send(IpcResponse {
                body: Ok(DaemonResponse::PauseDone),
                destination_id: sender_id,
            })
            .unwrap();

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
            path,
            monitor_id,
            sender_id,
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

        match self.wallpaper_states.entry(monitor_name.clone()) {
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

        self.wallpaper_paths.insert(monitor_name, path);

        if let Some(destination_id) = sender_id {
            runtime
                .ipc
                .send(IpcResponse {
                    body: Ok(DaemonResponse::WallpaperSet),
                    destination_id,
                })
                .unwrap();
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

                match SetupProfile::read() {
                    Ok(mut profile) => 'ok: {
                        debug!("read setup profile {profile:#?}");

                        let Some(info) = profile.monitors.remove(monitor_name.as_str()) else {
                            break 'ok;
                        };
                        let event = NewWallpaperEvent {
                            path: info.path,
                            ty: info.wallpaper_type,
                            target: WallpaperTarget::ForMonitor(monitor_id),
                            sender_id: None,
                        };

                        runtime.task_pool.emitter.emit(event).unwrap();
                    }
                    Err(SetupProfileError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                        tracing::debug!("no setup profile present");
                    }
                    Err(error) => error!(error = %error.chain(), "failed to read setup profile"),
                }

                let wall_config = runtime
                    .wallpaper_config(monitor_id)
                    .unwrap_or_else(|| panic!("no config for {monitor_id:?}"));

                let wallpapers = RunningWallpapers::new(wall_config, self.config.clone());

                self.wallpapers.insert(monitor_id, wallpapers);

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
        let NewWallpaperEvent {
            path,
            ty,
            target,
            sender_id,
        } = event;

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
            let ipc = runtime.ipc.clone();

            runtime
                .task_pool
                .spawn(async move |mut emitter| {
                    match wallpaper::create(gpu, &path, ty, config, packages).await {
                        Ok(wallpaper) => emitter
                            .emit(WallpaperPreparedEvent {
                                path,
                                wallpaper,
                                monitor_id,
                                sender_id,
                            })
                            .unwrap(),
                        Err(error) => report_error(&ipc, sender_id, error.clone()),
                    }
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
        WallpaperPreviewEvent {
            path,
            ty,
            size,
            sender_id,
            time,
        }: WallpaperPreviewEvent,
    ) -> PostEventActions {
        let gpu = runtime.wgpu.clone();
        let ipc = runtime.ipc.clone();

        let packages = self.package_registry.clone();

        const MAX_PREVIEW_SIZE: u32 = 8192;

        if size.x > MAX_PREVIEW_SIZE || size.y > MAX_PREVIEW_SIZE {
            error!(
                ?size,
                max_size = MAX_PREVIEW_SIZE,
                "max preview size exceeded"
            );

            let response = IpcResponse {
                body: Err(DaemonError::ImageDimensionsTooBig {
                    width: size.x,
                    height: size.y,
                    max_width: MAX_PREVIEW_SIZE,
                    max_height: MAX_PREVIEW_SIZE,
                }),
                destination_id: sender_id,
            };
            ipc.send(response).unwrap();
        }

        let config = WallpaperConfig {
            surface_size: size,
            surface_format: wgpu::TextureFormat::Rgba8Unorm,
        };

        runtime
            .task_pool
            .spawn(async move |_| {
                let mut wallpaper =
                    match wallpaper::create(Arc::clone(&gpu), &path, ty, config, packages).await {
                        Ok(wallpaper) => wallpaper,
                        Err(error) => {
                            report_error(&ipc, Some(sender_id), error.clone());
                            return;
                        }
                    };

                wallpaper.advance_time(time);

                let pipeline = PreviewPipeline::new(&gpu, config);

                pipeline.render_async(&gpu, &mut wallpaper, move |buffer| {
                    let rgba = buffer.get_mapped_range(..).unwrap().to_vec();

                    ipc.send(IpcResponse {
                        body: Ok(DaemonResponse::Preview {
                            width: size.x,
                            height: size.y,
                            rgba,
                        }),
                        destination_id: sender_id,
                    })
                    .unwrap();
                });
            })
            .await;

        PostEventActions::empty()
    }
}

impl Handle<CurrentWallpaperEvent> for WallpaperApp {
    async fn handle(
        &mut self,
        runtime: &mut Runtime,
        CurrentWallpaperEvent { target, sender_id }: CurrentWallpaperEvent,
    ) -> PostEventActions {
        let current = match target {
            WallpaperTarget::ForMonitor(id) => {
                let name = runtime.wayland.client_state.monitor_name(id).unwrap();
                let path = self.wallpaper_paths[&name].clone();

                HashMap::from_iter([(name.to_string(), path)])
            }
            WallpaperTarget::ForAll => self
                .wallpaper_paths
                .iter()
                .map(|(name, path)| (name.to_string(), path.clone()))
                .collect(),
        };

        runtime
            .ipc
            .send(IpcResponse {
                body: Ok(DaemonResponse::Current(current)),
                destination_id: sender_id,
            })
            .unwrap();

        PostEventActions::empty()
    }
}

impl Handle<ConfigReloadEvent> for WallpaperApp {
    async fn handle(
        &mut self,
        runtime: &mut Runtime,
        // FIXME(hack3rmann): use path
        ConfigReloadEvent {
            path: _,
            config,
            sender_id,
        }: ConfigReloadEvent,
    ) -> PostEventActions {
        self.config = Arc::new(config.unwrap_or_else(Config::read));

        for wall in self.wallpapers.values_mut() {
            wall.reload_config(&runtime.wgpu, self.config.clone());
        }

        if let Some(destination_id) = sender_id {
            runtime
                .ipc
                .send(IpcResponse {
                    body: Ok(DaemonResponse::ConfigReloaded),
                    destination_id,
                })
                .unwrap();
        }

        PostEventActions::REDRAW
    }
}

fn report_error(
    ipc: &Sender<IpcResponse<DaemonResult>>,
    destination_id: Option<ClientId>,
    error: DaemonError,
) {
    error!(error = %error.chain());

    let Some(destination_id) = destination_id else {
        return;
    };

    ipc.send(IpcResponse {
        body: Err(error.clone()),
        destination_id,
    })
    .unwrap();
}
