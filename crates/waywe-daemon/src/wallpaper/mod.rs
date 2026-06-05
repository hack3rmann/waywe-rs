pub mod optimized;
pub mod test_scene;
pub mod transition;

use crate::wallpaper::optimized::{
    OptimizedWallpaper,
    image::{Color, ImageWallpaper},
    video::VideoWallpaper,
};
use glam::UVec2;
use std::{path::Path, sync::Arc};
use test_scene::SceneTestWallpaper;
use waywe_ipc::WallpaperType;
use waywe_runtime::{
    frame::FrameInfo,
    gpu::Wgpu,
    wayland::{MonitorId, Wayland},
};
use waywe_scene::wallpaper::{
    PreparedWallpaper, Wallpaper as SceneWallpaper, WallpaperBuilder as _,
};

pub fn create(
    gpu: Arc<Wgpu>,
    wayland: Arc<Wayland>,
    path: &Path,
    ty: WallpaperType,
    monitor_id: MonitorId,
) -> OptimizedWallpaper {
    let monitor_size = {
        let surfaces = gpu.surfaces.read().unwrap();
        let surface = &surfaces[&monitor_id];
        UVec2::new(surface.config.width, surface.config.height)
    };

    let config = WallpaperConfig {
        surface_size: monitor_size,
        surface_format: gpu.surfaces.read().unwrap()[&monitor_id].format,
    };

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
        WallpaperType::Scene => {
            let mut wallpaper = SceneWallpaper::new(gpu, &wayland, monitor_id);
            SceneTestWallpaper.build(&mut wallpaper);
            OptimizedWallpaper::Scene(PreparedWallpaper::prepare(wallpaper))
        }
        WallpaperType::Video => {
            let wallpaper = VideoWallpaper::new(path, &gpu, config).unwrap();
            OptimizedWallpaper::Video(wallpaper)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WallpaperConfig {
    pub surface_size: UVec2,
    pub surface_format: wgpu::TextureFormat,
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
