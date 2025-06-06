use crate::runtime::{
    Runtime,
    wayland::{ClientState, Compositor, LayerShell, LayerSurface, Surface, WLR_NAMESPACE, Wayland},
};
use glam::UVec2;
use std::{
    ffi::{CStr, CString},
    sync::{Once, atomic::Ordering::Relaxed},
    thread,
    time::Duration,
};
use thiserror::Error;
use tokio::runtime::{Builder as AsyncRuntimeBuilder, Runtime as AsyncRuntime};
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
    async_runtime: AsyncRuntime,
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

        let async_runtime = AsyncRuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let event_queue = EventQueue::default();

        let runtime = Runtime::new(wayland);

        Self {
            runtime,
            app,
            async_runtime,
            event_queue,
        }
    }

    pub fn run(&mut self) {
        self.async_runtime.block_on(async {
            self.runtime.timer.mark_event_loop_start_time();

            loop {
                self.runtime.timer.mark_frame_start();

                for event in self.event_queue.events.drain(..) {
                    self.app.process_event(&mut self.runtime, event).await;
                }

                let info = match self.app.frame(&mut self.runtime).await {
                    Ok(info) => info,
                    Err(FrameError::StopRequested) => return,
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

pub struct EventQueue {
    pub events: Vec<Event>,
}

impl Default for EventQueue {
    fn default() -> Self {
        const FILE_NAMES: &[&CStr] = &[
            c"/home/hack3rmann/Downloads/Telegram Desktop/845514446/video.mp4",
            c"/home/hack3rmann/Downloads/snowfall-in-forest.3840x2160.mp4",
            c"/home/hack3rmann/Downloads/nailmaster-mato.3840x2160.mp4",
            c"/home/hack3rmann/Downloads/alone-hollow-knight.3840x2160.mp4",
            c"/home/hack3rmann/Downloads/queens-gardens-hollow-knight.3840x2160.mp4",
            c"/home/hack3rmann/Videos/ObsRecordings/2025-05-16 15-51-50.mp4",
            c"/home/hack3rmann/Pictures/Wallpapers/night-sky-purple-moon-clouds-3840x2160.mp4",
            c"/home/hack3rmann/Downloads/sample-1.avi",
            c"http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
        ];

        Self {
            events: vec![Event::UpdateWallpaper {
                path: CString::from(FILE_NAMES[4]),
            }],
        }
    }
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
