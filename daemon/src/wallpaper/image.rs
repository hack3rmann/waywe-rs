use super::{RenderState, Wallpaper};
use crate::{
    event_loop::{FrameError, FrameInfo},
    image_pipeline::{ImagePipeline, COLOR_WHITE},
    runtime::{gpu::Wgpu, wayland::MonitorId, Runtime, RuntimeFeatures},
};
use glam::UVec2;
use image::{ImageReader, error::ImageError};
use std::{io, path::Path};
use thiserror::Error;

pub struct ImageWallpaper {
    pipeline: ImagePipeline,
    is_render_done: bool,
}

impl ImageWallpaper {
    pub fn new(
        gpu: &Wgpu,
        monitor_size: UVec2,
        path: impl AsRef<Path>,
        monitor_id: MonitorId,
    ) -> Result<Self, ImageWallpaperCreationError> {
        let path = path.as_ref();
        let reader = ImageReader::open(path)?;
        let image = reader.decode()?.into_rgba8();

        Ok(Self {
            is_render_done: false,
            pipeline: ImagePipeline::new(
                gpu,
                &image,
                // TODO(hack3rmann): let the user decide
                COLOR_WHITE,
                monitor_size,
                monitor_id,
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

    fn render_state(&self) -> RenderState {
        match self.is_render_done {
            false => RenderState::NeedFrame,
            true => RenderState::Done,
        }
    }

    fn frame(
        &mut self,
        _: &Runtime,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError> {
        self.pipeline.render(encoder, surface_view);
        self.is_render_done = true;

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
