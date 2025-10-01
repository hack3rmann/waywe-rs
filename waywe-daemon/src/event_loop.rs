use crate::{
    app::{NewWallpaperEvent, WallpaperPauseEvent},
    app_layer::{AppLayer, DynAppLayer},
    event::{AbsorbError, Event, EventReceiver, IntoEvent},
    runtime::{
        ControlFlow, Runtime,
        wayland::{MonitorId, Wayland},
    },
    task_pool::TaskPool,
};
use runtime::{
    DaemonCommand, Epoll, IpcSocket, RecvError, WallpaperType, epoll::PolledFds, ipc::Server,
    signals,
};
use rustix::io::Errno;
use std::{
    io,
    os::fd::AsFd as _,
    path::PathBuf,
    sync::{Once, atomic::Ordering, mpsc::TryRecvError},
    time::Duration,
    vec::Drain,
};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use tracing::{debug, error};
use video::RatioI32;

pub struct EventLoop {
    // NOTE(hack3rmann): app should be dropped first to release all the resources from the runtime
    app_layer: DynAppLayer,
    runtime: Runtime,
    event_queue: EventQueue,
    epoll: Epoll,
}

impl EventLoop {
    pub fn new<A: AppLayer>(app_layer: A) -> Self {
        static SIGNALS_ONCE: Once = Once::new();
        SIGNALS_ONCE.call_once(signals::setup);

        let app_layer = DynAppLayer::new(app_layer);

        let event_queue = match EventQueue::new() {
            Ok(queue) => queue,
            Err(error) => panic!("failed to create event queue: {error:?}"),
        };

        let wayland = Wayland::new(event_queue.custom_receiver.make_emitter().unwrap());

        let task_pool = TaskPool::new(event_queue.custom_receiver.make_emitter().unwrap());

        let runtime = Runtime::new(wayland, ControlFlow::Busy, task_pool);

        let fds = [
            runtime.wayland.display.as_fd(),
            runtime.ipc.as_fd(),
            event_queue.custom_receiver.pipe_fd(),
        ];

        let epoll = match Epoll::new(fds) {
            Ok(epoll) => epoll,
            Err(error) => panic!("failed to create epoll: {error:?}"),
        };

        Self {
            runtime,
            app_layer,
            event_queue,
            epoll,
        }
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

            self.runtime.task_pool.erase_finished();
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
                self.app_layer
                    .handle_event(&mut self.runtime, &mut event)
                    .await;
            }

            let info = match self.app_layer.frame(&mut self.runtime).await {
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

        self.app_layer.exit(&mut self.runtime).await;
    }

    pub fn run(&mut self) {
        let async_runtime = AsyncRuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        async_runtime.block_on(self.run_async());
    }
}

#[derive(Debug)]
pub enum WallpaperTarget {
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
        cli: &IpcSocket<Server, DaemonCommand>,
    ) -> Result<(), RecvError> {
        let command = match cli.try_recv() {
            Ok(command) => command,
            Err(RecvError::Empty) => return Ok(()),
            Err(error) => return Err(error),
        };

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
            DaemonCommand::SetScene { monitor } => {
                let Some(target) = get_target(monitor.as_deref()) else {
                    return Ok(());
                };

                self.add(NewWallpaperEvent {
                    path: PathBuf::default(),
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

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInfo {
    pub target_frame_time: Option<Duration>,
}

impl FrameInfo {
    pub const MAX_FPS: Duration = RatioI32::new(1, 60).unwrap().to_duration_seconds();

    pub const fn new_60_fps() -> Self {
        Self {
            target_frame_time: Some(Self::MAX_FPS),
        }
    }

    pub fn min_or_60_fps(self, other: Self) -> Self {
        match (self.target_frame_time, other.target_frame_time) {
            (Some(time1), Some(time2)) => Self {
                target_frame_time: Some(time1.min(time2).min(Self::MAX_FPS)),
            },
            (Some(time), None) | (None, Some(time)) => Self {
                target_frame_time: Some(time.min(Self::MAX_FPS)),
            },
            (None, None) => Self {
                target_frame_time: None,
            },
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum FrameError {
    #[error("event loop stop requested")]
    StopRequested,
    #[error("frame skipped due to error")]
    Skip,
    #[error("no work to do")]
    NoWorkToDo,
}
