use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures, wayland::MonitorId},
    wallpaper::{
        OldWallpaper as OldWallpaper,
        scene::{
            FrameRateSetting, WallpaperConfig, WallpaperFlags, Startup, Update, Time, Wallpaper,
            assets::{AssetHandle, Assets},
            image::{Image, ImageMaterial},
            mesh::{Mesh, Mesh3d, MeshMaterial, Vertex},
            render::Render,
            transform::Transform,
            video::{Video, VideoMaterial},
        },
    },
};
use bevy_ecs::prelude::*;
use for_sure::Almost;
use glam::{Quat, Vec2, Vec3};
use smallvec::smallvec;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

pub struct SceneTestWallpaper {
    pub scene: Wallpaper,
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
    pub video1_aspect_ratio: f32,
    pub video2: AssetHandle<Video>,
    pub video2_material: AssetHandle<VideoMaterial>,
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
        let mut scene = Wallpaper::new_with_config(
            monitor_id,
            WallpaperConfig {
                framerate: FrameRateSetting::GuessFromScene,
            },
        );

        scene.add_systems(Update, Self::rotate_meshes);
        scene.add_systems(Startup, (Self::spawn_mesh, Self::spawn_videos));
        scene.world.init_resource::<TestAssets>();

        scene.world.run_schedule(Startup);
        scene.flags |= WallpaperFlags::STARTUP_DONE;

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

            transform.rotation = Quat::from_axis_angle(Vec3::X + time_scale * Vec3::Y, time);
        }
    }
}

impl OldWallpaper for SceneTestWallpaper {
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
                if NOT_FIRST_TIME.fetch_or(true, Ordering::Relaxed) {
                    let mut renderer = runtime.scene_renderer.write().unwrap();
                    renderer.world.run_schedule(Render);
                }
            });

            self.scene.update();

            handle.join().unwrap();
        });

        let mut renderer = runtime.scene_renderer.write().unwrap();
        self.scene.extract(&mut renderer.world);

        let frame_info = match self.scene.world.resource::<FrameRateSetting>() {
            FrameRateSetting::TargetFrameDuration(duration) => FrameInfo {
                target_frame_time: Some(*duration),
            },
            FrameRateSetting::NoUpdate => FrameInfo {
                target_frame_time: None,
            },
            FrameRateSetting::GuessFromScene => FrameInfo::new_60_fps(),
        };

        Ok(frame_info)
    }

    fn required_features() -> RuntimeFeatures
    where
        Self: Sized,
    {
        RuntimeFeatures::SCENE_RENDERER
    }
}
