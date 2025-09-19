pub mod image;
pub mod image_pipeline;
pub mod scene;
pub mod transition;
pub mod video;
pub mod video_pipeline;

use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{gpu::Wgpu, wayland::MonitorId, Runtime, RuntimeFeatures},
    wallpaper::scene::{test_scene::SceneTestWallpaper, Startup},
};
use glam::UVec2;
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

pub trait OldWallpaper: Any + Send + Sync {
    fn frame(
        &mut self,
        runtime: &Runtime,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError>;

    fn free_frame(&mut self, _runtime: &Runtime) -> Result<FrameInfo, FrameError> {
        Ok(FrameInfo::default())
    }

    fn render_state(&self) -> RenderState {
        RenderState::NeedFrame
    }

    fn required_features() -> RuntimeFeatures
    where
        Self: Sized;
}
static_assertions::assert_obj_safe!(OldWallpaper);

pub type DynWallpaper = Box<dyn OldWallpaper>;

pub trait IntoDynWallpaper {
    fn into_dyn_wallpaper(self) -> DynWallpaper
    where
        Self: Sized;
}

impl<W: OldWallpaper + Sized> IntoDynWallpaper for W {
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
    gpu: &Wgpu,
    monitor_size: UVec2,
    path: &Path,
    ty: WallpaperType,
    monitor_id: MonitorId,
) -> Result<DynWallpaper, WallpaperCreationError> {
    match ty {
        WallpaperType::Video => VideoWallpaper::new(
            gpu,
            monitor_size,
            &pathbuf_into_cstring(path.to_owned()),
            monitor_id,
        )
        .map(IntoDynWallpaper::into_dyn_wallpaper)
        .map_err(WallpaperCreationError::from),
        WallpaperType::Image => ImageWallpaper::new(gpu, monitor_size, path, monitor_id)
            .map(IntoDynWallpaper::into_dyn_wallpaper)
            .map_err(WallpaperCreationError::from),
        WallpaperType::Scene => Ok({
            use scene::wallpaper::{ImageWallpaper, *};
            // FIXME:
            let path = Path::new("/home/hack3rmann/Pictures/Wallpapers/wallhaven-6kx95l_2520x1680.png");
            let builder = ImageWallpaper {
                path: path.to_owned(),
            };
            let mut wallpaper = builder.build(WallpaperBuildConfig { monitor_id });
            wallpaper.world.run_schedule(Startup);
            WallpaperWrapper(wallpaper).into_dyn_wallpaper()
        }),
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
            // FIXME:
            Self::Scene => SceneTestWallpaper::required_features(),
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
