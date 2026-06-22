use crate::wallpaper_app::{NewWallpaperEvent, WallpaperPauseEvent};
use calloop::{
    EventLoop as CalloopEventLoop, LoopHandle, LoopSignal,
    signals::{Signal, Signals},
    timer::{TimeoutAction, Timer},
};
use std::{io, vec::Drain};
use thiserror::Error;
use tokio::runtime::{Builder as AsyncRuntimeBuilder, Runtime as AsyncRuntime};
use tracing::info;
use waywe_ipc::{DaemonCommand, IpcServer, ipc::server::CreateServerError};
use waywe_runtime::{
    Runtime,
    app::{App, DynApp},
    event::{Event, EventReceiver, IntoEvent, PostEventActions},
    frame::{FrameError, FrameInfo},
    task_pool::TaskPool,
    wayland::{MonitorId, Wayland},
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

        let wayland = Wayland::default();
        let task_pool = TaskPool::new(event_emitter);
        let runtime = Runtime::new(wayland.clone(), task_pool);
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
        };

        Self::register_sources(&handle, signals, custom_receiver, wayland)?;
        state.process_event_queue();

        Ok(Self { calloop, state })
    }

    pub fn run(&mut self) {
        self.state.runtime.timer.mark_event_loop_start_time();

        self.calloop
            .run(None, &mut self.state, move |state| {
                state.event_loop_frame();
            })
            .expect("failed to run event loop");

        self.state.tokio.block_on(async {
            self.state.app.exit(&mut self.state.runtime).await;
        });
    }

    fn register_sources(
        handle: &LoopHandle<'static, LoopState>,
        signals: Signals,
        custom_receiver: EventReceiver,
        wayland: Wayland,
    ) -> Result<(), CreateEventLoopError> {
        handle
            .insert_source(signals, |event, &mut (), state| {
                info!(signal = ?event.signal(), "caught stop signal");
                state.loop_signal.stop();
            })
            .map_err(calloop::Error::from)?;

        handle
            .insert_source(wayland, move |event, &mut (), state| {
                state.event_queue.add(event);
            })
            .map_err(calloop::Error::from)?;

        handle.insert_idle(move |state| {
            state.runtime.wayland.flush();
        });

        let ipc = IpcServer::<DaemonCommand>::new()?;
        handle
            .insert_source(ipc, move |command, &mut (), state| {
                state.handle_daemon_command(command);
            })
            .map_err(calloop::Error::from)?;

        handle
            .insert_source(custom_receiver, move |event, &mut (), state| {
                state.event_queue.add_dyn(event);
            })
            .map_err(calloop::Error::from)?;

        Ok(())
    }
}

struct LoopState {
    loop_signal: LoopSignal,
    loop_handle: LoopHandle<'static, Self>,
    app: DynApp,
    runtime: Runtime,
    event_queue: EventQueue,
    tokio: AsyncRuntime,
    is_frame_requested: bool,
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

    fn handle_daemon_command(&mut self, command: DaemonCommand) {
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

        let event = match command {
            DaemonCommand::Show { path, monitor, ty } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return;
                };

                NewWallpaperEvent { path, ty, target }.into_event()
            }
            DaemonCommand::Pause { monitor } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return;
                };

                WallpaperPauseEvent { target }.into_event()
            }
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
