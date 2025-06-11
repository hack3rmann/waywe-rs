use super::{RequiresFeatures, Wallpaper};
use crate::{
    event_loop::{FrameError, FrameInfo},
    image_pipeline::ImagePipeline,
    runtime::{Runtime, RuntimeFeatures},
};
use image::{ImageReader, error::ImageError};
use std::{io, path::Path};
use thiserror::Error;

pub struct ImageWallpaper {
    pipeline: ImagePipeline,
}

impl ImageWallpaper {
    pub fn new(
        runtime: &mut Runtime,
        path: impl AsRef<Path>,
    ) -> Result<Self, ImageWallpaperCreationError> {
        let path = path.as_ref();
        let reader = ImageReader::open(path)?;
        let image = reader.decode()?.into_rgba8();

        Ok(Self {
            pipeline: ImagePipeline::new(
                &mut runtime.wgpu,
                &image,
                runtime.wayland.client_state.monitor_size(),
            ),
        })
    }
}

impl RequiresFeatures for ImageWallpaper {
    const REQUIRED_FEATURES: RuntimeFeatures = RuntimeFeatures::GPU;
}

impl Wallpaper for ImageWallpaper {
    fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        let surface_texture = runtime.wgpu.surface.get_current_texture().unwrap();
        let surface_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = runtime
            .wgpu
            .device
            .create_command_encoder(&Default::default());

        self.pipeline.render(&mut encoder, &surface_view);

        let submission_index = runtime.wgpu.queue.submit([encoder.finish()]);
        _ = runtime
            .wgpu
            .device
            .poll(wgpu::Maintain::wait_for(submission_index));

        surface_texture.present();

        runtime.control_flow.idle();

        Ok(FrameInfo {
            target_frame_time: None,
        })
    }
}

#[derive(Debug, Error)]
pub enum ImageWallpaperCreationError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Decode(#[from] ImageError),
}
