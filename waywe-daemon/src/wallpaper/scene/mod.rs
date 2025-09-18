#![allow(clippy::type_complexity, clippy::too_many_arguments)]

pub mod assets;
pub mod image;
pub mod material;
pub mod render;
pub mod render_test;
pub mod sprite;
pub mod transform;

use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures, wayland::MonitorId},
    wallpaper::{
        Wallpaper,
        scene::{
            assets::Assets,
            image::{Image, ImageMaterial, ImagePlugin},
            render::{SceneExtract, SceneRender},
            render_test::{Mesh, MeshMaterial},
            transform::{Transform, TransformPlugin},
        },
    },
};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};
use derive_more::{Deref, DerefMut};
use for_sure::Almost;
use glam::{Quat, Vec2, Vec3};
use std::{
    mem,
    result::Result,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
    thread,
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

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneUpdate;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneStartup;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScenePostExtract;

#[derive(Resource, Deref, DerefMut)]
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
        world.add_schedule(Schedule::new(ScenePostExtract));

        world.insert_resource(Monitor(monitor_id));
        world.init_resource::<DummyWorld>();
        world.init_resource::<Time>();

        let mut this = Self {
            world,
            startup_done: false,
        };

        // FIXME: add default plugins in another way
        this.add_plugin(TransformPlugin);
        this.add_plugin(ImagePlugin);

        this
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

        self.world.run_schedule(ScenePostExtract);
    }

    pub fn add_plugin(&mut self, plugin: impl ScenePlugin) -> &mut Self {
        plugin.init(self);
        self
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

        scene.add_systems(SceneUpdate, Self::rotate_meshes);
        scene.add_systems(SceneStartup, Self::spawn_mesh);

        Self { scene }
    }

    pub fn spawn_mesh(
        mut commands: Commands,
        mut images: ResMut<Assets<Image>>,
        mut materials: ResMut<Assets<ImageMaterial>>,
    ) {
        // FIXME(hack3rmann): use local image
        const PATH: &str = "/home/hack3rmann/Pictures/Wallpapers/All/wallhaven-28kdom.png";
        let image = ::image::ImageReader::open(PATH)
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();

        let image = images.add(Image { image });
        let material = materials.add(ImageMaterial { image });

        commands.spawn((
            Mesh::rect(Vec2::ONE),
            Transform::from_translation(0.1 * Vec3::ONE),
            MeshMaterial(material),
        ));
    }

    pub fn rotate_meshes(mut transforms: Query<&mut Transform, With<Mesh>>, time: Res<Time>) {
        for mut transform in &mut transforms {
            transform.rotation = Quat::from_rotation_z(time.elapsed.as_secs_f32());
        }
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

        {
            let mut renderer = runtime.scene_renderer.write().unwrap();
            renderer.apply_queued();
        }

        static NOT_FIRST_TIME: AtomicBool = AtomicBool::new(false);

        thread::scope(|s| {
            let handle = s.spawn(|| {
                if NOT_FIRST_TIME.fetch_or(true, Relaxed) {
                    let mut renderer = runtime.scene_renderer.write().unwrap();
                    renderer.world.run_schedule(SceneRender);
                }
            });

            self.scene.update();

            handle.join().unwrap();
        });

        let mut renderer = runtime.scene_renderer.write().unwrap();
        self.scene.extract(&mut renderer.world);

        Ok(FrameInfo::new_60_fps())
    }

    fn required_features() -> RuntimeFeatures
    where
        Self: Sized,
    {
        RuntimeFeatures::SCENE_RENDERER
    }
}

pub trait ScenePlugin {
    fn init(self, scene: &mut Scene);
}
