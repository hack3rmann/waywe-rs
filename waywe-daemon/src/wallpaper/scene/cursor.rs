use crate::wallpaper::{Wallpaper, scene::plugin::Plugin};
use bevy_ecs::prelude::*;
use glam::UVec2;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper
            .main
            .init_resource::<Cursor>()
            .add_observer(update_cursor_position);
    }
}

#[derive(Clone, Copy, Debug, Event)]
pub struct CursorMoved {
    pub position: UVec2,
}

#[derive(Clone, Copy, Debug, Resource, Default)]
pub struct Cursor {
    pub position: UVec2,
}

pub fn update_cursor_position(moved: Trigger<CursorMoved>, mut cursor: ResMut<Cursor>) {
    cursor.position = moved.position;
}
