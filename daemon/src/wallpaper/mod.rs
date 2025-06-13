pub mod image;
pub mod transition;
pub mod video;

use crate::{
    event_loop::{FrameError, FrameInfo, WallpaperType},
    runtime::{Runtime, RuntimeFeatures},
};
use image::{ImageWallpaper, ImageWallpaperCreationError};
use transmute_extra::pathbuf_into_cstring;
use std::{any::Any, path::Path};
use thiserror::Error;
use video::{VideoWallpaper, VideoWallpaperCreationError};

pub trait Wallpaper: Any + Send + Sync {
    fn frame(
        &mut self,
        runtime: &Runtime,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError>;

    fn required_features() -> RuntimeFeatures
    where
        Self: Sized;
}
static_assertions::assert_obj_safe!(Wallpaper);

pub type DynWallpaper = Box<dyn Wallpaper>;

pub trait IntoDynWallpaper {
    fn into_dyn_wallpaper(self) -> DynWallpaper
    where
        Self: Sized;
}

impl<W: Wallpaper + Sized> IntoDynWallpaper for W {
    fn into_dyn_wallpaper(self) -> DynWallpaper
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl IntoDynWallpaper for DynWallpaper {
    fn into_dyn_wallpaper(self) -> DynWallpaper
    where
        Self: Sized,
    {
        self
    }
}

pub fn create(
    runtime: &mut Runtime,
    path: &Path,
    ty: WallpaperType,
) -> Result<DynWallpaper, WallpaperCreationError> {
    match ty {
        WallpaperType::Video => VideoWallpaper::new(runtime, &pathbuf_into_cstring(path.to_owned()))
            .map(IntoDynWallpaper::into_dyn_wallpaper)
            .map_err(WallpaperCreationError::from),
        WallpaperType::Image => ImageWallpaper::new(runtime, path)
            .map(IntoDynWallpaper::into_dyn_wallpaper)
            .map_err(WallpaperCreationError::from),
    }
}

#[derive(Debug, Error)]
pub enum WallpaperCreationError {
    #[error(transparent)]
    Image(#[from] ImageWallpaperCreationError),
    #[error(transparent)]
    Video(#[from] VideoWallpaperCreationError),
}
