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
use waywe_uuid::TypeUuid;

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
#[derive(Clone, Copy, Debug, Event, TypeUuid)]
#[uuid = "bd244623-f6cf-42c7-9a22-cdcbfc7adf2e"]
pub struct CursorMoved {
    /// New cursor position in pixels.
    pub position: UVec2,
}

/// Resource tracking cursor position.
#[derive(Clone, Copy, Debug, Resource, TypeUuid, Default)]
#[uuid = "9799dc6e-a5e7-4501-9251-12b9607e7f39"]
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
