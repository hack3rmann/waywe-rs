use crate::harness::{
    Rect, assert_avg_color, assert_region_approx, default_config, render_wallpaper,
    solid_rgba_image, test_gpu,
};
use image::Rgba;
use waywe_daemon::wallpaper::optimized::image::{Color, ImageWallpaper};
use waywe_runtime::testing::DEFAULT_SIZE;

#[test]
fn solid_red_image_wallpaper() {
    let gpu = test_gpu();
    let image = solid_rgba_image(32, 32, [255, 0, 0, 255]);
    let mut wallpaper = ImageWallpaper::new(&gpu, &image, Color::BLACK, default_config());

    let frame = render_wallpaper(&gpu, &mut wallpaper, DEFAULT_SIZE);
    assert_avg_color(
        &frame,
        Rect::full(DEFAULT_SIZE),
        [255, 0, 0, 255],
        12,
        "solid_red_image",
    );
}

#[test]
fn transparency_color_shows_through_zero_alpha() {
    let gpu = test_gpu();
    // Fully transparent image → background transparency color (opaque green)
    let image = solid_rgba_image(32, 32, [0, 0, 0, 0]);
    let green = Color(0x00_FF_00_FF);
    let mut wallpaper = ImageWallpaper::new(&gpu, &image, green, default_config());

    let frame = render_wallpaper(&gpu, &mut wallpaper, DEFAULT_SIZE);
    // Center should be green after gamma encode of (0,1,0)
    assert_region_approx(
        &frame,
        Rect::new(
            DEFAULT_SIZE.x / 4,
            DEFAULT_SIZE.y / 4,
            DEFAULT_SIZE.x / 2,
            DEFAULT_SIZE.y / 2,
        ),
        [0, 255, 0, 255],
        20,
        "transparency_green",
    );
}

#[test]
fn checker_image_has_both_colors() {
    let gpu = test_gpu();
    let mut image = solid_rgba_image(32, 32, [255, 0, 0, 255]);
    for y in 0..16 {
        for x in 0..16 {
            image.put_pixel(x, y, Rgba([0, 0, 255, 255]));
        }
    }
    let mut wallpaper = ImageWallpaper::new(&gpu, &image, Color::BLACK, default_config());
    let frame = render_wallpaper(&gpu, &mut wallpaper, DEFAULT_SIZE);

    // Sample corners of the letterboxed content — just ensure not uniform black
    let avg_r =
        frame.pixels().map(|p| u64::from(p.0[0])).sum::<u64>() / frame.pixels().len() as u64;
    let avg_b =
        frame.pixels().map(|p| u64::from(p.0[2])).sum::<u64>() / frame.pixels().len() as u64;
    assert!(avg_r > 20, "expected some red, avg_r={avg_r}");
    assert!(avg_b > 20, "expected some blue, avg_b={avg_b}");
}
