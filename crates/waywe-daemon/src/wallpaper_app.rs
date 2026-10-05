use crate::{
    event_loop::{DisableConfigWatcher, EnableConfigWatcher, WallpaperTarget},
    wallpaper::{
        self, Wallpaper, WallpaperConfig,
        optimized::OptimizedWallpaper,
        package_registry::PackageRegistry,
        pause_rules::PauseRules,
        preview::PreviewPipeline,
        transition::{RunningWallpapers, SubmissionId},
    },
};
use calloop::channel::Sender;
use display_error_chain::ErrorChainExt;
use glam::UVec2;
use miette_diagnostic_chain::DiagnosticChain;
use smallvec::{SmallVec, smallvec};
use std::{
    collections::HashMap,
    io::ErrorKind,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{debug, error};
use waywe_config::Config;
use waywe_ipc::{
    WallpaperType,
    command::{DaemonError, DaemonResponse, DaemonResult, PauseMode},
    ipc::server::{ClientId, IpcResponse},
    profile::{Monitor, SetupProfile, SetupProfileError},
};
use waywe_runtime::{
    Runtime,
    app::App,
    event::{EventHandler, Handle, PostEventActions, TryReplicate},
    frame::{FrameError, FrameInfo},
    gpu::SurfaceResult,
    platform::{MonitorMap, MonitorName, PlatformEvent, SurfaceInfo},
};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WallpaperState {
    pub needs_redraw: bool,
}

#[derive(Default)]
pub struct WallpaperApp {
    pub wallpapers: MonitorMap<RunningWallpapers>,
    pub wallpaper_states: MonitorMap<WallpaperState>,
    pub wallpaper_paths: MonitorMap<PathBuf>,
    pub config: Arc<Config>,
    pub package_registry: PackageRegistry,
    pub last_instant: Option<Instant>,
    pub monitors: MonitorMap<SurfaceInfo>,
    pub pause_rules: PauseRules,
    pub waiting_submissions: HashMap<SubmissionId, ClientId>,
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
    pub monitor_name: MonitorName,
    pub sender_id: Option<ClientId>,
    pub wait_transition: bool,
}

impl TryReplicate for WallpaperPreparedEvent {}

#[derive(Clone, Debug)]
pub struct NewWallpaperEvent {
    pub path: PathBuf,
    pub ty: WallpaperType,
    pub target: WallpaperTarget,
    pub sender_id: Option<ClientId>,
    pub wait_transition: bool,
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
    pub sender_id: Option<ClientId>,
    pub config: Option<Config>,
}

impl App for WallpaperApp {
    fn populate_handler(&mut self, handler: &mut EventHandler<Self>) {
        handler
            .add_event::<PlatformEvent>()
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
            .replace(Instant::now())
            .map(|i| i.elapsed())
            .unwrap_or_default();

        for (monitor_name, wall) in self.wallpapers.iter_mut() {
            if let Some(state) = self.wallpaper_states.get(monitor_name.as_str())
                && !state.needs_redraw
            {
                results.push(Err(FrameError::NoWorkToDo));
                continue;
            }

            let monitor_info = &self.monitors[monitor_name.as_str()];

            let (needs_reconfigure, surface) = match runtime
                .wgpu
                .get_current_surface(runtime.platform.as_ref(), monitor_info)
            {
                SurfaceResult::Ok(texture) => (false, texture),
                SurfaceResult::Reconfigure(texture) => (true, texture),
                SurfaceResult::Skip => {
                    results.push(Err(FrameError::NoWorkToDo));
                    continue;
                }
                SurfaceResult::Err => {
                    tracing::error!(?monitor_name, "failed to get_current_texture on surface");
                    results.push(Err(FrameError::NoWorkToDo));
                    continue;
                }
            };

            let mut encoder = runtime
                .wgpu
                .device
                .create_command_encoder(&Default::default());

            wall.advance_time(time_delta);
            let result = wall.render(&runtime.wgpu, &surface.texture, &mut encoder);

            results.push(result);

            runtime.wgpu.queue.submit([encoder.finish()]);
            runtime.wgpu.queue.present(surface);

            if !wall.needs_redraw() {
                let state = self
                    .wallpaper_states
                    .get_mut(monitor_name.as_str())
                    .unwrap();
                state.needs_redraw = false;
            }

            if needs_reconfigure {
                runtime.wgpu.reconfigure_surface(monitor_name);
            }

            for id in wall.drain_finished_submissions() {
                let client_id = self.waiting_submissions.remove(&id);
                send_response(&runtime.ipc, client_id, DaemonResponse::WallpaperSet);
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

    fn config(&self) -> &Config {
        &self.config
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

        self.pause_rules.toggle(target.clone(), mode);

        match target {
            WallpaperTarget::ForAll => {
                for wall in self.wallpapers.values_mut() {
                    wall.toggle_pause(mode);
                }

                for state in self.wallpaper_states.values_mut() {
                    state.needs_redraw = true;
                }
            }
            WallpaperTarget::ForMonitor(name) => {
                let wall = self.wallpapers.get_mut(&name).unwrap();
                wall.toggle_pause(mode);

                let state = self.wallpaper_states.get_mut(&name).unwrap();
                state.needs_redraw = true;
            }
        }

        send_response(&runtime.ipc, Some(sender_id), DaemonResponse::PauseDone);

        PostEventActions::REDRAW
    }
}

impl Handle<WallpaperPreparedEvent> for WallpaperApp {
    async fn handle(
        &mut self,
        runtime: &mut Runtime,
        event: WallpaperPreparedEvent,
    ) -> PostEventActions {
        let WallpaperPreparedEvent {
            mut wallpaper,
            path,
            monitor_name,
            sender_id,
            wait_transition,
        } = event;

        let monitor_info = &self.monitors[monitor_name.as_str()];

        // NOTE(hack3rmann): wallpaper may be prepared after monitor is disconnected
        let Some(config) = runtime.wallpaper_config(monitor_info) else {
            return PostEventActions::empty();
        };

        // NOTE(hack3rmann): monitor could be unplugged when this event arrives
        let Some(run) = self.wallpapers.get_mut(&monitor_name) else {
            return PostEventActions::empty();
        };

        // NOTE(hack3rmann): we may get outdated wallpaper configuration if resize event comes
        // before WallpaperPreparedEvent and after NewWallpaperEvent
        wallpaper.configure(&runtime.wgpu, config);
        let submission_id = run.enqueue_wallpaper(&runtime.wgpu, wallpaper);

        let pause_state = self.pause_rules.get(&monitor_name);
        run.set_pause(pause_state);

        self.wallpaper_states
            .entry(monitor_name.clone())
            .and_modify(|s| s.needs_redraw = true)
            .or_insert(WallpaperState { needs_redraw: true });

        self.wallpaper_paths.insert(monitor_name, path);

        if let Some(sender_id) = sender_id {
            if wait_transition {
                self.waiting_submissions.insert(submission_id, sender_id);
            } else {
                send_response(&runtime.ipc, Some(sender_id), DaemonResponse::WallpaperSet);
            }
        }

        PostEventActions::REDRAW
    }
}

impl Handle<PlatformEvent> for WallpaperApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: PlatformEvent) -> PostEventActions {
        match event {
            PlatformEvent::ResizeRequested {
                monitor_name,
                phisical_size: size,
            } => {
                runtime.wgpu.resize_surface(&monitor_name, size);

                if let Some(state) = self.wallpaper_states.get_mut(&monitor_name) {
                    state.needs_redraw = true;
                }

                let Some(wall) = self.wallpapers.get_mut(&monitor_name) else {
                    return PostEventActions::empty();
                };
                let surface_format = {
                    let surfaces = runtime.wgpu.surfaces.read().unwrap();
                    surfaces[&monitor_name].format
                };

                let config = WallpaperConfig {
                    surface_size: size,
                    surface_format,
                };

                wall.configure(&runtime.wgpu, config);

                PostEventActions::REDRAW
            }
            PlatformEvent::MonitorPlugged { info } => {
                debug!(?info, "new monitor detected");

                self.monitors
                    .insert(info.monitor_name.clone(), info.clone());

                runtime
                    .wgpu
                    .register_surface(runtime.platform.as_ref(), &info);

                match SetupProfile::read() {
                    Ok(mut profile) => 'ok: {
                        debug!(?profile, "read setup profile");

                        let Some(profile_info) =
                            profile.monitors.remove(info.monitor_name.as_str())
                        else {
                            break 'ok;
                        };
                        let event = NewWallpaperEvent {
                            path: profile_info.path,
                            ty: profile_info.wallpaper_type,
                            target: WallpaperTarget::ForMonitor(info.monitor_name.clone()),
                            sender_id: None,
                            wait_transition: false,
                        };

                        runtime.tasks.emitter.emit(event);
                    }
                    Err(SetupProfileError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                        tracing::debug!("no setup profile present");
                    }
                    Err(error) => error!(error = %error.chain(), "failed to read setup profile"),
                }

                let wall_config = runtime
                    .wallpaper_config(&info)
                    .unwrap_or_else(|| panic!("no config for '{}'", info.monitor_name));

                // TODO(hack3rmann): add pause rules here
                let wallpapers = RunningWallpapers::new(wall_config, self.config.clone());

                self.wallpapers.insert(info.monitor_name, wallpapers);

                PostEventActions::empty()
            }
            PlatformEvent::MonitorUnplugged { monitor_name: name } => {
                debug!(%name, "unplugged a monitor");

                self.monitors.remove(name.as_str());

                _ = self.wallpapers.remove(&name);

                runtime.wgpu.unregister_surface(&name);

                PostEventActions::empty()
            }
            PlatformEvent::CursorMoved { position: _ } => PostEventActions::empty(),
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
            wait_transition,
        } = event;

        let monitor_names: SmallVec<[_; 4]> = match target {
            WallpaperTarget::ForAll => self.monitors.keys().cloned().collect(),
            WallpaperTarget::ForMonitor(name) => smallvec![name],
        };

        for monitor_name in monitor_names {
            let path = path.clone();
            let gpu = Arc::clone(&runtime.wgpu);

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

            let monitor_info = &self.monitors[monitor_name.as_str()];

            let config = runtime
                .wallpaper_config(monitor_info)
                .unwrap_or_else(|| panic!("no config for {monitor_name:?}"));
            let packages = self.package_registry.clone();
            let ipc = runtime.ipc.clone();
            let error_ipc = runtime.ipc.clone();

            runtime
                .tasks
                .spawn_blocking(move |mut emitter| {
                    match wallpaper::create(gpu, &path, ty, config, packages) {
                        Ok(wallpaper) => emitter.emit(WallpaperPreparedEvent {
                            path,
                            wallpaper,
                            monitor_name,
                            sender_id,
                            wait_transition,
                        }),
                        Err(error) => report_error(&ipc, sender_id, error.clone()),
                    }
                })
                .on_error(move |error| {
                    report_error(&error_ipc, sender_id, DaemonError::from_generic(error));
                });
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
        let error_ipc = runtime.ipc.clone();

        let packages = self.package_registry.clone();

        const MAX_PREVIEW_SIZE: u32 = 8192;

        if size.x > MAX_PREVIEW_SIZE || size.y > MAX_PREVIEW_SIZE {
            error!(
                ?size,
                max_size = MAX_PREVIEW_SIZE,
                "max preview size exceeded"
            );

            report_error(
                &ipc,
                Some(sender_id),
                DaemonError::ImageDimensionsTooBig {
                    width: size.x,
                    height: size.y,
                    max_width: MAX_PREVIEW_SIZE,
                    max_height: MAX_PREVIEW_SIZE,
                },
            );
        }

        let config = WallpaperConfig {
            surface_size: size,
            surface_format: wgpu::TextureFormat::Rgba8Unorm,
        };

        runtime
            .tasks
            .spawn_blocking(move |_| {
                let mut wallpaper =
                    match wallpaper::create(Arc::clone(&gpu), &path, ty, config, packages) {
                        Ok(wallpaper) => wallpaper,
                        Err(error) => {
                            report_error(&ipc, Some(sender_id), error);
                            return;
                        }
                    };

                wallpaper.advance_time(time);

                let pipeline = PreviewPipeline::new(&gpu, config);

                pipeline.render_async(&gpu, &mut wallpaper, move |buffer| {
                    let buffer = match buffer {
                        Ok(b) => b,
                        Err(error) => {
                            report_error(&ipc, Some(sender_id), DaemonError::from_generic(error));
                            return;
                        }
                    };
                    let rgba = buffer.get_mapped_range(..).unwrap().to_vec();

                    send_response(
                        &ipc,
                        Some(sender_id),
                        DaemonResponse::Preview {
                            width: size.x,
                            height: size.y,
                            rgba,
                        },
                    );
                });
            })
            .on_error(move |error| {
                report_error(
                    &error_ipc,
                    Some(sender_id),
                    DaemonError::from_generic(error),
                );
            });

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
            WallpaperTarget::ForMonitor(name) => {
                let path = self.wallpaper_paths[&name].clone();
                HashMap::from_iter([(name.to_string(), path)])
            }
            WallpaperTarget::ForAll => self
                .wallpaper_paths
                .iter()
                .map(|(name, path)| (name.to_string(), path.clone()))
                .collect(),
        };

        send_response(
            &runtime.ipc,
            Some(sender_id),
            DaemonResponse::Current(current),
        );

        PostEventActions::empty()
    }
}

impl Handle<ConfigReloadEvent> for WallpaperApp {
    async fn handle(
        &mut self,
        runtime: &mut Runtime,
        ConfigReloadEvent {
            path,
            sender_id,
            config,
        }: ConfigReloadEvent,
    ) -> PostEventActions {
        let config = match config {
            Some(config) => config,
            None => match Config::read_from(path.as_ref()) {
                Ok(config) => config,
                Err(error) => {
                    let report = error.diagnostic_chain().to_string();
                    report_error(&runtime.ipc, sender_id, DaemonError::Generic(report));

                    return PostEventActions::empty();
                }
            },
        };

        match (
            self.config.config.disable_hot_reload,
            config.config.disable_hot_reload,
        ) {
            (true, false) => runtime.tasks.emitter.emit(EnableConfigWatcher),
            (false, true) => runtime.tasks.emitter.emit(DisableConfigWatcher),
            (true, true) | (false, false) => {}
        }

        self.config = Arc::new(config);

        for wall in self.wallpapers.values_mut() {
            wall.reload_config(&runtime.wgpu, self.config.clone());
        }

        send_response(&runtime.ipc, sender_id, DaemonResponse::ConfigReloaded);

        debug!(config = ?self.config, "reloaded config");

        PostEventActions::REDRAW
    }
}

fn send_response(
    ipc: &Sender<IpcResponse<DaemonResult>>,
    destination_id: Option<ClientId>,
    response: DaemonResponse,
) {
    let Some(destination_id) = destination_id else {
        return;
    };

    ipc.send(IpcResponse {
        body: Ok(response),
        destination_id,
    })
    .unwrap();
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
