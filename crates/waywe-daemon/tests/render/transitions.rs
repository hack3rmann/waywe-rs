use crate::harness::{Rect, assert_avg_color, default_config, solid_rgba_image, test_gpu};
use std::sync::Arc;
use std::time::Duration;
use waywe_config::{Angle, AnimationConfig, BlurConfig, Config, Effects, Transition};
use waywe_daemon::wallpaper::{
    Wallpaper,
    optimized::{
        OptimizedWallpaper,
        image::{Color, ImageWallpaper},
    },
    transition::{RunningWallpapers, SubmissionIdGenerator},
};
use waywe_runtime::testing::{DEFAULT_FORMAT, DEFAULT_SIZE, OffscreenSurface, readback_rgba8};

fn config_with_style(style: Transition, duration_ms: u64) -> Arc<Config> {
    Arc::new(Config {
        animation: AnimationConfig {
            duration: duration_ms,
            style,
            ..Default::default()
        },
        ..Default::default()
    })
}

fn solid_wall(gpu: &waywe_runtime::gpu::Wgpu, rgba: [u8; 4]) -> OptimizedWallpaper {
    let image = solid_rgba_image(32, 32, rgba);
    OptimizedWallpaper::Image(ImageWallpaper::new(
        gpu,
        &image,
        Color::BLACK,
        default_config(),
    ))
}

fn transition_frame_at(style: Transition, fraction: f32, test_name: &str, min_r: u64, min_b: u64) {
    let gpu = test_gpu();
    let duration_ms = 1_000;
    let config = config_with_style(style, duration_ms);
    let ids = Arc::new(SubmissionIdGenerator::default());
    let mut running = RunningWallpapers::new(default_config(), config, ids);

    running.enqueue_wallpaper(&gpu, solid_wall(&gpu, [255, 0, 0, 255]));
    running.enqueue_wallpaper(&gpu, solid_wall(&gpu, [0, 0, 255, 255]));

    let advance_ms = (duration_ms as f32 * fraction) as u64;
    running.advance_time(Duration::from_millis(advance_ms));

    let surface = OffscreenSurface::new(&gpu.device, DEFAULT_SIZE, DEFAULT_FORMAT);
    let texture = surface.texture_clone();
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    running
        .render(&gpu, &texture, &mut encoder)
        .expect("transition frame");
    gpu.queue.submit([encoder.finish()]);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();

    let image = readback_rgba8(&gpu, &texture);
    let avg_r = image.pixels().map(|p| u64::from(p.0[0])).sum::<u64>() / image.len() as u64;
    let avg_b = image.pixels().map(|p| u64::from(p.0[2])).sum::<u64>() / image.len() as u64;
    assert!(
        avg_r > min_r && avg_b > min_b,
        "{test_name}: expected both colors in transition, avg_r={avg_r} avg_b={avg_b}"
    );
}

#[test]
fn circle_transition_midpoint_blends() {
    // Circle reveal grows from the center; sample late enough that blue is visible.
    transition_frame_at(Transition::default(), 0.85, "circle", 5, 5);
}

#[test]
fn slide_transition_midpoint_blends() {
    transition_frame_at(
        Transition::Slide {
            angle: Angle::Degrees(0.0),
        },
        0.5,
        "slide",
        10,
        10,
    );
}

#[test]
fn fadeout_transition_midpoint_blends() {
    transition_frame_at(Transition::Fadeout, 0.5, "fadeout", 10, 10);
}

#[test]
fn effects_on_running_wallpaper() {
    let gpu = test_gpu();
    let config = Arc::new(Config {
        effects: vec![Effects::Blur(BlurConfig {
            n_levels: 1,
            level_multiplier: 1,
        })],
        ..Default::default()
    });
    let ids = Arc::new(SubmissionIdGenerator::default());
    let mut running = RunningWallpapers::new(default_config(), config, ids);
    running.enqueue_wallpaper(&gpu, solid_wall(&gpu, [255, 0, 0, 255]));

    let surface = OffscreenSurface::new(&gpu.device, DEFAULT_SIZE, DEFAULT_FORMAT);
    let texture = surface.texture_clone();
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    running.render(&gpu, &texture, &mut encoder).unwrap();
    gpu.queue.submit([encoder.finish()]);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();

    let image = readback_rgba8(&gpu, &texture);
    assert_avg_color(
        &image,
        Rect::full(DEFAULT_SIZE),
        [255, 0, 0, 255],
        50,
        "effects_running",
    );
}
