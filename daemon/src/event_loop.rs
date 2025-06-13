use crate::runtime::{
    ControlFlow, Runtime,
    wayland::{ClientState, Wayland},
};
use runtime::{
    DaemonCommand, RecvError, WallpaperType,
    ipc::RecvMode,
    profile::{SetupProfile, SetupProfileError},
    signals,
};
use std::{
    io::ErrorKind,
    path::PathBuf,
    sync::{Once, atomic::Ordering},
    time::Duration,
};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use tracing::{debug, error, info};
use video::RatioI32;

pub struct EventLoop<A> {
    runtime: Runtime,
    event_queue: EventQueue,
    app: A,
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
        let mut event_queue = EventQueue::default();

        match SetupProfile::read() {
            Ok(profile) => {
                debug!("found profile config");

                event_queue.events.push(Event::NewWallpaper {
                    path: profile.path.into_owned(),
                    ty: profile.wallpaper_type,
                });
            }
            Err(SetupProfileError::Io(error)) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => info!(?error, "can not read setup profile config"),
        }

        let control_flow = if event_queue.events.is_empty() {
            ControlFlow::Idle
        } else {
            ControlFlow::Busy
        };

        let runtime = Runtime::new(wayland, control_flow);

        Self {
            runtime,
            app,
            event_queue,
        }
    }

    pub fn run(&mut self) {
        let async_runtime = AsyncRuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        async_runtime.block_on(async {
            self.runtime.timer.mark_event_loop_start_time();

            'event_loop: loop {
                self.runtime.timer.mark_frame_start();

                if signals::SHOULD_EXIT.load(Ordering::Relaxed) {
                    debug!("caught stop signal");
                    break self.runtime.control_flow.stop();
                }

                let recv_mode = match self.runtime.control_flow {
                    ControlFlow::Busy => RecvMode::NonBlocking,
                    ControlFlow::Idle => {
                        debug!("daemon is waiting for incoming requests");
                        RecvMode::Blocking
                    }
                    ControlFlow::ShouldStop => {
                        debug!("shutdowning daemon");
                        break 'event_loop;
                    }
                };

                self.runtime.timer.mark_block_start();

                match self.runtime.ipc.socket.recv(recv_mode) {
                    Ok(events) => {
                        debug!(n_events = events.len(), "cli commands received");

                        self.event_queue
                            .events
                            .extend(events.into_iter().map(|command| match command {
                                DaemonCommand::SetVideo { path } => Event::NewWallpaper {
                                    path,
                                    ty: WallpaperType::Video,
                                },
                                DaemonCommand::SetImage { path } => Event::NewWallpaper {
                                    path,
                                    ty: WallpaperType::Image,
                                },
                            }));

                        self.runtime.timer.mark_wallpaper_start_time();
                    }
                    Err(RecvError::Empty) => {}
                    Err(error) => {
                        error!(?error, "failed to recv from waywe-cli");
                    }
                }

                self.runtime.timer.mark_block_end();

                for event in self.event_queue.events.drain(..) {
                    let Event::NewWallpaper { path, ty } = &event;

                    if let Err(error) = SetupProfile::new(path, *ty).store() {
                        error!(?error, "failed to save runtime profile");
                    }

                    self.app.process_event(&mut self.runtime, event).await;
                }

                let info = match self.app.frame(&mut self.runtime).await {
                    Ok(info) => info,
                    Err(FrameError::StopRequested) => break 'event_loop,
                    Err(FrameError::Skip) => continue 'event_loop,
                };

                // that modifies `client_state`
                self.runtime.wayland.display.roundtrip(
                    self.runtime.wayland.main_queue.as_mut(),
                    self.runtime.wayland.client_state.as_ref(),
                );

                if let Some(target_frame_time) = info.target_frame_time {
                    self.runtime.timer.sleep_enough(target_frame_time);
                } else {
                    self.runtime.control_flow.idle();
                }

                self.event_queue
                    .populate_from_wayland_client_state(&self.runtime.wayland.client_state);
            }
        });
    }
}

#[derive(Debug)]
pub enum Event {
    NewWallpaper { path: PathBuf, ty: WallpaperType },
}

#[derive(Debug, Default)]
pub struct EventQueue {
    pub events: Vec<Event>,
}

impl EventQueue {
    pub fn populate_from_wayland_client_state(&mut self, _state: &ClientState) {
        // nothing
    }
}

pub trait App {
    fn process_event(
        &mut self,
        runtime: &mut Runtime,
        event: Event,
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
}
