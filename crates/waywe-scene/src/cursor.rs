//! Cursor tracking and interaction.
//!
//! This module provides cursor tracking functionality for wallpapers,
//! allowing entities to respond to cursor position and movement.
//!
//! # Plugins
//!
//! - [`CursorPlugin`]: Adds cursor functionality to a wallpaper
//!
//! # Core Types
//!
//! - [`Cursor`]: Resource tracking cursor position
//! - [`CursorMoved`]: Event triggered when the cursor moves
//!
//! # Systems
//!
//! - [`update_cursor_position`]: System that updates the cursor position

use crate::{plugin::Plugin, wallpaper::Wallpaper};
use glam::UVec2;
use waywe_ecs::prelude::*;

/// Plugin for cursor functionality.
///
/// Adds systems and resources for tracking cursor position.
pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper
            .main
            .init_resource::<Cursor>()
            .add_observer(update_cursor_position);
    }
}

/// Event triggered when the cursor moves.
#[derive(Clone, Copy, Debug, Event)]
pub struct CursorMoved {
    /// New cursor position in pixels.
    pub position: UVec2,
}

/// Resource tracking cursor position.
#[derive(Clone, Copy, Debug, Resource, Default)]
pub struct Cursor {
    /// Current cursor position in pixels.
    pub position: UVec2,
}

/// Observer system to update cursor position.
///
/// This system is triggered by [`CursorMoved`] events and updates
/// the [`Cursor`] resource.
pub fn update_cursor_position(moved: On<CursorMoved>, mut cursor: ResMut<Cursor>) {
    cursor.position = moved.position;
}
