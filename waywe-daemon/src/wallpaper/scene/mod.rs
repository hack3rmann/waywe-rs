pub mod render;
pub mod render_test;

use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures, wayland::MonitorId},
    wallpaper::{
        Wallpaper,
        scene::{
            render::{SceneExtract, SceneRender},
            render_test::Mesh,
        },
    },
};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};
use derive_more::Deref;
use for_sure::Almost;
use std::{
    mem,
    result::Result,
    time::{Duration, Instant},
};
use video::RatioI32;

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

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneUpdate;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneStartup;

#[derive(Resource, Deref)]
pub struct MainWorld(pub World);

#[derive(Resource, Default)]
pub struct DummyWorld(pub World);

#[derive(Resource, Clone, Copy)]
pub struct Monitor(pub MonitorId);

pub struct Scene {
    pub startup_done: bool,
    pub world: World,
}

impl Scene {
    pub fn new(monitor_id: MonitorId) -> Self {
        let mut world = World::new();

        let mut update = Schedule::new(SceneUpdate);
        update.add_systems(update_time);

        world.add_schedule(update);
        world.add_schedule(Schedule::new(SceneStartup));

        world.insert_resource(Monitor(monitor_id));
        world.init_resource::<DummyWorld>();
        world.init_resource::<Time>();

        Self {
            world,
            startup_done: false,
        }
    }

    pub fn add_systems<M>(
        &mut self,
        label: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        let mut schedules = self.world.get_resource_or_init::<Schedules>();
        schedules.add_systems(label, systems);
        self
    }

    pub fn update(&mut self) {
        if !self.startup_done {
            self.world.run_schedule(SceneStartup);
            self.startup_done = true;
        }

        self.world.run_schedule(SceneUpdate);
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
    }
}

pub fn update_time(mut time: ResMut<Time>) {
    let now = Instant::now();
    let delta = now.duration_since(time.prev);

    time.delta = delta;
    time.elapsed += delta;
    time.prev = now;
}

pub struct SceneTestWallpaper {
    pub scene: Scene,
}

impl SceneTestWallpaper {
    pub fn new_test(monitor_id: MonitorId) -> Self {
        let mut scene = Scene::new(monitor_id);
        scene.world.spawn(Mesh);

        Self { scene }
    }
}

impl Wallpaper for SceneTestWallpaper {
    fn frame(
        &mut self,
        _: &Runtime,
        _: &mut wgpu::CommandEncoder,
        _: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError> {
        Err(FrameError::NoWorkToDo)
    }

    fn free_frame(&mut self, runtime: &Runtime) -> Result<FrameInfo, FrameError> {
        if Almost::is_nil(&runtime.scene_renderer) {
            return Err(FrameError::Skip);
        }

        let mut renderer = runtime.scene_renderer.write().unwrap();
        renderer.apply_queued();

        self.scene.update();
        self.scene.extract(&mut renderer.world);

        renderer.world.run_schedule(SceneRender);

        Ok(FrameInfo::new_60_fps())
    }

    fn required_features() -> RuntimeFeatures
    where
        Self: Sized,
    {
        RuntimeFeatures::SCENE_RENDERER
    }
}
