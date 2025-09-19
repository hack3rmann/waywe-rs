#![allow(clippy::type_complexity, clippy::too_many_arguments)]

pub mod assets;
pub mod image;
pub mod material;
pub mod mesh;
pub mod render;
pub mod sprite;
pub mod transform;
pub mod video;

use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures, wayland::MonitorId},
    wallpaper::{
        Wallpaper,
        scene::{
            assets::{AssetHandle, Assets},
            image::{Image, ImageMaterial, ImagePlugin},
            mesh::{Mesh, Mesh3d, MeshMaterial, MeshPlugin, Vertex},
            render::{SceneExtract, SceneRender},
            transform::{Transform, TransformPlugin},
            video::{Video, VideoMaterial, VideoPlugin},
        },
    },
};
use bevy_ecs::{label::DynEq, prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};
use bitflags::bitflags;
use derive_more::{Deref, DerefMut};
use for_sure::Almost;
use glam::{Vec2, Vec3};
use smallvec::smallvec;
use std::{
    any::Any,
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

bitflags! {
    pub struct SceneFlags: u32 {
        const STARTUP_DONE = 1;
        const NO_UPDATE = 2;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SceneConfig {
    no_update: bool,
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

        if !config.no_update {
            let mut update = Schedule::new(SceneUpdate);
            update.add_systems(update_time);
            world.add_schedule(update);
        }
        world.init_resource::<Time>();

        world.add_schedule(Schedule::new(SceneStartup));
        world.add_schedule(Schedule::new(ScenePostExtract));

        world.insert_resource(Monitor(monitor_id));
        world.init_resource::<DummyWorld>();

        let mut flags = SceneFlags::empty();

        if config.no_update {
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

#[derive(Component)]
pub struct TimeScale(pub f32);

#[derive(Resource)]
pub struct TestAssets {
    pub quad_mesh: AssetHandle<Mesh>,
    pub triangle_mesh: AssetHandle<Mesh>,
    pub image: AssetHandle<Image>,
    pub image_aspect_ratio: f32,
    pub image_material: AssetHandle<ImageMaterial>,
    pub video1: AssetHandle<Video>,
    pub video1_material: AssetHandle<VideoMaterial>,
    pub video2: AssetHandle<Video>,
    pub video2_material: AssetHandle<VideoMaterial>,
    pub video1_aspect_ratio: f32,
    pub video2_aspect_ratio: f32,
}

impl FromWorld for TestAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        let quad_mesh = meshes.add(Mesh::rect(Vec2::ONE));
        let triangle_mesh = meshes.add(Mesh {
            vertices: smallvec![
                Vertex(Vec3::new(-0.5, -0.5, 0.0)),
                Vertex(Vec3::new(0.5, -0.5, 0.0)),
                Vertex(Vec3::new(0.0, 0.5, 0.0)),
            ],
        });

        let mut videos = world.resource_mut::<Assets<Video>>();

        let video1 = Video::new(c"target/test-video.mp4").unwrap();
        let video1_aspect_ratio = video1.frame_aspect_ratio();
        let video1_handle = videos.add(video1);

        let video2 = Video::new(c"target/test-video2.mp4").unwrap();
        let video2_aspect_ratio = video2.frame_aspect_ratio();
        let video2_handle = videos.add(video2);

        let mut images = world.resource_mut::<Assets<Image>>();

        // FIXME(hack3rmann): use local image
        const PATH: &str = "target/test-image.png";
        let image = ::image::ImageReader::open(PATH)
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();

        let image_aspect_ratio = image.height() as f32 / image.width() as f32;
        let image = images.add(Image { image });

        let mut image_materials = world.resource_mut::<Assets<ImageMaterial>>();
        let image_material = image_materials.add(ImageMaterial { image });

        let mut video_materials = world.resource_mut::<Assets<VideoMaterial>>();
        let video1_material = video_materials.add(VideoMaterial {
            video: video1_handle,
        });
        let video2_material = video_materials.add(VideoMaterial {
            video: video2_handle,
        });

        Self {
            quad_mesh,
            triangle_mesh,
            image,
            image_aspect_ratio,
            image_material,
            video1: video1_handle,
            video1_aspect_ratio,
            video1_material,
            video2: video2_handle,
            video2_aspect_ratio,
            video2_material,
        }
    }
}

impl SceneTestWallpaper {
    pub fn new_test(monitor_id: MonitorId) -> Self {
        let mut scene = Scene::new(monitor_id);

        scene.add_systems(SceneUpdate, Self::rotate_meshes);
        scene.add_systems(SceneStartup, (Self::spawn_mesh, Self::spawn_videos));
        scene.world.init_resource::<TestAssets>();

        Self { scene }
    }

    pub fn spawn_videos(mut commands: Commands, assets: Res<TestAssets>) {
        const SCALE: f32 = 0.6;

        commands.spawn((
            Mesh3d(assets.quad_mesh),
            MeshMaterial(assets.video1_material),
            Transform::default().scaled_by(Vec3::new(
                SCALE,
                assets.video1_aspect_ratio * SCALE,
                1.0,
            )),
            TimeScale(0.5),
        ));

        commands.spawn((
            Mesh3d(assets.quad_mesh),
            MeshMaterial(assets.video2_material),
            Transform::default().scaled_by(Vec3::new(
                SCALE,
                assets.video2_aspect_ratio * SCALE,
                1.0,
            )),
            TimeScale(0.3),
        ));

        commands.spawn((
            Mesh3d(assets.triangle_mesh),
            MeshMaterial(assets.video2_material),
            Transform::default().scaled_by(Vec3::new(
                SCALE,
                assets.video2_aspect_ratio * SCALE,
                1.0,
            )),
            TimeScale(0.8),
        ));
    }

    pub fn spawn_mesh(mut commands: Commands, assets: Res<TestAssets>) {
        const SCALE: f32 = 0.6;
        let aspect_scale = Vec3::new(SCALE, SCALE * assets.image_aspect_ratio, 1.0);

        commands.spawn((
            Mesh3d(assets.quad_mesh),
            Transform::default().scaled_by(aspect_scale),
            MeshMaterial(assets.image_material),
            TimeScale(1.0),
        ));

        commands.spawn((
            Mesh3d(assets.quad_mesh),
            Transform::default().scaled_by(aspect_scale),
            MeshMaterial(assets.image_material),
            TimeScale(std::f32::consts::FRAC_PI_2),
        ));
    }

    pub fn rotate_meshes(
        mut transforms: Query<(&mut Transform, &TimeScale), With<Mesh3d>>,
        time: Res<Time>,
    ) {
        let time = time.elapsed.as_secs_f32();

        for (mut transform, &TimeScale(time_scale)) in &mut transforms {
            transform.translation.x = 0.5 * (time_scale * time).cos();
            transform.translation.y = 0.5 * (time_scale * time).sin();
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
