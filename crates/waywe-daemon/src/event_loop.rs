use crate::{
    config::ConfigEventSource,
    wallpaper_app::{
        ConfigReloadEvent, CurrentWallpaperEvent, NewWallpaperEvent, WallpaperPauseEvent,
        WallpaperPreviewEvent,
    },
};
use calloop::{
    EventLoop as CalloopEventLoop, LoopHandle, LoopSignal, RegistrationToken,
    channel::{Channel, channel},
    signals::{Signal, Signals},
    timer::{TimeoutAction, Timer},
};
use display_error_chain::ErrorChainExt;
use glam::UVec2;
use miette_diagnostic_chain::DiagnosticChain;
use std::{io, vec::Drain};
use thiserror::Error;
use tokio::runtime::{Builder as AsyncRuntimeBuilder, Runtime as AsyncRuntime};
use tracing::{debug, error, info};
use waywe_ipc::{
    DaemonCommand, IpcServer,
    command::DaemonResult,
    ipc::server::{CreateServerError, IpcEvent, IpcResponse},
};
use waywe_runtime::{
    Runtime,
    app::{App, DynApp},
    event::{Event, EventReceiver, IntoEvent, PostEventActions},
    frame::{FrameError, FrameInfo},
    task_pool::TaskPool,
    wayland::{MonitorId, Wayland, WaylandEventSource},
};

#[derive(Debug, Error)]
pub enum CreateEventLoopError {
    #[error("failed to create event queue")]
    CrateEventQueue(#[source] io::Error),
    #[error(transparent)]
    Ipc(#[from] CreateServerError),
    #[error("failed to create calloop event loop")]
    Calloop(#[from] calloop::Error),
}

pub struct EventLoop {
    calloop: CalloopEventLoop<'static, LoopState>,
    state: LoopState,
}

impl EventLoop {
    pub fn new(app: impl App) -> Result<Self, CreateEventLoopError> {
        // NOTE(hack3rmann): `Signals::new` blocks given signals from the current thread
        // It's important that we create this before spawning any thread, so the child thread
        // will ingerit the blocked signals
        let signals = Signals::new(&[
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGHUP,
            Signal::SIGTERM,
        ])?;

        let tokio = AsyncRuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");

        let (mut event_queue, custom_receiver) =
            EventQueue::new().map_err(CreateEventLoopError::CrateEventQueue)?;
        let event_emitter = custom_receiver.make_emitter().unwrap();

        let (ipc_sender, ipc_channel) = channel();
        let wayland = Wayland::default();
        let task_pool = TaskPool::new(event_emitter);
        let runtime = Runtime::new(wayland.clone(), task_pool, ipc_sender);
        let app = DynApp::new(app);

        runtime.wayland.drain_stored_events(|event| {
            event_queue.add(event);
        });

        let calloop = CalloopEventLoop::try_new().map_err(CreateEventLoopError::Calloop)?;
        let loop_signal = calloop.get_signal();
        let handle = calloop.handle();

        let mut state = LoopState {
            loop_signal,
            app,
            runtime,
            event_queue,
            tokio,
            loop_handle: handle.clone(),
            is_frame_requested: true,
            config_watcher_token: None,
        };

        Self::register_sources(&handle, signals, custom_receiver, wayland, ipc_channel)?;

        if !state.app.config().config.disable_hot_reload {
            add_config_watcher_source(&mut state)?;
        }

        state.process_event_queue();

        Ok(Self { calloop, state })
    }

    pub fn run(&mut self) {
        self.state.runtime.timer.mark_event_loop_start_time();

        self.calloop
            .run(None, &mut self.state, move |state| {
                state.event_loop_frame();
            })
            .unwrap_or_else(|err| panic!("failed to run event loop: {}", err.chain()));

        self.state.tokio.block_on(async {
            self.state.app.exit(&mut self.state.runtime).await;
        });
    }

    fn register_sources(
        handle: &LoopHandle<'static, LoopState>,
        signals: Signals,
        custom_receiver: EventReceiver,
        wayland: Wayland,
        ipc_channel: Channel<IpcResponse<DaemonResult>>,
    ) -> Result<(), CreateEventLoopError> {
        handle
            .insert_source(signals, |event, &mut (), state| {
                info!(signal = ?event.signal(), "caught stop signal");
                state.loop_signal.stop();
            })
            .map_err(calloop::Error::from)?;

        handle
            .insert_source(
                WaylandEventSource::new(wayland),
                move |event, &mut (), state| {
                    state.event_queue.add(event);
                },
            )
            .map_err(calloop::Error::from)?;

        let ipc = IpcServer::<DaemonCommand, DaemonResult>::new(ipc_channel)?;
        handle
            .insert_source(ipc, move |command, &mut (), state| {
                state.handle_daemon_command(command);
            })
            .map_err(calloop::Error::from)?;

        handle
            .insert_source(custom_receiver, move |mut event, &mut (), state| {
                handle_event_loop_event(&mut event, state);
                state.event_queue.add_dyn(event);
            })
            .map_err(calloop::Error::from)?;

        Ok(())
    }
}

fn add_config_watcher_source(state: &mut LoopState) -> Result<(), calloop::Error> {
    let token = state.loop_handle.insert_source(
        ConfigEventSource::new(),
        move |config, &mut (), state| {
            let config = match config {
                Ok(config) => config,
                Err(error) => {
                    error!(error = %error.diagnostic_chain(), "failed to reload config");
                    return;
                }
            };

            state.event_queue.add(ConfigReloadEvent {
                path: None,
                config: Some(config),
                sender_id: None,
            });
        },
    )?;

    state.config_watcher_token = Some(token);

    Ok(())
}

#[derive(Clone, Debug)]
pub struct EnableConfigWatcher;

#[derive(Clone, Debug)]
pub struct DisableConfigWatcher;

fn handle_event_loop_event(event: &mut Event, state: &mut LoopState) {
    event.handle_sync(|_: EnableConfigWatcher| {
        debug!("enabling config hot reload");

        if let Err(error) = add_config_watcher_source(state) {
            error!(error = %error.chain(), "failed to enable config watcher");
        }

        PostEventActions::empty()
    });

    event.handle_sync(|_: DisableConfigWatcher| {
        debug!("disabling config hot reload");

        let Some(token) = state.config_watcher_token.take() else {
            error!("bug: tried to disable unexistent config watcher");
            return PostEventActions::empty();
        };

        state.loop_handle.remove(token);

        PostEventActions::empty()
    });
}

struct LoopState {
    loop_signal: LoopSignal,
    loop_handle: LoopHandle<'static, Self>,
    app: DynApp,
    runtime: Runtime,
    event_queue: EventQueue,
    tokio: AsyncRuntime,
    is_frame_requested: bool,
    config_watcher_token: Option<RegistrationToken>,
}

impl LoopState {
    fn process_event_queue(&mut self) {
        self.tokio.block_on(async {
            self.runtime.task_pool.erase_finished().await;

            for mut event in self.event_queue.drain() {
                let actions = self.app.handle_event(&mut self.runtime, &mut event).await;

                if actions.contains(PostEventActions::REDRAW) {
                    self.is_frame_requested = true;
                }
            }
        });
    }

    fn event_loop_frame(&mut self) {
        self.process_event_queue();

        if !self.is_frame_requested {
            return;
        }

        self.runtime.timer.mark_frame_start();

        let info = self.tokio.block_on(self.app.frame(&mut self.runtime));

        match info {
            Ok(FrameInfo {
                target_frame_time: Some(target),
            }) => {
                let delay = self.runtime.timer.next_frame_delay(target);

                self.loop_handle
                    .insert_source(Timer::from_duration(delay), |_, &mut (), state| {
                        state.is_frame_requested = true;
                        TimeoutAction::Drop
                    })
                    .unwrap();
            }
            Err(FrameError::StopRequested) => {
                self.loop_signal.stop();
            }
            Ok(FrameInfo {
                target_frame_time: None,
            })
            | Err(FrameError::NoWorkToDo) => {
                // go sleep mode
            }
        }

        self.is_frame_requested = false;
    }

    fn handle_daemon_command(&mut self, command: IpcEvent<DaemonCommand>) {
        let wayland = self.runtime.wayland.clone();
        let get_target = move |monitor_name: Option<&str>| {
            let Some(name) = monitor_name else {
                return Some(WallpaperTarget::ForAll);
            };

            let target = wayland
                .client_state
                .monitor_id(name)
                .map(WallpaperTarget::ForMonitor)?;

            Some(target)
        };

        let event = match command.event {
            DaemonCommand::Show { path, monitor, ty } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return;
                };

                NewWallpaperEvent {
                    path,
                    ty,
                    target,
                    sender_id: Some(command.sender_id),
                }
                .into_event()
            }
            DaemonCommand::Preview {
                ty,
                path,
                width,
                height,
                time,
            } => {
                debug!(?ty, ?path, ?width, ?height, "preview");

                WallpaperPreviewEvent {
                    path,
                    ty,
                    size: UVec2::new(width, height),
                    sender_id: command.sender_id,
                    time,
                }
                .into_event()
            }
            DaemonCommand::Pause { monitor, mode } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return;
                };

                WallpaperPauseEvent {
                    target,
                    mode,
                    sender_id: command.sender_id,
                }
                .into_event()
            }
            DaemonCommand::Current { monitor } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return;
                };

                CurrentWallpaperEvent {
                    target,
                    sender_id: command.sender_id,
                }
                .into_event()
            }
            DaemonCommand::ConfigReload { path } => ConfigReloadEvent {
                path,
                sender_id: Some(command.sender_id),
                config: None,
            }
            .into_event(),
        };

        self.event_queue.add_dyn(event);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallpaperTarget {
    #[default]
    ForAll,
    ForMonitor(MonitorId),
}

pub struct EventQueue {
    events: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Result<(Self, EventReceiver), io::Error> {
        Ok((Self { events: vec![] }, EventReceiver::new()?))
    }

    pub fn add(&mut self, event: impl IntoEvent) {
        self.add_dyn(event.into_event());
    }

    pub fn add_dyn(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Drain<'_, Event> {
        self.events.drain(..)
    }
}
