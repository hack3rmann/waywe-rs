use crate::{
    app::NewWallpaperEvent,
    event::{AbsorbError, Event, EventHandler, EventReceiver, IntoEvent},
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
    io::{self},
    os::fd::AsFd as _,
    sync::{Once, atomic::Ordering},
    time::Duration,
    vec::Drain,
};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use tracing::{debug, error};
use video::RatioI32;

pub struct EventLoop<A: App> {
    runtime: Runtime,
    event_queue: EventQueue,
    event_handler: EventHandler<A>,
    app: A,
    epoll: Epoll,
}

impl<A: App + Default> Default for EventLoop<A> {
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<A: App> EventLoop<A> {
    pub fn new(mut app: A) -> Self {
        static SIGNALS_ONCE: Once = Once::new();
        SIGNALS_ONCE.call_once(signals::setup);

        let mut event_handler = EventHandler::default();
        app.populate_handler(&mut event_handler);

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
            app,
            event_queue,
            event_handler,
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

            self.runtime.wayland.display.roundtrip(
                self.runtime.wayland.main_queue.as_mut(),
                self.runtime.wayland.client_state.as_ref(),
            );

            if let Err(error) = self.event_queue.populate_events_from_custom() {
                error!(?error, "failed to populate custom events");
            }

            if let Err(error) = self
                .event_queue
                .populate_from_cli(&self.runtime.wayland, &self.runtime.ipc)
            {
                error!(?error, "can not recv from waywe-cli");
            }

            for event in self.event_queue.drain() {
                self.event_handler
                    .handle(&mut self.app, &mut self.runtime, event)
                    .await;
            }

            self.runtime.timer.mark_wallpaper_start_time();

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
        let async_runtime = AsyncRuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        async_runtime.block_on(self.run_async());
    }
}

#[derive(Debug)]
pub enum SetWallpaper {
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
        match cli.try_recv() {
            Ok(command) => {
                let (path, monitor, ty) = match command {
                    DaemonCommand::SetVideo { path, monitor } => {
                        (path, monitor, WallpaperType::Video)
                    }
                    DaemonCommand::SetImage { path, monitor } => {
                        (path, monitor, WallpaperType::Image)
                    }
                };

                let monitor_id = monitor
                    .as_ref()
                    .and_then(|name| wayland.client_state.monitor_id(name));

                let set = monitor_id
                    .map(SetWallpaper::ForMonitor)
                    .unwrap_or(SetWallpaper::ForAll);

                self.add(NewWallpaperEvent { path, ty, set });

                Ok(())
            }
            Err(RecvError::Empty) => Ok(()),
            Err(error) => Err(error),
        }
    }
}

pub trait App: 'static {
    fn populate_handler(&mut self, handler: &mut EventHandler<Self>)
    where
        Self: Sized;

    fn frame(
        &mut self,
        runtime: &mut Runtime,
    ) -> impl Future<Output = Result<FrameInfo, FrameError>> + Send;

    fn exit(&mut self, _runtime: &mut Runtime) -> impl Future<Output = ()> + Send {
        async {}
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInfo {
    pub target_frame_time: Option<Duration>,
}

impl FrameInfo {
    pub fn best_with_60_fps(self, other: Self) -> Self {
        const MAX_FPS: Duration = RatioI32::new(1, 60).unwrap().to_duration_seconds();

        match (self.target_frame_time, other.target_frame_time) {
            (Some(time1), Some(time2)) => Self {
                target_frame_time: Some(time1.min(time2).min(MAX_FPS)),
            },
            (Some(time), None) | (None, Some(time)) => Self {
                target_frame_time: Some(time.min(MAX_FPS)),
            },
            (None, None) => Self {
                target_frame_time: Some(MAX_FPS),
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
