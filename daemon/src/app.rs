use crate::{
    event_loop::{App, Event, FrameError, FrameInfo},
    runtime::Runtime,
    wallpaper::{
        self, transition::TransitionWallpaper, DynWallpaper, IntoDynWallpaper, RenderState, RequiredFeaturesExt as _
    },
};
use tracing::error;

#[derive(Default)]
pub struct VideoApp {
    pub wallpaper: Option<DynWallpaper>,
    pub do_force_frame: bool,
}

impl VideoApp {
    pub fn set_wallpaper(&mut self, runtime: &mut Runtime, wallpaper: impl IntoDynWallpaper) {
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

impl App for VideoApp {
    async fn process_event(&mut self, runtime: &mut Runtime, event: Event) {
        match event {
            Event::NewWallpaper { path, ty } => {
                runtime.enable(ty.required_features()).await;

                let wallpaper = match wallpaper::create(runtime, &path, ty) {
                    Ok(wallpaper) => wallpaper,
                    Err(error) => {
                        error!(?error, ?path, ?ty, "failed to create a wallpaper");
                        return;
                    }
                };

                runtime.control_flow.busy();
                self.set_wallpaper(runtime, wallpaper);
            }
            Event::ResizeRequested { size } => {
                runtime.wgpu.resize_surface(size);
                self.do_force_frame = true;
            }
        }
    }

    async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
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
