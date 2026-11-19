//! Minimal scene package that clears the surface to solid red.

use glam::Vec3;
use waywe_scene::{Scene, clear_screen::ClearColor, prelude::*};

#[derive(Default, Scene)]
pub struct ClearScene;

impl WallpaperBuilder for ClearScene {
    fn build(self, wallpaper: &mut Wallpaper) {
        // DefaultPlugins is required so PostStartup `guess_framerate` has `Assets<Video>`.
        wallpaper.add_plugins(DefaultPlugins);
        wallpaper
            .render
            .insert_resource(ClearColor(Vec3::new(1.0, 0.0, 0.0)));
    }
}
