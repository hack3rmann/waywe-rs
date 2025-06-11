use crate::runtime::{
    ControlFlow, Runtime,
    wayland::{ClientState, Compositor, LayerShell, LayerSurface, Surface, WLR_NAMESPACE, Wayland},
};
use glam::UVec2;
use runtime::{ipc::RecvMode, signals, DaemonCommand, RecvError};
use tracing::debug;
use std::{
    ffi::CString,
    sync::{atomic::Ordering::{self, Relaxed}, Once},
    thread,
    time::Duration,
};
use thiserror::Error;
use tokio::runtime::Builder as AsyncRuntimeBuilder;
use video::RatioI32;
use wayland_client::{
    interface::{
        WlCompositorCreateSurfaceRequest, WlSurfaceCommitRequest, WlSurfaceSetBufferScaleRequest,
        ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellLayer, ZwlrLayerSurfaceAnchor,
        ZwlrLayerSurfaceKeyboardInteractivity, ZwlrLayerSurfaceSetAnchorRequest,
        ZwlrLayerSurfaceSetExclusiveZoneRequest, ZwlrLayerSurfaceSetKeyboardInteractivityRequest,
        ZwlrLayerSurfaceSetMarginRequest,
    },
    sys::{object::WlObjectHandle, wire::WlStackMessageBuffer},
};

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

        let mut wayland = Wayland::new();
        let mut buf = WlStackMessageBuffer::new();

        let registry = wayland
            .display
            .create_registry(&mut buf, wayland.main_queue.as_mut().storage_mut());

        wayland
            .display
            .roundtrip(wayland.main_queue.as_mut(), wayland.client_state.as_ref());

        let compositor = registry
            .bind::<Compositor>(&mut buf, wayland.main_queue.as_mut().storage_mut())
            .unwrap();

        let layer_shell = registry
            .bind::<LayerShell>(&mut buf, wayland.main_queue.as_mut().storage_mut())
            .unwrap();

        let surface: WlObjectHandle<Surface> = compositor.create_object(
            &mut buf,
            wayland.main_queue.as_mut().storage_mut(),
            WlCompositorCreateSurfaceRequest,
        );

        let layer_surface: WlObjectHandle<LayerSurface> = layer_shell.create_object(
            &mut buf,
            wayland.main_queue.as_mut().storage_mut(),
            ZwlrLayerShellGetLayerSurfaceRequest {
                surface: surface.id(),
                output: None,
                layer: ZwlrLayerShellLayer::Background,
                namespace: WLR_NAMESPACE,
            },
        );

        layer_surface.request(
            &mut buf,
            &wayland.main_queue.as_ref().storage(),
            ZwlrLayerSurfaceSetAnchorRequest {
                anchor: ZwlrLayerSurfaceAnchor::all(),
            },
        );

        layer_surface.request(
            &mut buf,
            &wayland.main_queue.as_ref().storage(),
            ZwlrLayerSurfaceSetExclusiveZoneRequest { zone: -1 },
        );

        layer_surface.request(
            &mut buf,
            &wayland.main_queue.as_ref().storage(),
            ZwlrLayerSurfaceSetMarginRequest {
                top: 0,
                right: 0,
                bottom: 0,
                left: 0,
            },
        );

        layer_surface.request(
            &mut buf,
            &wayland.main_queue.as_ref().storage(),
            ZwlrLayerSurfaceSetKeyboardInteractivityRequest {
                keyboard_interactivity: ZwlrLayerSurfaceKeyboardInteractivity::None,
            },
        );

        surface.request(
            &mut buf,
            &wayland.main_queue.as_ref().storage(),
            WlSurfaceSetBufferScaleRequest { scale: 1 },
        );

        surface.request(
            &mut buf,
            &wayland.main_queue.as_ref().storage(),
            WlSurfaceCommitRequest,
        );

        wayland
            .display
            .roundtrip(wayland.main_queue.as_mut(), wayland.client_state.as_ref());

        let screen_size = UVec2::new(
            wayland.client_state.monitor_width.load(Relaxed),
            wayland.client_state.monitor_height.load(Relaxed),
        );

        assert_ne!(screen_size.x, 0);
        assert_ne!(screen_size.y, 0);

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
                    },
                    ControlFlow::ShouldStop => {
                        debug!("shutdowning daemon");
                        break 'event_loop;
                    }
                };

                match self.runtime.ipc.socket.recv(recv_mode) {
                    Ok(events) => {
                        self.event_queue.events.extend(events.into_iter().map(
                            |DaemonCommand::SetVideo { path }| Event::UpdateWallpaper { path },
                        ))
                    }
                    Err(RecvError::Empty) => {},
                    Err(error) => {
                        tracing::error!(?error, "failed to recv from waywe-cli");
                    }
                }

                for event in self.event_queue.events.drain(..) {
                    self.app.process_event(&mut self.runtime, event).await;
                }

                let info = match self.app.frame(&mut self.runtime).await {
                    Ok(info) => info,
                    Err(FrameError::StopRequested) => break 'event_loop,
                    Err(FrameError::Skip) => continue,
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

                // the modify `event_queue`
                self.event_queue
                    .populate_from_wayland_client_state(&self.runtime.wayland.client_state);
            }
        });
    }
}

#[derive(Debug)]
pub enum Event {
    UpdateWallpaper { path: CString },
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
