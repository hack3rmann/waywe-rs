use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{wayland::MonitorId, Runtime, RuntimeFeatures},
    wallpaper::{
        scene::{
            assets::Assets, image::{Image, ImageMaterial}, mesh::{Mesh, Mesh3d, MeshMaterial}, render::{Renderer, Render}, transform::Transform, FrameRateSetting, Startup, Wallpaper, WallpaperConfig
        }, OldWallpaper
    },
};
use bevy_ecs::prelude::*;
use derive_more::Deref;
use for_sure::Almost;
use glam::{Vec2, Vec3};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

pub struct WallpaperBuildConfig {
    pub monitor_id: MonitorId,
}

pub trait WallpaperBuilder {
    fn build(self, config: WallpaperBuildConfig) -> Wallpaper;

    #[expect(unused_variables)]
    fn initialize_renderer(&mut self, renderer: &mut Renderer) {}
}

pub struct ImageWallpaper {
    pub path: PathBuf,
}

#[derive(Resource, Deref)]
pub struct ImagePath(pub PathBuf);

impl WallpaperBuilder for ImageWallpaper {
    fn build(self, config: WallpaperBuildConfig) -> Wallpaper {
        let mut scene = Wallpaper::new_with_config(
            config.monitor_id,
            WallpaperConfig {
                framerate: FrameRateSetting::NoUpdate,
            },
        );

        scene.world.insert_resource(ImagePath(self.path));
        scene.add_systems(Startup, setup);

        scene
    }
}

pub fn setup(
    mut commands: Commands,
    path: Res<ImagePath>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ImageMaterial>>,
) {
    let image_data = ::image::ImageReader::open(&**path)
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();

    let aspect_ratio = image_data.height() as f32 / image_data.width() as f32;

    let mesh = meshes.add(Mesh::rect(Vec2::ONE));
    let image = images.add(Image { image: image_data });
    let material = materials.add(ImageMaterial { image });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial(material),
        Transform::default().scaled_by(Vec3::new(1.0, aspect_ratio, 1.0)),
    ));
}

pub struct WallpaperWrapper(pub Wallpaper);

impl OldWallpaper for WallpaperWrapper {
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

        if let FrameRateSetting::NoUpdate = self.0.world.resource::<FrameRateSetting>() {
            let mut renderer = runtime.scene_renderer.write().unwrap();
            self.0.startup();
            self.0.extract(&mut renderer.world);
            renderer.world.run_schedule(Render);
        } else {
            thread::scope(|s| {
                let handle = s.spawn(|| {
                    if NOT_FIRST_TIME.fetch_or(true, Ordering::Relaxed) {
                        let mut renderer = runtime.scene_renderer.write().unwrap();
                        renderer.world.run_schedule(Render);
                    }
                });

                self.0.update();

                handle.join().unwrap();
            });
        }

        let mut renderer = runtime.scene_renderer.write().unwrap();
        self.0.extract(&mut renderer.world);

        let frame_info = match self.0.world.resource::<FrameRateSetting>() {
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
