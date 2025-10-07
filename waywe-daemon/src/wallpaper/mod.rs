pub mod test_scene;
pub mod transition;

use std::{path::Path, sync::Arc};
use test_scene::SceneTestWallpaper;
use waywe_default_wallpapers::{ImageWallpaper, VideoWallpaper};
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
) -> PreparedWallpaper {
    let mut wallpaper = Wallpaper::new(gpu, &wayland, monitor_id);

    match ty {
        WallpaperType::Image => {
            ImageWallpaper {
                path: path.to_owned(),
            }
            .build(&mut wallpaper);
        }
        WallpaperType::Scene => {
            SceneTestWallpaper.build(&mut wallpaper);
        }
        WallpaperType::Video => {
            VideoWallpaper {
                path: path.to_owned(),
            }
            .build(&mut wallpaper);
        }
    }

    PreparedWallpaper::prepare(wallpaper)
}
