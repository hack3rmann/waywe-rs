use glam::UVec2;
use std::sync::Arc;
use waywe_runtime::{
    gpu::{SurfaceResult, Wgpu},
    platform::{MonitorName, Scale, SurfaceInfo},
    testing::{
        DEFAULT_SIZE, InjectedAcquire, OffscreenPlatform, Rect, assert_region_approx,
        clear_texture, readback_rgba8,
    },
};

fn monitor(name: &str, size: UVec2) -> SurfaceInfo {
    SurfaceInfo {
        monitor_name: MonitorName::from(name),
        physical_size: size,
        scale: Scale::ONE,
    }
}

#[test]
fn register_clear_and_readback() {
    let gpu = Arc::new(Wgpu::new());
    let platform = OffscreenPlatform::single_monitor(gpu.clone());
    let info = monitor("DP-1", DEFAULT_SIZE);

    gpu.register_surface(platform.as_ref(), &info);

    let surface = match gpu.get_current_surface(platform.as_ref(), &info) {
        SurfaceResult::Ok(surface) | SurfaceResult::Reconfigure(surface) => surface,
        _ => panic!("unexpected surface result"),
    };

    clear_texture(&gpu, surface.texture(), wgpu::Color::RED);
    surface.present(&gpu.queue);

    let image = readback_rgba8(&gpu, &platform.surface("DP-1").unwrap().texture_clone());
    assert_region_approx(
        &image,
        Rect::full(DEFAULT_SIZE),
        [255, 0, 0, 255],
        2,
        "register_clear_and_readback",
    );
}

#[test]
fn resize_surface_updates_texture_size() {
    let gpu = Arc::new(Wgpu::new());
    let platform = OffscreenPlatform::single_monitor(gpu.clone());
    let info = monitor("DP-1", DEFAULT_SIZE);
    gpu.register_surface(platform.as_ref(), &info);

    let new_size = UVec2::new(128, 72);
    gpu.resize_surface("DP-1", new_size);

    let surface = platform.surface("DP-1").unwrap();
    assert_eq!(surface.size(), new_size);

    let tex = surface.texture_clone();
    assert_eq!(tex.width(), new_size.x);
    assert_eq!(tex.height(), new_size.y);
}

#[test]
fn timeout_and_occluded_skip() {
    let gpu = Arc::new(Wgpu::new());
    let platform = OffscreenPlatform::single_monitor(gpu.clone());
    let info = monitor("DP-1", DEFAULT_SIZE);
    gpu.register_surface(platform.as_ref(), &info);

    platform.inject("DP-1", InjectedAcquire::Timeout);
    assert!(matches!(
        gpu.get_current_surface(platform.as_ref(), &info),
        SurfaceResult::Skip
    ));

    platform.inject("DP-1", InjectedAcquire::Occluded);
    assert!(matches!(
        gpu.get_current_surface(platform.as_ref(), &info),
        SurfaceResult::Skip
    ));
}

#[test]
fn outdated_retries_then_success() {
    let gpu = Arc::new(Wgpu::new());
    let platform = OffscreenPlatform::single_monitor(gpu.clone());
    let info = monitor("DP-1", DEFAULT_SIZE);
    gpu.register_surface(platform.as_ref(), &info);

    platform.inject("DP-1", InjectedAcquire::Outdated);
    match gpu.get_current_surface(platform.as_ref(), &info) {
        SurfaceResult::Ok(_) | SurfaceResult::Reconfigure(_) => {}
        _ => panic!("expected success after outdated, got non-ok"),
    }
}

#[test]
fn lost_recreates_surface() {
    let gpu = Arc::new(Wgpu::new());
    let platform = OffscreenPlatform::single_monitor(gpu.clone());
    let info = monitor("DP-1", DEFAULT_SIZE);
    gpu.register_surface(platform.as_ref(), &info);

    let before = Arc::as_ptr(&platform.surface("DP-1").unwrap());
    platform.inject("DP-1", InjectedAcquire::Lost);

    match gpu.get_current_surface(platform.as_ref(), &info) {
        SurfaceResult::Ok(_) | SurfaceResult::Reconfigure(_) => {}
        _ => panic!("expected success after lost"),
    }

    let after = Arc::as_ptr(&platform.surface("DP-1").unwrap());
    assert_ne!(before, after, "Lost should recreate the offscreen surface");
}

#[test]
fn validation_returns_err() {
    let gpu = Arc::new(Wgpu::new());
    let platform = OffscreenPlatform::single_monitor(gpu.clone());
    let info = monitor("DP-1", DEFAULT_SIZE);
    gpu.register_surface(platform.as_ref(), &info);

    platform.inject("DP-1", InjectedAcquire::Validation);
    assert!(matches!(
        gpu.get_current_surface(platform.as_ref(), &info),
        SurfaceResult::Err
    ));
}

#[test]
fn suboptimal_returns_reconfigure() {
    let gpu = Arc::new(Wgpu::new());
    let platform = OffscreenPlatform::single_monitor(gpu.clone());
    let info = monitor("DP-1", DEFAULT_SIZE);
    gpu.register_surface(platform.as_ref(), &info);

    platform.inject("DP-1", InjectedAcquire::Suboptimal);
    assert!(matches!(
        gpu.get_current_surface(platform.as_ref(), &info),
        SurfaceResult::Reconfigure(_)
    ));
}
