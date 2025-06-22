use crate::{
    event::{AbsorbError, CustomEvent, EventReceiver},
    runtime::{
        ControlFlow, Runtime,
        wayland::{ClientState, Wayland},
    },
};
use glam::UVec2;
use runtime::{
    DaemonCommand, Epoll, RecvError, WallpaperType,
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
};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use tracing::{debug, error, info, warn};
use video::RatioI32;

pub struct EventLoop<A: App> {
    runtime: Runtime,
    event_queue: EventQueue<A::UserEvent>,
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
        static TRACING_ONCE: Once = Once::new();
        TRACING_ONCE.call_once(tracing_subscriber::fmt::init);

        static SIGNALS_ONCE: Once = Once::new();
        SIGNALS_ONCE.call_once(signals::setup);

        let wayland = Wayland::new();

        let mut event_queue = match EventQueue::<A::UserEvent>::new() {
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

        let runtime = Runtime::new(wayland, control_flow);

        event_queue.events.clear();

        let fds = [
            runtime.wayland.display.as_fd(),
            runtime.ipc.socket.as_fd(),
            event_queue.custom_receiver.pipe_fd(),
        ];

        let epoll = match Epoll::new(fds, None) {
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

    pub(crate) fn populate_events_from_wayland(&mut self) {
        // that modifies `client_state`
        self.runtime.wayland.display.roundtrip(
            self.runtime.wayland.main_queue.as_mut(),
            self.runtime.wayland.client_state.as_ref(),
        );

        self.event_queue
            .populate_from_wayland_client_state(&self.runtime.wayland.client_state);
    }

    pub fn run(&mut self) {
        let async_runtime = AsyncRuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        async_runtime.block_on(async {
            self.runtime.timer.mark_event_loop_start_time();

            'event_loop: loop {
                self.runtime.timer.mark_frame_start();

                if signals::SHOULD_EXIT.load(Ordering::Relaxed) {
                    debug!("caught stop signal");
                    self.runtime.control_flow.stop();
                }

                match self.runtime.control_flow {
                    ControlFlow::Busy => {}
                    ControlFlow::Idle => match self.epoll.wait() {
                        Ok(polled_fds) => {
                            if polled_fds.contains(&self.runtime.wayland.display) {
                                self.populate_events_from_wayland();
                            }

                            let custom_count =
                                polled_fds.count_of(self.event_queue.custom_receiver.pipe_fd());

                            if let Err(error) = self.event_queue.collect(custom_count) {
                                error!(?error, "failed to collect custom events");
                            }
                        }
                        Err(Errno::INTR) => {}
                        Err(error) => {
                            error!(?error, "failed to wait on multiple sockets");
                        }
                    },
                    ControlFlow::ShouldStop => {
                        debug!("shutting down daemon");
                        break 'event_loop;
                    }
                }

                self.runtime.timer.mark_block_start();

                match self.runtime.ipc.socket.nonblocking_recv() {
                    Ok(command) => {
                        self.event_queue.events.push(match command {
                            DaemonCommand::SetVideo { path } => Event::NewWallpaper {
                                path,
                                ty: WallpaperType::Video,
                            },
                            DaemonCommand::SetImage { path } => Event::NewWallpaper {
                                path,
                                ty: WallpaperType::Image,
                            },
                        });

                        self.runtime.timer.mark_wallpaper_start_time();
                    }
                    Err(RecvError::Timeout) => unreachable!(),
                    Err(RecvError::Empty) => {}
                    Err(error) => {
                        error!(?error, "failed to recv from waywe-cli");
                    }
                }

                self.runtime.timer.mark_block_end();

                for event in self.event_queue.events.drain(..) {
                    if let Event::NewWallpaper { path, ty } = &event {
                        let size = self.runtime.wayland.client_state.monitor_size();

                        if let Err(error) = SetupProfile::new(path, *ty, size).store() {
                            error!(?error, "failed to save runtime profile");
                        }
                    }

                    self.app.process_event(&mut self.runtime, event).await;
                }

                let info = match self.app.frame(&mut self.runtime).await {
                    Ok(info) => info,
                    Err(FrameError::StopRequested) => break 'event_loop,
                    Err(FrameError::Skip | FrameError::NoWorkToDo) => {
                        self.populate_events_from_wayland();
                        continue 'event_loop;
                    }
                };

                self.populate_events_from_wayland();

                if let Some(target_frame_time) = info.target_frame_time {
                    self.runtime.timer.sleep_enough(target_frame_time);
                } else {
                    self.runtime.control_flow.idle();
                }
            }
        });
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

    pub fn collect(&mut self, count: usize) -> Result<(), AbsorbError> {
        self.events.reserve(count);

        for _ in 0..count {
            let event = self.custom_receiver.recv()?;
            self.events.push(Event::Custom(event));
        }

        Ok(())
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
    type UserEvent: CustomEvent;

    fn process_event(
        &mut self,
        runtime: &mut Runtime,
        event: Event<Self::UserEvent>,
    ) -> impl Future<Output = ()> + Send;

    fn frame(
        &mut self,
        runtime: &mut Runtime,
    ) -> impl Future<Output = Result<FrameInfo, FrameError>> + Send;
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
