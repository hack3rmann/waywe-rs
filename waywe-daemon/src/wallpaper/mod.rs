pub mod default;
pub mod scene;
pub mod transition;

use crate::{
    runtime::{
        gpu::Wgpu,
        wayland::{MonitorId, Wayland},
    },
    wallpaper::{
        default::{image::ImageWallpaper, test::SceneTestWallpaper, video::VideoWallpaper},
        scene::wallpaper::{PreparedWallpaper, Wallpaper, WallpaperBuilder as _},
    },
};
use runtime::WallpaperType;
use std::{path::Path, sync::Arc};

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
