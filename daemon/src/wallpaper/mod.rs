pub mod image;
pub mod transition;
pub mod video;
pub mod interpolation;

use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures},
};
use image::{ImageWallpaper, ImageWallpaperCreationError};
use runtime::WallpaperType;
use std::{any::Any, path::Path};
use thiserror::Error;
use transmute_extra::pathbuf_into_cstring;
use video::{VideoWallpaper, VideoWallpaperCreationError};

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum RenderState {
    #[default]
    NeedFrame,
    Done,
}

pub trait Wallpaper: Any + Send + Sync {
    fn frame(
        &mut self,
        runtime: &Runtime,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError>;

    fn render_state(&self) -> RenderState {
        RenderState::NeedFrame
    }

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
        WallpaperType::Video => {
            VideoWallpaper::new(runtime, &pathbuf_into_cstring(path.to_owned()))
                .map(IntoDynWallpaper::into_dyn_wallpaper)
                .map_err(WallpaperCreationError::from)
        }
        WallpaperType::Image => ImageWallpaper::new(runtime, path)
            .map(IntoDynWallpaper::into_dyn_wallpaper)
            .map_err(WallpaperCreationError::from),
    }
}

pub trait RequiredFeaturesExt {
    fn required_features(self) -> RuntimeFeatures;
}

impl RequiredFeaturesExt for WallpaperType {
    fn required_features(self) -> RuntimeFeatures {
        match self {
            Self::Video => VideoWallpaper::required_features(),
            Self::Image => ImageWallpaper::required_features(),
        }
    }
}

#[derive(Debug, Error)]
pub enum WallpaperCreationError {
    #[error(transparent)]
    Image(#[from] ImageWallpaperCreationError),
    #[error(transparent)]
    Video(#[from] VideoWallpaperCreationError),
}
