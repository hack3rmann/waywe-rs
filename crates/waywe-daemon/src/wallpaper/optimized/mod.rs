pub mod image;
pub mod video;

use crate::wallpaper::optimized::{image::ImageWallpaper, video::VideoWallpaper};
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
            OptimizedWallpaper::Image(wallpaper) => {
                wallpaper.frame(surface, encoder);
                FrameInfo {
                    target_frame_time: None,
                }
            }
            OptimizedWallpaper::Video(wallpaper) => wallpaper.frame(gpu, surface, encoder),
            OptimizedWallpaper::Scene(wallpaper) => wallpaper.frame(surface.clone(), encoder),
        }
    }
}
