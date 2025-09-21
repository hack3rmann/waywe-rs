use crate::wallpaper::scene::{
    Startup, Update,
    assets::{AssetHandle, Assets},
    image::{Image, ImageMaterial},
    mesh::{Mesh, Mesh3d, MeshMaterial, Vertex},
    plugin::DefaultPlugins,
    time::Time,
    transform::Transform,
    video::{Video, VideoMaterial},
    wallpaper::{Wallpaper, WallpaperBuilder},
};
use bevy_ecs::prelude::*;
use glam::{Quat, Vec2, Vec3};
use smallvec::smallvec;

pub struct SceneTestWallpaper;

impl WallpaperBuilder for SceneTestWallpaper {
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);

        wallpaper
            .main
            .add_systems(Update, rotate_meshes)
            .add_systems(Startup, (spawn_mesh, spawn_videos))
            .init_resource::<TestAssets>();
    }
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

        let video1 = Video::new("target/test-video.mp4").unwrap();
        let video1_aspect_ratio = video1.frame_aspect_ratio();
        let video1_handle = videos.add(video1);

        let video2 = Video::new("target/test-video2.mp4").unwrap();
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

pub fn spawn_videos(mut commands: Commands, assets: Res<TestAssets>) {
    const SCALE: f32 = 0.6;

    commands.spawn((
        Mesh3d(assets.quad_mesh),
        MeshMaterial(assets.video1_material),
        Transform::default().scaled_by(Vec3::new(SCALE, assets.video1_aspect_ratio * SCALE, 1.0)),
        TimeScale(0.5),
    ));

    commands.spawn((
        Mesh3d(assets.quad_mesh),
        MeshMaterial(assets.video2_material),
        Transform::default().scaled_by(Vec3::new(SCALE, assets.video2_aspect_ratio * SCALE, 1.0)),
        TimeScale(0.3),
    ));

    commands.spawn((
        Mesh3d(assets.triangle_mesh),
        MeshMaterial(assets.video2_material),
        Transform::default().scaled_by(Vec3::new(SCALE, assets.video2_aspect_ratio * SCALE, 1.0)),
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
