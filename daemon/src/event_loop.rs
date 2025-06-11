use crate::runtime::{
    ControlFlow, Runtime,
    wayland::{ClientState, Wayland},
};
use runtime::{DaemonCommand, RecvError, ipc::RecvMode, signals};
use std::{
    ffi::CString,
    path::PathBuf,
    sync::{Once, atomic::Ordering},
    thread,
    time::Duration,
};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use tracing::debug;
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
        let event_queue = EventQueue::default();

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
                                DaemonCommand::SetVideo { path } => Event::NewVideo { path },
                                DaemonCommand::SetImage { path } => Event::NewImage { path },
                            }))
                    }
                    Err(RecvError::Empty) => {}
                    Err(error) => {
                        tracing::error!(?error, "failed to recv from waywe-cli");
                    }
                }

                self.runtime.timer.mark_block_end();

                for event in self.event_queue.events.drain(..) {
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

                let render_time = self.runtime.timer.current_frame_duration();
                let sleep_time = info.target_frame_time.saturating_sub(render_time);

                // TODO(hack3rmann): skip a frame if `time_borrow >= target_frame_time`
                if !sleep_time.is_zero() {
                    let unborrowed_time = sleep_time.saturating_sub(self.runtime.timer.time_borrow);

                    if !unborrowed_time.is_zero() {
                        self.runtime.timer.time_borrow = Duration::default();
                        thread::sleep(unborrowed_time);
                    } else {
                        self.runtime.timer.time_borrow -= sleep_time;
                        tracing::warn!(
                            ?render_time,
                            "speeding up current frame due to time borrow"
                        );
                    }
                // ignore first frame lag
                } else if !self.runtime.timer.is_first_frame() {
                    self.runtime.timer.time_borrow += render_time - info.target_frame_time;
                    tracing::warn!(?render_time, "frame took too long to prepare");
                }

                self.event_queue
                    .populate_from_wayland_client_state(&self.runtime.wayland.client_state);
            }
        });
    }
}

#[derive(Debug)]
pub enum Event {
    NewVideo { path: CString },
    NewImage { path: PathBuf },
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

#[derive(Clone, Debug)]
pub struct FrameInfo {
    pub target_frame_time: Duration,
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self {
            target_frame_time: RatioI32::new(1, 60).unwrap().to_duration_seconds(),
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
