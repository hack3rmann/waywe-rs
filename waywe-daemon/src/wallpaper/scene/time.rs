use bevy_ecs::prelude::*;
use std::time::{Duration, Instant};

#[derive(Resource)]
pub struct Time {
    pub prev: Instant,
    pub elapsed: Duration,
    pub delta: Duration,
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

pub fn update_time(mut time: ResMut<Time>) {
    let now = Instant::now();
    let delta = now.duration_since(time.prev);

    time.delta = delta;
    time.elapsed += delta;
    time.prev = now;
}
