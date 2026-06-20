use crate::wallpaper_app::{NewWallpaperEvent, WallpaperPauseEvent};
use calloop::{
    EventLoop as CalloopEventLoop, LoopHandle, LoopSignal, RegistrationToken,
    signals::{Signal, Signals},
    timer::{TimeoutAction, Timer},
};
use std::{io, time::Instant, vec::Drain};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use tracing::{debug, error};
use waywe_ipc::{DaemonCommand, IpcServer, WallpaperType, ipc::server::CreateServerError};
use waywe_runtime::{
    Runtime,
    app::{App, DynApp},
    event::{Event, EventReceiver, IntoEvent},
    frame::FrameError,
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

struct LoopState {
    loop_signal: LoopSignal,
    app: DynApp,
    runtime: Runtime,
    event_queue: EventQueue,
    tokio: tokio::runtime::Handle,
    frame_timer_token: Option<RegistrationToken>,
}

pub struct EventLoop {
    #[expect(dead_code)]
    tokio_rt: tokio::runtime::Runtime,
    calloop: CalloopEventLoop<'static, LoopState>,
    state: LoopState,
}

impl EventLoop {
    pub fn new(app: impl App) -> Result<Self, CreateEventLoopError> {
        let tokio_rt = AsyncRuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");
        let tokio = tokio_rt.handle().clone();

        let (event_queue, custom_receiver) =
            EventQueue::new().map_err(CreateEventLoopError::CrateEventQueue)?;
        let event_emitter = custom_receiver.make_emitter().unwrap();

        let wayland = Wayland::default();
        let task_pool = TaskPool::new(event_emitter);
        let runtime = Runtime::new(wayland, task_pool);
        let ipc = IpcServer::<DaemonCommand>::new()?;
        let app = DynApp::new(app);

        let calloop = CalloopEventLoop::try_new().map_err(CreateEventLoopError::Calloop)?;
        let loop_signal = calloop.get_signal();
        let handle = calloop.handle();

        let mut state = LoopState {
            loop_signal,
            app,
            runtime,
            event_queue,
            tokio,
            frame_timer_token: None,
        };

        register_sources(&handle, ipc, custom_receiver, &mut state)?;
        state.runtime.timer.mark_event_loop_start_time();
        absorb_wayland_events(&mut state);
        process_event_queue(&mut state);

        Ok(Self {
            tokio_rt,
            calloop,
            state,
        })
    }

    pub fn run(&mut self) {
        let handle = self.calloop.handle();
        request_frame(&handle, &mut self.state);

        self.calloop
            .run(None, &mut self.state, |_| {})
            .expect("failed to run event loop");

        self.state
            .tokio
            .block_on(self.state.app.exit(&mut self.state.runtime));
    }
}

fn register_sources(
    handle: &LoopHandle<'static, LoopState>,
    ipc: IpcServer<DaemonCommand>,
    custom_receiver: EventReceiver,
    state: &mut LoopState,
) -> Result<(), calloop::Error> {
    let signals = Signals::new(&[
        Signal::SIGINT,
        Signal::SIGQUIT,
        Signal::SIGHUP,
        Signal::SIGTERM,
    ])?;
    handle.insert_source(signals, |_, &mut (), state| {
        debug!("caught stop signal");
        state.loop_signal.stop();
    })?;

    let wayland = state.runtime.wayland.clone();
    handle.insert_source(wayland, {
        let handle = handle.clone();
        move |event, &mut (), state| {
            state.event_queue.add(event);
            request_frame(&handle, state);
        }
    })?;

    handle.insert_source(ipc, {
        let handle = handle.clone();
        move |command, &mut (), state| {
            handle_daemon_command(state, command);
            request_frame(&handle, state);
        }
    })?;

    handle.insert_source(custom_receiver, {
        let handle = handle.clone();
        move |event, &mut (), state| {
            state.event_queue.events.push(event);
            request_frame(&handle, state);
        }
    })?;

    Ok(())
}

fn absorb_wayland_events(state: &mut LoopState) {
    state.runtime.wayland.display_roundtrip();
    state.runtime.wayland.drain_stored_events(|event| {
        state.event_queue.add(event);
    });
}

fn process_event_queue(state: &mut LoopState) {
    state
        .tokio
        .block_on(state.runtime.task_pool.erase_finished());

    for mut event in state.event_queue.drain() {
        state
            .tokio
            .block_on(state.app.handle_event(&mut state.runtime, &mut event));
    }
}

fn request_frame(handle: &LoopHandle<'static, LoopState>, state: &mut LoopState) {
    if let Some(token) = state.frame_timer_token.take() {
        handle.remove(token);
    }

    match handle.insert_source(Timer::immediate(), on_frame_timer) {
        Ok(token) => state.frame_timer_token = Some(token),
        Err(error) => error!(?error, "failed to schedule frame"),
    }

    state.loop_signal.wakeup();
}

fn on_frame_timer(_: Instant, _: &mut (), state: &mut LoopState) -> TimeoutAction {
    absorb_wayland_events(state);
    state.runtime.timer.mark_frame_start();

    state
        .tokio
        .block_on(state.runtime.task_pool.erase_finished());

    for mut event in state.event_queue.drain() {
        state
            .tokio
            .block_on(state.app.handle_event(&mut state.runtime, &mut event));
    }

    match state.tokio.block_on(state.app.frame(&mut state.runtime)) {
        Ok(info) => {
            if let Some(target_frame_time) = info.target_frame_time {
                let delay = state.runtime.timer.next_frame_delay(target_frame_time);
                TimeoutAction::ToDuration(delay)
            } else {
                state.frame_timer_token = None;
                TimeoutAction::Drop
            }
        }
        Err(FrameError::StopRequested) => {
            debug!("shutting down daemon");
            state.loop_signal.stop();
            state.frame_timer_token = None;
            TimeoutAction::Drop
        }
        Err(FrameError::Skip | FrameError::NoWorkToDo) => {
            state.frame_timer_token = None;
            TimeoutAction::Drop
        }
    }
}

fn handle_daemon_command(state: &mut LoopState, command: DaemonCommand) {
    let wayland = &state.runtime.wayland;

    let get_target = |monitor_name: Option<&str>| {
        let Some(name) = monitor_name else {
            return Some(WallpaperTarget::ForAll);
        };

        let target = wayland
            .client_state
            .monitor_id(name)
            .map(WallpaperTarget::ForMonitor)?;

        Some(target)
    };

    match command {
        DaemonCommand::SetVideo { path, monitor } => {
            let Some(target) = get_target(monitor.as_deref()) else {
                return;
            };

            state.event_queue.add(NewWallpaperEvent {
                path,
                ty: WallpaperType::Video,
                target,
            });
        }
        DaemonCommand::SetImage { path, monitor } => {
            let Some(target) = get_target(monitor.as_deref()) else {
                return;
            };

            state.event_queue.add(NewWallpaperEvent {
                path,
                ty: WallpaperType::Image,
                target,
            });
        }
        DaemonCommand::SetScene { path, monitor } => {
            let Some(target) = get_target(monitor.as_deref()) else {
                return;
            };

            state.event_queue.add(NewWallpaperEvent {
                path,
                ty: WallpaperType::Scene,
                target,
            });
        }
        DaemonCommand::Pause { monitor } => {
            let Some(target) = get_target(monitor.as_deref()) else {
                return;
            };

            state.event_queue.add(WallpaperPauseEvent { target });
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallpaperTarget {
    #[default]
    ForAll,
    ForMonitor(MonitorId),
}

pub struct EventQueue {
    pub events: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Result<(Self, EventReceiver), io::Error> {
        Ok((Self { events: vec![] }, EventReceiver::new()?))
    }

    pub fn add(&mut self, event: impl IntoEvent) {
        self.events.push(event.into_event());
    }

    pub fn drain(&mut self) -> Drain<'_, Event> {
        self.events.drain(..)
    }
}
