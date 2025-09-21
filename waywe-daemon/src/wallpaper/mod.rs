pub mod scene;

use crate::{
    runtime::{
        gpu::Wgpu,
        wayland::{MonitorId, Wayland},
    },
    wallpaper::scene::wallpaper::PreparedWallpaper,
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
    match ty {
        WallpaperType::Image => {
            use scene::wallpaper::*;

            let mut wallpaper = Wallpaper::new(gpu, &wayland, monitor_id);

            ImageWallpaper {
                path: path.to_owned(),
            }
            .build(&mut wallpaper);

            PreparedWallpaper::prepare(wallpaper)
        }
        WallpaperType::Video => todo!(),
        WallpaperType::Scene => todo!(),
    }
}
