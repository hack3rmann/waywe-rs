pub mod optimized;
pub mod test_scene;
pub mod transition;

use crate::wallpaper::optimized::{
    OptimizedWallpaper,
    image::{Color, ImageWallpaper},
    render::RenderWallpaper,
    video::VideoWallpaper,
};
use std::{path::Path, sync::Arc};
use waywe_ipc::WallpaperType;
use waywe_runtime::{frame::FrameInfo, gpu::Wgpu};

pub use waywe_runtime::WallpaperConfig;

pub fn create(
    gpu: Arc<Wgpu>,
    path: &Path,
    ty: WallpaperType,
    config: WallpaperConfig,
) -> OptimizedWallpaper {
    match ty {
        WallpaperType::Image => {
            let image = image::ImageReader::open(path)
                .unwrap()
                .decode()
                .unwrap()
                .into_rgba8();
            let wallpaper = ImageWallpaper::new(&gpu, &image, Color::BLACK, config);
            OptimizedWallpaper::Image(wallpaper)
        }
        WallpaperType::Video => {
            let wallpaper = VideoWallpaper::new(path, &gpu, config).unwrap();
            OptimizedWallpaper::Video(wallpaper)
        }
        WallpaperType::Scene => {
            let wallpaper = RenderWallpaper::load(path, &gpu, config);
            OptimizedWallpaper::Scene(wallpaper)
        }
    }
}

pub trait Wallpaper {
    fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig);

    fn frame(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo;
}
