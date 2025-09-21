#![allow(clippy::type_complexity, clippy::too_many_arguments)]

pub mod assets;
pub mod cursor;
pub mod extract;
pub mod image;
pub mod material;
pub mod mesh;
pub mod plugin;
pub mod render;
pub mod subapp;
pub mod time;
pub mod transform;
pub mod video;
pub mod wallpaper;

use crate::{
    event_loop::FrameInfo,
    runtime::wayland::MonitorId,
    wallpaper::scene::{assets::Assets, time::Time, video::Video},
};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use bitflags::bitflags;
use derive_more::{Deref, DerefMut};
use glam::UVec2;
use std::time::Duration;

#[derive(Clone, Copy, Resource, Debug, PartialEq)]
pub enum FrameRateSetting {
    TargetFrameDuration(Duration),
    NoUpdate,
    GuessFromScene,
}

impl FrameRateSetting {
    pub const CAP_TO_60_FPS: Self = Self::TargetFrameDuration(FrameInfo::MAX_FPS);
}

impl Default for FrameRateSetting {
    fn default() -> Self {
        Self::CAP_TO_60_FPS
    }
}

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Update;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Startup;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PostStartup;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PostExtract;

#[derive(Resource, Deref, DerefMut)]
pub struct MainWorld(pub World);

#[derive(Resource, Default)]
pub struct DummyWorld(pub World);

#[derive(Resource, Clone, Copy)]
pub struct Monitor {
    pub id: MonitorId,
    pub size: UVec2,
}

impl Monitor {
    pub const fn aspect_ratio(self) -> f32 {
        self.size.y as f32 / self.size.x as f32
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Default, Eq, PartialOrd, Ord, Hash, Resource)]
    pub struct WallpaperFlags: u32 {
        const STARTUP_DONE = 1;
        const NO_UPDATE = 2;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WallpaperConfig {
    pub framerate: FrameRateSetting,
}

pub fn guess_framerate(videos: Res<Assets<Video>>, mut setting: ResMut<FrameRateSetting>) {
    if !matches!(&*setting, FrameRateSetting::GuessFromScene) {
        return;
    }

    let min_duration = videos
        .iter()
        .map(|(_, video)| video.frame_time_fallback)
        .min()
        .unwrap_or(FrameInfo::MAX_FPS);

    *setting = FrameRateSetting::TargetFrameDuration(min_duration);
}
