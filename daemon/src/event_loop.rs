use crate::{
    event::{AbsorbError, CustomEvent, EventReceiver},
    runtime::{
        ControlFlow, Runtime,
        wayland::{ClientState, Wayland},
    },
    task_pool::TaskPool,
};
use glam::UVec2;
use runtime::{
    DaemonCommand, Epoll, IpcSocket, RecvError, WallpaperType,
    ipc::Server,
    profile::{SetupProfile, SetupProfileError},
    signals,
};
use rustix::io::Errno;
use std::{
    io::{self, ErrorKind},
    os::fd::AsFd as _,
    path::PathBuf,
    sync::{Once, atomic::Ordering},
    time::Duration,
    vec::Drain,
};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use tracing::{debug, error, info, warn};
use video::RatioI32;

pub struct EventLoop<A: App> {
    runtime: Runtime<A::CustomEvent>,
    event_queue: EventQueue<A::CustomEvent>,
    app: A,
    epoll: Epoll,
}

impl<A: App + Default> Default for EventLoop<A> {
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<A: App> EventLoop<A> {
    pub fn new(app: A) -> Self {
        static SIGNALS_ONCE: Once = Once::new();
        SIGNALS_ONCE.call_once(signals::setup);

        let wayland = Wayland::new();

        let mut event_queue = match EventQueue::<A::CustomEvent>::new() {
            Ok(queue) => queue,
            Err(error) => panic!("failed to create event queue: {error:?}"),
        };

        match SetupProfile::read() {
            Ok(profile) => {
                debug!("found profile config");

                event_queue.events.push(Event::NewWallpaper {
                    path: profile.path.into_owned(),
                    ty: profile.wallpaper_type,
                });
            }
            Err(SetupProfileError::Io(error)) if error.kind() == ErrorKind::NotFound => {}
            Err(SetupProfileError::Decode(message)) => {
                warn!(
                    ?message,
                    "could not read setup profile cache, may be caused by outdated files",
                );
            }
            Err(error) => info!(?error, "can not read setup profile config"),
        }

        let control_flow = if event_queue.events.is_empty() {
            ControlFlow::Idle
        } else {
            ControlFlow::Busy
        };

        let task_pool = TaskPool::new(event_queue.custom_receiver.make_emitter().unwrap());

        let runtime = Runtime::new(wayland, control_flow, task_pool);

        let fds = [
            runtime.wayland.display.as_fd(),
            runtime.ipc.socket.as_fd(),
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
            epoll,
        }
    }

    async fn run_async(&mut self) {
        self.runtime.timer.mark_event_loop_start_time();

        'event_loop: loop {
            self.runtime.timer.mark_frame_start();

            match self.runtime.control_flow {
                ControlFlow::Busy => {
                    if signals::SHOULD_EXIT.load(Ordering::Relaxed) {
                        debug!("caught stop signal");
                        break 'event_loop;
                    }
                }
                ControlFlow::Idle => {
                    self.runtime.timer.mark_block_start();

                    match self.epoll.wait(None) {
                        Ok(_polled_fds) => {}
                        Err(Errno::INTR) => {
                            if signals::SHOULD_EXIT.load(Ordering::Relaxed) {
                                debug!("caught stop signal");
                                break 'event_loop;
                            }
                        }
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

            self.runtime.task_pool.erase_finished();

            self.runtime.wayland.display.roundtrip(
                self.runtime.wayland.main_queue.as_mut(),
                self.runtime.wayland.client_state.as_ref(),
            );

            self.event_queue
                .populate_from_wayland_client_state(&self.runtime.wayland.client_state);

            if let Err(error) = self.event_queue.populate_events_from_custom() {
                error!(?error, "failed to populate custom events");
            }

            if let Err(error) = self.event_queue.populate_from_cli(&self.runtime.ipc.socket) {
                error!(?error, "can not recv from waywe-cli");
            }

            for event in self.event_queue.drain() {
                self.app.process_event(&mut self.runtime, event).await;
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
pub enum Event<T> {
    Custom(T),
    NewWallpaper { path: PathBuf, ty: WallpaperType },
    ResizeRequested { size: UVec2 },
}

#[derive(Debug)]
pub struct EventQueue<T> {
    pub events: Vec<Event<T>>,
    pub custom_receiver: EventReceiver<T>,
}

impl<T: CustomEvent> EventQueue<T> {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            events: vec![],
            custom_receiver: EventReceiver::new()?,
        })
    }

    pub fn add(&mut self, event: Event<T>) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Drain<'_, Event<T>> {
        self.events.drain(..)
    }

    pub fn populate_events_from_custom(&mut self) -> Result<(), AbsorbError> {
        loop {
            match self.custom_receiver.try_recv() {
                Ok(value) => self.events.push(Event::Custom(value)),
                Err(AbsorbError::WouldBlock) => return Ok(()),
                Err(error) => return Err(error),
            }
        }
    }

    pub fn populate_from_cli(
        &mut self,
        cli: &IpcSocket<Server, DaemonCommand>,
    ) -> Result<(), RecvError> {
        match cli.try_recv() {
            Ok(command) => {
                let event = match command {
                    DaemonCommand::SetVideo { path } => Event::NewWallpaper {
                        path,
                        ty: WallpaperType::Video,
                    },
                    DaemonCommand::SetImage { path } => Event::NewWallpaper {
                        path,
                        ty: WallpaperType::Image,
                    },
                };

                self.add(event);

                Ok(())
            }
            Err(RecvError::Empty) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub fn populate_from_wayland_client_state(&mut self, state: &ClientState) {
        if state.resize_requested.load(Ordering::Acquire) {
            state.resize_requested.store(false, Ordering::Release);
            self.events.push(Event::ResizeRequested {
                size: state.monitor_size(),
            });
        }
    }
}

pub trait App {
    type CustomEvent: CustomEvent;

    fn process_event(
        &mut self,
        runtime: &mut Runtime<Self::CustomEvent>,
        event: Event<Self::CustomEvent>,
    ) -> impl Future<Output = ()> + Send;

    fn frame(
        &mut self,
        runtime: &mut Runtime<Self::CustomEvent>,
    ) -> impl Future<Output = Result<FrameInfo, FrameError>> + Send;

    fn exit(
        &mut self,
        _runtime: &mut Runtime<Self::CustomEvent>,
    ) -> impl Future<Output = ()> + Send {
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

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("event loop stop requested")]
    StopRequested,
    #[error("frame skipped due to error")]
    Skip,
    #[error("no work to do")]
    NoWorkToDo,
}
