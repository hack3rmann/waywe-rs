pub mod image;
pub mod render;
pub mod video;

use super::WallpaperConfig;
use crate::wallpaper::{
    Wallpaper,
    optimized::{image::ImageWallpaper, render::RenderWallpaper, video::VideoWallpaper},
};
use waywe_runtime::{frame::FrameInfo, gpu::Wgpu};

#[expect(clippy::large_enum_variant)]
pub enum OptimizedWallpaper {
    Image(ImageWallpaper),
    Video(VideoWallpaper),
    Scene(RenderWallpaper),
}

impl Wallpaper for OptimizedWallpaper {
    fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig) {
        match self {
            OptimizedWallpaper::Image(wall) => wall.configure(gpu, config),
            OptimizedWallpaper::Video(wall) => wall.configure(gpu, config),
            OptimizedWallpaper::Scene(wall) => wall.configure(gpu, config),
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
            OptimizedWallpaper::Scene(wall) => wall.frame(gpu, surface, encoder),
        }
    }
}
