//! Time tracking for animations and updates.
//!
//! This module provides time tracking functionality for the wallpaper
//! scene system, allowing for smooth animations and consistent updates.
//!
//! # Core Types
//!
//! - [`Time`]: Resource tracking time information
//!
//! # Systems
//!
//! - [`update_time`]: System that updates the time resource each frame

use bevy_ecs::prelude::*;
use std::time::Duration;

/// Time tracking resource.
///
/// This resource tracks the elapsed time and time between frames,
/// which is essential for animations and consistent updates.
#[derive(Resource)]
pub struct Time {
    /// Total elapsed time since the start.
    pub elapsed: Duration,
    /// Time elapsed since the last frame.
    pub delta: Duration,
}

impl Time {
    pub fn update(&mut self, delta: Duration) {
        self.delta = delta;
        self.elapsed += delta;
    }
}

impl Default for Time {
    fn default() -> Self {
        Self {
            elapsed: Duration::ZERO,
            delta: Duration::ZERO,
        }
    }
}
