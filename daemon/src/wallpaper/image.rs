use super::Wallpaper;
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

impl Wallpaper for ImageWallpaper {
    fn required_features() -> RuntimeFeatures
    where
        Self: Sized,
    {
        RuntimeFeatures::GPU
    }

    fn frame(
        &mut self,
        _: &Runtime,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError> {
        self.pipeline.render(encoder, surface_view);

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
