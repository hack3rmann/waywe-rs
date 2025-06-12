use std::any::Any;

use crate::{
    event_loop::{App, Event, FrameError, FrameInfo},
    runtime::Runtime,
    wallpaper::{
        DynWallpaper, RequiresFeatures, Wallpaper, image::ImageWallpaper,
        transition::TransitionWallpaper, video::VideoWallpaper,
    },
};
use smallvec::SmallVec;
use tracing::error;

#[derive(Default)]
pub struct VideoApp {
    pub wallpapers: SmallVec<[DynWallpaper; 3]>,
}

impl VideoApp {
    pub fn set_wallpaper(&mut self, runtime: &mut Runtime, wallpaper: impl Wallpaper) {
        match self.wallpapers.len() {
            0 => self.wallpapers.push(Box::new(wallpaper)),
            1 => {
                let from = self.wallpapers.drain(..).next().unwrap();
                self.wallpapers.push(Box::new(TransitionWallpaper::new(
                    runtime,
                    from,
                    Box::new(wallpaper),
                )));
            }
            _ => {}
        }
    }

    pub fn resolve_transitions(&mut self) {
        if self.wallpapers.is_empty() {
            return;
        }

        let first = self.wallpapers.first().unwrap().as_ref() as &dyn Any;

        if !first.is::<TransitionWallpaper>() {
            return;
        }

        let first = self.wallpapers.remove(0) as Box<dyn Any>;

        let transition = first.downcast::<TransitionWallpaper>().unwrap();
        self.wallpapers.insert(0, transition.try_resolve());
    }
}

impl App for VideoApp {
    async fn process_event(&mut self, runtime: &mut Runtime, event: Event) {
        match event {
            Event::NewImage { path } => {
                runtime.enable(ImageWallpaper::REQUIRED_FEATURES).await;

                let wallpaper = match ImageWallpaper::new(runtime, &path) {
                    Ok(wallpaper) => wallpaper,
                    Err(error) => {
                        error!(?error, ?path, "failed to create image wallpaper");
                        return;
                    }
                };

                runtime.control_flow.busy();
                self.set_wallpaper(runtime, wallpaper);
            }
            Event::NewVideo { path } => {
                runtime.enable(VideoWallpaper::REQUIRED_FEATURES).await;

                let wallpaper = match VideoWallpaper::new(runtime, &path) {
                    Ok(wallpaper) => wallpaper,
                    Err(error) => {
                        error!(?error, ?path, "failed to create video wallpaper");
                        return;
                    }
                };

                runtime.control_flow.busy();
                self.set_wallpaper(runtime, wallpaper);
            }
        }
    }

    async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        self.resolve_transitions();

        let Some(generator) = self.wallpapers.first_mut() else {
            runtime.control_flow.idle();
            return Err(FrameError::Skip);
        };

        let surface_texture = runtime.wgpu.surface.get_current_texture().unwrap();
        let surface_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = runtime
            .wgpu
            .device
            .create_command_encoder(&Default::default());
        let result = generator.frame(runtime, &mut encoder, &surface_view);
        _ = runtime.wgpu.queue.submit([encoder.finish()]);

        surface_texture.present();

        result
    }
}
