pub mod image;
pub mod video;

use super::WallpaperConfig;
use crate::wallpaper::{
    Wallpaper,
    optimized::{image::ImageWallpaper, video::VideoWallpaper},
};
use waywe_runtime::{frame::FrameInfo, gpu::Wgpu};
use waywe_scene::wallpaper::PreparedWallpaper;

#[expect(clippy::large_enum_variant)]
pub enum OptimizedWallpaper {
    Image(ImageWallpaper),
    Video(VideoWallpaper),
    Scene(PreparedWallpaper),
}

impl OptimizedWallpaper {
    pub fn frame(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo {
        match self {
            OptimizedWallpaper::Image(wallpaper) => wallpaper.frame(gpu, surface, encoder),
            OptimizedWallpaper::Video(wallpaper) => wallpaper.frame(gpu, surface, encoder),
            OptimizedWallpaper::Scene(wallpaper) => wallpaper.frame(surface.clone(), encoder),
        }
    }
}

impl Wallpaper for OptimizedWallpaper {
    fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig) {
        match self {
            OptimizedWallpaper::Image(wall) => wall.configure(gpu, config),
            OptimizedWallpaper::Video(wall) => wall.configure(gpu, config),
            OptimizedWallpaper::Scene(_) => todo!("scene wallpaper configure"),
        }
    }

    fn frame(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo {
        match self {
            OptimizedWallpaper::Image(wall) => wall.frame(gpu, surface, encoder),
            OptimizedWallpaper::Video(wall) => wall.frame(gpu, surface, encoder),
            OptimizedWallpaper::Scene(wall) => wall.frame(surface.clone(), encoder),
        }
    }
}
