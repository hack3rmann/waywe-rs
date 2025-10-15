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

use std::time::{Duration, Instant};
use waywe_ecs::prelude::*;

/// Time tracking resource.
///
/// This resource tracks the elapsed time and time between frames,
/// which is essential for animations and consistent updates.
#[derive(Resource)]
pub struct Time {
    /// The previous frame's timestamp.
    pub prev: Instant,
    /// Total elapsed time since the start.
    pub elapsed: Duration,
    /// Time elapsed since the last frame.
    pub delta: Duration,
}

impl Time {
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.prev);

        self.delta = delta;
        self.elapsed += delta;
        self.prev = now;
    }
}

impl Default for Time {
    fn default() -> Self {
        Self {
            prev: Instant::now(),
            elapsed: Duration::ZERO,
            delta: Duration::ZERO,
        }
    }
}

/// System to update the time resource.
///
/// This system should be added to the update schedule to keep
/// the time resource current.
pub fn update_time(mut time: ResMut<Time>) {
    time.update();
}
