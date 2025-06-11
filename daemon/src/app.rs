use crate::{
    event_loop::{App, Event, FrameError, FrameInfo},
    runtime::Runtime,
    wallpaper::{RequiresFeatures, Wallpaper, image::ImageWallpaper, video::VideoWallpaper},
};
use tracing::error;

#[derive(Default)]
pub struct VideoApp {
    pub wallpaper: Option<Box<dyn Wallpaper>>,
}

impl VideoApp {
    pub fn set_wallpaper(&mut self, wallpaper: impl Wallpaper) {
        self.wallpaper = Some(Box::new(wallpaper));
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
                self.set_wallpaper(wallpaper);
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
                self.set_wallpaper(wallpaper);
            }
        }
    }

    async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        let Some(generator) = self.wallpaper.as_mut() else {
            runtime.control_flow.idle();
            return Err(FrameError::Skip);
        };

        generator.frame(runtime)
    }
}
