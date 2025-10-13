pub mod optimized;
pub mod test_scene;
pub mod transition;

use crate::wallpaper::optimized::{
    OptimizedWallpaper, image::ImageWallpaper, video::VideoWallpaper,
};
use glam::UVec2;
use std::{path::Path, sync::Arc};
use test_scene::SceneTestWallpaper;
use waywe_ipc::WallpaperType;
use waywe_runtime::{
    gpu::Wgpu,
    wayland::{MonitorId, Wayland},
};
use waywe_scene::wallpaper::{PreparedWallpaper, Wallpaper, WallpaperBuilder as _};

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum RenderState {
    #[default]
    NeedFrame,
    Done,
}

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

    match ty {
        WallpaperType::Image => {
            let image = image::ImageReader::open(path)
                .unwrap()
                .decode()
                .unwrap()
                .into_rgba8();
            let wallpaper = ImageWallpaper::new(&gpu, &image, 0, monitor_size, monitor_id);
            OptimizedWallpaper::Image(wallpaper)
        }
        WallpaperType::Scene => {
            let mut wallpaper = Wallpaper::new(gpu, &wayland, monitor_id);
            SceneTestWallpaper.build(&mut wallpaper);
            OptimizedWallpaper::Scene(PreparedWallpaper::prepare(wallpaper))
        }
        WallpaperType::Video => {
            let wallpaper = VideoWallpaper::new(path, &gpu, monitor_size, monitor_id).unwrap();
            OptimizedWallpaper::Video(wallpaper)
        }
    }
}
