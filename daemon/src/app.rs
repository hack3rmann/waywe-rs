use crate::{
    event::EventEmitter,
    event_loop::{App, Event, FrameError, FrameInfo},
    runtime::Runtime,
    wallpaper::{
        self, DynWallpaper, IntoDynWallpaper, RenderState, RequiredFeaturesExt as _,
        transition::TransitionWallpaper,
    },
};
use std::sync::Arc;

#[derive(Default)]
pub struct VideoApp {
    pub wallpaper: Option<DynWallpaper>,
    pub do_force_frame: bool,
}

impl VideoApp {
    pub fn set_wallpaper(
        &mut self,
        runtime: &Runtime<VideoAppEvent>,
        wallpaper: impl IntoDynWallpaper,
    ) {
        let wallpaper = wallpaper.into_dyn_wallpaper();

        self.wallpaper = Some(match self.wallpaper.take() {
            None => wallpaper,
            Some(from) => TransitionWallpaper::new(runtime, from, wallpaper).into_dyn_wallpaper(),
        });
    }

    pub fn resolve_transitions(&mut self) {
        if let Some(wallpaper) = self.wallpaper.take() {
            self.wallpaper = Some(TransitionWallpaper::try_resolve_any(wallpaper));
        }
    }
}

pub enum VideoAppEvent {
    WallpaperPrepared(DynWallpaper),
    Error(Box<dyn std::error::Error + Send + 'static>),
}

impl App for VideoApp {
    type CustomEvent = VideoAppEvent;

    async fn process_event(
        &mut self,
        runtime: &mut Runtime<Self::CustomEvent>,
        event: Event<Self::CustomEvent>,
    ) {
        match event {
            Event::Custom(VideoAppEvent::WallpaperPrepared(wallpaper)) => {
                runtime.control_flow.busy();
                self.set_wallpaper(runtime, wallpaper);
            }
            Event::Custom(_custom) => {}
            Event::NewWallpaper { path, ty } => {
                runtime.enable(ty.required_features()).await;

                let gpu = Arc::clone(&runtime.wgpu);
                let monitor_size = runtime.wayland.client_state.monitor_size();

                runtime
                    .task_pool
                    .spawn(move |mut emitter: EventEmitter<VideoAppEvent>| {
                        let event = match wallpaper::create(&gpu, monitor_size, &path, ty) {
                            Ok(wallpaper) => VideoAppEvent::WallpaperPrepared(wallpaper),
                            Err(error) => VideoAppEvent::Error(Box::new(error)),
                        };

                        emitter.emit(event).unwrap();
                    });
            }
            Event::ResizeRequested { size } => {
                runtime.wgpu.resize_surface(size);
                self.do_force_frame = true;
            }
        }
    }

    async fn frame(
        &mut self,
        runtime: &mut Runtime<Self::CustomEvent>,
    ) -> Result<FrameInfo, FrameError> {
        self.resolve_transitions();

        let Some(wallpaper) = self.wallpaper.as_mut() else {
            runtime.control_flow.idle();
            return Err(FrameError::NoWorkToDo);
        };

        if !self.do_force_frame && wallpaper.render_state() == RenderState::Done {
            runtime.control_flow.idle();
            return Err(FrameError::NoWorkToDo);
        }

        let surface_texture = runtime.wgpu.surface.get_current_texture().unwrap();
        let surface_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = runtime
            .wgpu
            .device
            .create_command_encoder(&Default::default());
        let result = wallpaper.frame(runtime, &mut encoder, &surface_view);
        _ = runtime.wgpu.queue.submit([encoder.finish()]);

        surface_texture.present();
        self.do_force_frame = false;

        result
    }
}
