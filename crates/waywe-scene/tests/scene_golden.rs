mod common;

use common::{assert_full_avg, render_scene};
use glam::{Vec2, Vec3};
use std::path::PathBuf;
use waywe_scene::{
    assets::Assets,
    clear_screen::ClearColor,
    image::{Image, ImageMaterial},
    mesh::{Mesh, Mesh3d, MeshMaterial},
    prelude::*,
    sprite::Sprite,
    transform::Transform,
    video::Video,
};

struct ClearRed;

impl WallpaperBuilder for ClearRed {
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);
        wallpaper
            .render
            .insert_resource(ClearColor(Vec3::new(1.0, 0.0, 0.0)));
    }
}

#[test]
fn clear_color_red() {
    let image = render_scene(ClearRed);
    assert_full_avg(&image, [255, 0, 0, 255], 15, "scene_clear_red");
}

struct TexturedMesh;

impl WallpaperBuilder for TexturedMesh {
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);
        wallpaper.render.insert_resource(ClearColor(Vec3::ZERO));
        wallpaper.main.add_systems(Startup, spawn_red_quad);
    }
}

fn spawn_red_quad(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ImageMaterial>>,
) {
    let mut rgba = image::RgbaImage::new(8, 8);
    for p in rgba.pixels_mut() {
        *p = image::Rgba([255, 0, 0, 255]);
    }
    let image = images.add(Image { image: rgba });
    let material = materials.add(ImageMaterial { image });
    let mesh = meshes.add(Mesh::rect(Vec2::ONE));
    commands.spawn((Mesh3d(mesh), MeshMaterial(material), Transform::default()));
}

#[test]
fn textured_mesh_renders() {
    let image = render_scene(TexturedMesh);
    // Center should be dominated by the red quad over black clear
    let w = image.width();
    let h = image.height();
    let cx = w / 2;
    let cy = h / 2;
    let pixel = image.get_pixel(cx, cy).0;
    assert!(
        pixel[0] > 100,
        "expected red-ish center pixel, got {pixel:?}"
    );
}

struct SpriteScene;

impl WallpaperBuilder for SpriteScene {
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);
        wallpaper.render.insert_resource(ClearColor(Vec3::ZERO));
        wallpaper.main.add_systems(Startup, spawn_sprite);
    }
}

fn spawn_sprite(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ImageMaterial>>,
) {
    let mut rgba = image::RgbaImage::new(8, 8);
    for p in rgba.pixels_mut() {
        *p = image::Rgba([0, 255, 0, 255]);
    }
    let image = images.add(Image { image: rgba });
    let material = materials.add(ImageMaterial { image });
    commands.spawn((
        Sprite {
            texture: material.into(),
        },
        Transform::default(),
    ));
}

#[test]
fn sprite_renders() {
    let image = render_scene(SpriteScene);
    let pixel = image.get_pixel(image.width() / 2, image.height() / 2).0;
    assert!(
        pixel[1] > 100,
        "expected green-ish sprite center, got {pixel:?}"
    );
}

#[test]
fn video_export_uploads_planes() {
    use std::sync::Arc;
    use waywe_scene::{gpu::Gpu, video::RenderVideo};

    let gpu = Arc::new(pollster::block_on(Gpu::new()));
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../waywe-daemon/tests/fixtures/solid-blue.mp4");
    let video = Video::new(path).expect("open fixture video");
    let rendered = RenderVideo::export_from(&video, &gpu.adapter, &gpu.device);
    assert!(rendered.texture_y_plane.texture().width() >= 16);
    assert!(rendered.texture_uv_plane.texture().width() >= 8);
}
