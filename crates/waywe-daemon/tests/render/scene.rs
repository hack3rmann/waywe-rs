use crate::harness::{Rect, assert_avg_color, default_config, render_wallpaper, test_gpu};
use flate2::{Compression, write::GzEncoder};
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};
use tar::Builder;
use waywe_daemon::wallpaper::{
    optimized::render::RenderWallpaper, package_registry::PackageRegistry,
};
use waywe_runtime::testing::DEFAULT_SIZE;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn build_clear_scene_dylib() -> PathBuf {
    let root = workspace_root();
    let status = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["build", "-p", "waywe-clear-scene", "--quiet"])
        .status()
        .expect("spawn cargo build for waywe-clear-scene");
    assert!(status.success(), "waywe-clear-scene build failed");

    let so = root.join("target/debug/libwaywe_clear_scene.so");
    assert!(so.is_file(), "missing dylib at {}", so.display());
    so
}

fn package_ww(dylib: &Path, out: &Path) {
    let file = File::create(out).expect("create ww");
    let encoder = GzEncoder::new(file, Compression::default());
    let mut archive = Builder::new(encoder);
    archive.sparse(false);
    archive
        .append_path_with_name(dylib, "waywe-clear-scene/wallpaper.so")
        .expect("append wallpaper.so");
    let encoder = archive.into_inner().expect("finish tar");
    encoder.finish().expect("finish gzip");
}

#[test]
fn clear_scene_package_renders_red() {
    let dylib = build_clear_scene_dylib();
    let ww_path = std::env::temp_dir().join("waywe-clear-scene-test.ww");
    package_ww(&dylib, &ww_path);

    let gpu = test_gpu();
    let packages = PackageRegistry::default();
    let mut wallpaper =
        RenderWallpaper::load(&ww_path, &gpu, default_config(), packages).expect("load scene");

    let frame = render_wallpaper(&gpu, &mut wallpaper, DEFAULT_SIZE);
    assert_avg_color(
        &frame,
        Rect::full(DEFAULT_SIZE),
        [255, 0, 0, 255],
        20,
        "clear_scene_red",
    );

    let _ = std::fs::remove_file(ww_path);
}
