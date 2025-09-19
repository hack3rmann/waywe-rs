#![allow(clippy::type_complexity, clippy::too_many_arguments)]

pub mod assets;
pub mod image;
pub mod material;
pub mod mesh;
pub mod render;
pub mod sprite;
pub mod test_scene;
pub mod transform;
pub mod video;

use crate::{
    event_loop::FrameInfo,
    runtime::wayland::MonitorId,
    wallpaper::scene::{
        assets::Assets,
        image::ImagePlugin,
        mesh::MeshPlugin,
        render::SceneExtract,
        transform::TransformPlugin,
        video::{Video, VideoPlugin},
    },
};
use bevy_ecs::{label::DynEq, prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};
use bitflags::bitflags;
use derive_more::{Deref, DerefMut};
use std::{
    any::Any,
    mem,
    time::{Duration, Instant},
};

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
pub struct SceneUpdate;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneStartup;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScenePostStartup;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScenePostExtract;

#[derive(Resource, Deref, DerefMut)]
pub struct MainWorld(pub World);

#[derive(Resource, Default)]
pub struct DummyWorld(pub World);

#[derive(Resource, Clone, Copy)]
pub struct Monitor(pub MonitorId);

bitflags! {
    pub struct SceneFlags: u32 {
        const STARTUP_DONE = 1;
        const NO_UPDATE = 2;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SceneConfig {
    pub framerate: FrameRateSetting,
}

pub struct Scene {
    flags: SceneFlags,
    pub world: World,
}

impl Scene {
    pub fn new(monitor_id: MonitorId) -> Self {
        Self::new_with_config(monitor_id, SceneConfig::default())
    }

    pub fn new_with_config(monitor_id: MonitorId, config: SceneConfig) -> Self {
        let mut world = World::new();

        if !matches!(config.framerate, FrameRateSetting::NoUpdate) {
            let mut update = Schedule::new(SceneUpdate);
            update.add_systems(update_time);
            world.add_schedule(update);
        }
        world.init_resource::<Time>();
        world.insert_resource(config.framerate);

        let mut post_startup = Schedule::new(ScenePostStartup);
        post_startup.add_systems(guess_framerate);

        world.add_schedule(Schedule::new(SceneStartup));
        world.add_schedule(post_startup);
        world.add_schedule(Schedule::new(ScenePostExtract));

        world.insert_resource(Monitor(monitor_id));
        world.init_resource::<DummyWorld>();

        let mut flags = SceneFlags::empty();

        if matches!(config.framerate, FrameRateSetting::NoUpdate) {
            flags |= SceneFlags::NO_UPDATE;
        }

        let mut this = Self { world, flags };

        // FIXME: add default plugins in another way
        this.add_plugin(TransformPlugin);
        this.add_plugin(ImagePlugin);
        this.add_plugin(MeshPlugin);
        this.add_plugin(VideoPlugin);

        this
    }

    pub fn add_systems<M>(
        &mut self,
        label: impl ScheduleLabel + Any + Eq,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        if self.flags.contains(SceneFlags::NO_UPDATE) && label.dyn_eq(&SceneUpdate) {
            return self;
        }

        let mut schedules = self.world.get_resource_or_init::<Schedules>();
        schedules.add_systems(label, systems);
        self
    }

    pub fn update(&mut self) {
        if !self.flags.contains(SceneFlags::STARTUP_DONE) {
            self.world.run_schedule(SceneStartup);
            self.world.run_schedule(ScenePostStartup);
            self.flags |= SceneFlags::STARTUP_DONE;
        }

        if !self.flags.contains(SceneFlags::NO_UPDATE) {
            self.world.run_schedule(SceneUpdate);
        }
    }

    pub fn extract(&mut self, render_world: &mut World) {
        let DummyWorld(temp_world) = self
            .world
            .remove_resource::<DummyWorld>()
            .unwrap_or_default();
        let main_world = mem::replace(&mut self.world, temp_world);
        render_world.insert_resource(MainWorld(main_world));

        render_world.run_schedule(SceneExtract);

        let MainWorld(main_world) = render_world.remove_resource::<MainWorld>().unwrap();
        let temp_world = mem::replace(&mut self.world, main_world);
        self.world.insert_resource(DummyWorld(temp_world));

        self.world.run_schedule(ScenePostExtract);
    }

    pub fn add_plugin(&mut self, plugin: impl ScenePlugin) -> &mut Self {
        plugin.init(self);
        self
    }
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

pub fn update_time(mut time: ResMut<Time>) {
    let now = Instant::now();
    let delta = now.duration_since(time.prev);

    time.delta = delta;
    time.elapsed += delta;
    time.prev = now;
}

pub trait ScenePlugin {
    fn init(self, scene: &mut Scene);
}
