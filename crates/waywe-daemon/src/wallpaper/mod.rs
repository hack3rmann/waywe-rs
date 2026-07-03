pub mod optimized;
pub mod package_registry;
pub mod preview;
pub mod transition;

use crate::wallpaper::{
    optimized::{
        OptimizedWallpaper,
        image::{Color, ImageWallpaper},
        render::RenderWallpaper,
        video::VideoWallpaper,
    },
    package_registry::PackageRegistry,
};
use std::{path::Path, sync::Arc, time::Duration};
use waywe_ipc::{WallpaperType, command::DaemonError};
use waywe_runtime::{frame::FrameInfo, gpu::Wgpu};

pub use waywe_runtime::WallpaperConfig;

pub async fn create(
    gpu: Arc<Wgpu>,
    path: &Path,
    ty: WallpaperType,
    config: WallpaperConfig,
    packages: PackageRegistry,
) -> Result<OptimizedWallpaper, DaemonError> {
    Ok(match ty {
        WallpaperType::Image => {
            let image = image::ImageReader::open(path)
                .map_err(DaemonError::from_generic)?
                .decode()
                .map_err(DaemonError::from_generic)?
                .into_rgba8();

            let wallpaper = ImageWallpaper::new(&gpu, &image, Color::BLACK, config);

            OptimizedWallpaper::Image(wallpaper)
        }
        WallpaperType::Video => {
            let wallpaper =
                VideoWallpaper::new(path, &gpu, config).map_err(DaemonError::from_generic)?;

            OptimizedWallpaper::Video(wallpaper)
        }
        WallpaperType::Scene => {
            let wallpaper = RenderWallpaper::load(path, &gpu, config, packages)
                .map_err(DaemonError::from_generic)?;
            OptimizedWallpaper::Scene(wallpaper)
        }
    })
}

pub trait Wallpaper {
    fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig);

    fn frame(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo;

    fn advance_time(&mut self, delta: Duration);
}
