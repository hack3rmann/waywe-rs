use crate::wallpaper_app::{NewWallpaperEvent, WallpaperPauseEvent};
use calloop::{
    LoopSignal,
    signals::{Signal, Signals},
};
use rustix::io::Errno;
use std::{
    io,
    os::fd::AsFd as _,
    sync::{Once, atomic::Ordering, mpsc::TryRecvError},
    time::Duration,
    vec::Drain,
};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use tracing::{debug, error};
use waywe_ipc::{
    DaemonCommand, IpcServer, RecvError, WallpaperType,
    epoll::{Epoll, PolledFds},
    signals,
};
use waywe_runtime::{
    ControlFlow, CreateRuntimeError, Runtime,
    app::{App, DynApp},
    event::{AbsorbError, Event, EventReceiver, IntoEvent},
    frame::FrameError,
    task_pool::TaskPool,
    wayland::{MonitorId, Wayland},
};

#[derive(Debug, Error)]
pub enum CreateEventLoopError {
    #[error(transparent)]
    CreateRuntime(#[from] CreateRuntimeError),
    #[error("failed to create event queue")]
    CrateEventQueue(#[source] io::Error),
    #[error("failed to create epoll instance")]
    CreateEpoll(#[source] Errno),
}

pub struct EventLoop {
    // NOTE(hack3rmann): app should be dropped first to release all the resources from the runtime
    app: DynApp,
    runtime: Runtime,
    event_queue: EventQueue,
    epoll: Epoll,
}

impl EventLoop {
    pub fn new(app: impl App) -> Result<Self, CreateEventLoopError> {
        static SIGNALS_ONCE: Once = Once::new();
        SIGNALS_ONCE.call_once(signals::setup);

        let app = DynApp::new(app);

        let event_queue = EventQueue::new().map_err(CreateEventLoopError::CrateEventQueue)?;
        let event_emitter = event_queue.custom_receiver.make_emitter().unwrap();

        let wayland = Wayland::new(event_emitter.clone());
        let task_pool = TaskPool::new(event_emitter);

        let runtime = Runtime::new(wayland, ControlFlow::Busy, task_pool)?;

        let fds = [
            runtime.wayland.display.as_fd(),
            runtime.ipc.as_fd(),
            event_queue.custom_receiver.pipe_fd(),
        ];

        let epoll = Epoll::new(fds).map_err(CreateEventLoopError::CreateEpoll)?;

        Ok(Self {
            runtime,
            app,
            event_queue,
            epoll,
        })
    }

    fn run_calloop(&mut self) {
        struct ClientState {
            loop_signal: LoopSignal,
        }

        let mut event_loop =
            calloop::EventLoop::<ClientState>::try_new().expect("failed to initilize poll");
        let handle = event_loop.handle();

        let signals = Signals::new(&[Signal::SIGINT, Signal::SIGQUIT, Signal::SIGHUP]).unwrap();
        handle
            .insert_source(signals, |_, &mut (), state| {
                state.loop_signal.stop();
            })
            .unwrap();

        handle
            .insert_source(self.runtime.wayland.clone(), |_, &mut (), _| todo!())
            .unwrap();

        let ipc = IpcServer::<DaemonCommand>::new().unwrap();
        handle.insert_source(ipc, |_, &mut (), _| todo!()).unwrap();

        let mut state = ClientState {
            loop_signal: event_loop.get_signal(),
        };

        event_loop
            .run(Duration::from_secs_f32(1.0 / 60.0), &mut state, |_state| {
                // TODO: do frame when time comes
            })
            .unwrap();
    }

    async fn run_async(&mut self) {
        self.runtime.timer.mark_event_loop_start_time();

        let mut polled_fds = PolledFds::with_capacity(1);

        'event_loop: loop {
            self.runtime.timer.mark_frame_start();

            match self.runtime.control_flow {
                ControlFlow::Busy => {}
                ControlFlow::Idle => {
                    self.runtime.timer.mark_block_start();

                    match self.epoll.wait(&mut polled_fds, None) {
                        Ok(()) | Err(Errno::INTR) => {}
                        Err(error) => {
                            error!(?error, "failed to sleep on multiple sockets");
                        }
                    }

                    // Dispatch all wayland events first
                    if polled_fds.contains(&self.runtime.wayland.display) {
                        self.runtime.wayland.display_roundtrip();
                        continue;
                    }

                    self.runtime.timer.mark_block_end();
                }
                ControlFlow::ShouldStop => {
                    debug!("shutting down daemon");
                    break 'event_loop;
                }
            }

            if signals::SHOULD_EXIT.load(Ordering::Relaxed) {
                debug!("caught stop signal");
                break 'event_loop;
            }

            self.runtime.task_pool.erase_finished().await;
            self.runtime.wayland.display_roundtrip();

            if let Err(error) = self.event_queue.populate_events_from_custom()
                && !matches!(error, AbsorbError::TryRecv(TryRecvError::Empty))
            {
                error!(?error, "failed to populate custom events");
            }

            if let Err(error) = self
                .event_queue
                .populate_from_cli(&self.runtime.wayland, &self.runtime.ipc)
            {
                error!(?error, "can not recv from waywe-cli");
            }

            for mut event in self.event_queue.drain() {
                self.app.handle_event(&mut self.runtime, &mut event).await;
            }

            let info = match self.app.frame(&mut self.runtime).await {
                Ok(info) => info,
                Err(FrameError::StopRequested) => break 'event_loop,
                Err(FrameError::Skip | FrameError::NoWorkToDo) => continue 'event_loop,
            };

            if let Some(target_frame_time) = info.target_frame_time {
                self.runtime.timer.sleep_enough(target_frame_time);
            } else {
                self.runtime.control_flow.idle();
            }
        } // loop

        self.app.exit(&mut self.runtime).await;
    }

    pub fn run(&mut self) {
        let async_runtime = AsyncRuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        async_runtime.block_on(self.run_async());
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
    pub custom_receiver: EventReceiver,
}

impl EventQueue {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            events: vec![],
            custom_receiver: EventReceiver::new()?,
        })
    }

    pub fn add(&mut self, event: impl IntoEvent) {
        self.events.push(event.into_event());
    }

    pub fn drain(&mut self) -> Drain<'_, Event> {
        self.events.drain(..)
    }

    pub fn populate_events_from_custom(&mut self) -> Result<(), AbsorbError> {
        loop {
            match self.custom_receiver.try_recv() {
                Ok(value) => self.events.push(value),
                Err(AbsorbError::WouldBlock) => return Ok(()),
                Err(error) => return Err(error),
            }
        }
    }

    pub fn populate_from_cli(
        &mut self,
        wayland: &Wayland,
        cli: &IpcServer<DaemonCommand>,
    ) -> Result<(), RecvError> {
        let command = match cli.try_recv() {
            Ok(command) => command,
            Err(RecvError::Empty) => return Ok(()),
            Err(error) => return Err(error),
        };

        // TODO(hack3rmann): do actions for the monitors that are not plugged yet
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
                    return Ok(());
                };

                self.add(NewWallpaperEvent {
                    path,
                    ty: WallpaperType::Video,
                    target,
                });
            }
            DaemonCommand::SetImage { path, monitor } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return Ok(());
                };

                self.add(NewWallpaperEvent {
                    path,
                    ty: WallpaperType::Image,
                    target,
                });
            }
            DaemonCommand::SetScene { path, monitor } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return Ok(());
                };

                self.add(NewWallpaperEvent {
                    path,
                    ty: WallpaperType::Scene,
                    target,
                });
            }
            DaemonCommand::Pause { monitor } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return Ok(());
                };

                self.add(WallpaperPauseEvent { target });
            }
        };

        Ok(())
    }
}
