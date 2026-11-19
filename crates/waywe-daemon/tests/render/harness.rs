use glam::UVec2;
use std::sync::Arc;
use waywe_daemon::wallpaper::Wallpaper;
use waywe_runtime::{
    WallpaperConfig,
    gpu::Wgpu,
    testing::{DEFAULT_FORMAT, DEFAULT_SIZE, OffscreenSurface, clear_texture, readback_rgba8},
};

pub use waywe_runtime::testing::{Rect, assert_avg_color, assert_region_approx, solid_rgba_image};

pub fn test_gpu() -> Arc<Wgpu> {
    Arc::new(Wgpu::new())
}

pub fn wallpaper_config(size: UVec2) -> WallpaperConfig {
    WallpaperConfig {
        surface_size: size,
        surface_format: DEFAULT_FORMAT,
    }
}

pub fn default_config() -> WallpaperConfig {
    wallpaper_config(DEFAULT_SIZE)
}

/// Render one wallpaper frame to an offscreen texture and read back RGBA8.
pub fn render_wallpaper(
    gpu: &Wgpu,
    wallpaper: &mut impl Wallpaper,
    size: UVec2,
) -> image::RgbaImage {
    let surface = OffscreenSurface::new(&gpu.device, size, DEFAULT_FORMAT);
    let texture = surface.texture_clone();
    clear_texture(gpu, &texture, wgpu::Color::BLACK);

    let view = texture.create_view(&Default::default());
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    wallpaper.frame(gpu, &view, &mut encoder);
    gpu.queue.submit([encoder.finish()]);
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();

    readback_rgba8(gpu, &texture)
}
