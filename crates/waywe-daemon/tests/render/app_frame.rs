use crate::harness::{Rect, assert_avg_color, default_config, solid_rgba_image, test_gpu};
use calloop::channel::channel;
use glam::UVec2;
use std::sync::{Arc, Once};
use waywe_config::Config;
use waywe_daemon::{
    wallpaper::optimized::{
        OptimizedWallpaper,
        image::{Color, ImageWallpaper},
    },
    wallpaper_app::{WallpaperApp, WallpaperState},
};
use waywe_runtime::{
    Runtime,
    app::App,
    event::{EventReceiver, Handle},
    platform::{MonitorName, PlatformEvent, Scale, SurfaceInfo},
    tasks::Tasks,
    testing::{DEFAULT_SIZE, OffscreenPlatform, readback_rgba8},
    timer::Timer,
};

fn ensure_video_init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(video::init);
}

fn make_runtime(
    platform: Arc<dyn waywe_runtime::platform::WaywePlatform>,
    gpu: Arc<waywe_runtime::gpu::Wgpu>,
) -> (Runtime, EventReceiver) {
    ensure_video_init();
    let receiver = EventReceiver::new().expect("event receiver");
    let emitter = receiver.make_emitter().expect("emitter");
    let tasks = Tasks::new(emitter);
    let (ipc_tx, _ipc_rx) = channel();
    let runtime = Runtime {
        timer: Timer::default(),
        wgpu: gpu,
        platform,
        tasks,
        ipc: ipc_tx,
    };
    (runtime, receiver)
}

#[tokio::test]
async fn wallpaper_app_plug_frame_unplug() {
    let gpu = test_gpu();
    let platform = OffscreenPlatform::single_monitor(gpu.clone());
    let (mut runtime, _receiver) = make_runtime(
        platform.clone() as Arc<dyn waywe_runtime::platform::WaywePlatform>,
        gpu.clone(),
    );
    let mut app = WallpaperApp::from_config(Config::default());

    let info = SurfaceInfo {
        monitor_name: MonitorName::from("DP-1"),
        physical_size: DEFAULT_SIZE,
        scale: Scale::ONE,
    };

    let _ = Handle::<PlatformEvent>::handle(
        &mut app,
        &mut runtime,
        PlatformEvent::MonitorPlugged { info: info.clone() },
    )
    .await;

    assert!(app.monitors.contains_key("DP-1"));
    assert!(app.wallpapers.contains_key("DP-1"));
    assert!(runtime.wgpu.surfaces.read().unwrap().contains_key("DP-1"));

    // Enqueue a solid red image wallpaper
    let image = solid_rgba_image(32, 32, [255, 0, 0, 255]);
    let wall = OptimizedWallpaper::Image(ImageWallpaper::new(
        &runtime.wgpu,
        &image,
        Color::BLACK,
        default_config(),
    ));
    app.wallpapers
        .get_mut("DP-1")
        .unwrap()
        .enqueue_wallpaper(&runtime.wgpu, wall);
    app.wallpaper_states.insert(
        MonitorName::from("DP-1"),
        WallpaperState { needs_redraw: true },
    );

    let frame_info = App::frame(&mut app, &mut runtime).await.expect("frame");
    let _ = frame_info;

    let texture = platform.surface("DP-1").unwrap().texture_clone();
    let pixels = readback_rgba8(&runtime.wgpu, &texture);
    assert_avg_color(
        &pixels,
        Rect::full(DEFAULT_SIZE),
        [255, 0, 0, 255],
        20,
        "app_frame_red",
    );

    // Resize (updates the wgpu surface; monitor map size is left to the platform)
    let new_size = UVec2::new(128, 64);
    let _ = Handle::<PlatformEvent>::handle(
        &mut app,
        &mut runtime,
        PlatformEvent::ResizeRequested {
            monitor_name: MonitorName::from("DP-1"),
            phisical_size: new_size,
        },
    )
    .await;
    let surface_size = {
        let surfaces = runtime.wgpu.surfaces.read().unwrap();
        let cfg = &surfaces["DP-1"].config;
        UVec2::new(cfg.width, cfg.height)
    };
    assert_eq!(surface_size, new_size);

    // Unplug
    let _ = Handle::<PlatformEvent>::handle(
        &mut app,
        &mut runtime,
        PlatformEvent::MonitorUnplugged {
            monitor_name: MonitorName::from("DP-1"),
        },
    )
    .await;
    assert!(!app.monitors.contains_key("DP-1"));
    assert!(!app.wallpapers.contains_key("DP-1"));
    assert!(!runtime.wgpu.surfaces.read().unwrap().contains_key("DP-1"));
}
