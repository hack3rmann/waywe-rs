use glam::UVec2;
use std::sync::Arc;
use waywe_config::{BlurConfig, ConvolveConfig};
use waywe_runtime::{
    WallpaperConfig,
    effects::{Effects, config::EffectsBuilder},
    gpu::Wgpu,
    testing::{
        DEFAULT_FORMAT, DEFAULT_SIZE, OffscreenSurface, Rect, assert_avg_color, clear_texture,
        readback_rgba8,
    },
};

fn wallpaper_config() -> WallpaperConfig {
    WallpaperConfig {
        surface_size: DEFAULT_SIZE,
        surface_format: DEFAULT_FORMAT,
    }
}

fn render_effects(gpu: &Wgpu, mut effects: Effects) -> image::RgbaImage {
    let surface = OffscreenSurface::new(&gpu.device, DEFAULT_SIZE, DEFAULT_FORMAT);
    let texture = surface.texture_clone();
    clear_texture(
        gpu,
        &texture,
        wgpu::Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    );

    let view = texture.create_view(&Default::default());
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    effects.render(gpu, &view, &mut encoder);
    gpu.queue.submit([encoder.finish()]);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();

    readback_rgba8(gpu, &texture)
}

#[test]
fn blur_preserves_average_color_on_solid_input() {
    let gpu = Arc::new(Wgpu::new());
    let effects = EffectsBuilder::new()
        .add(BlurConfig {
            n_levels: 2,
            level_multiplier: 1,
        })
        .build(&gpu, wallpaper_config());

    let image = render_effects(&gpu, effects);
    assert_avg_color(
        &image,
        Rect::full(DEFAULT_SIZE),
        [255, 0, 0, 255],
        40,
        "blur_solid",
    );
}

#[test]
fn identity_convolve_preserves_solid_color() {
    let gpu = Arc::new(Wgpu::new());
    // 3x3 identity-ish kernel that keeps energy concentrated
    let kernel = [
        0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0,
    ];
    let effects = EffectsBuilder::new()
        .add(ConvolveConfig {
            kernel: kernel.to_vec(),
        })
        .build(&gpu, wallpaper_config());

    let image = render_effects(&gpu, effects);
    assert_avg_color(
        &image,
        Rect::full(UVec2::new(DEFAULT_SIZE.x, DEFAULT_SIZE.y)),
        [255, 0, 0, 255],
        8,
        "identity_convolve",
    );
}
