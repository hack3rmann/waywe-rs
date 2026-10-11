use crate::harness::{Rect, assert_avg_color, default_config, render_wallpaper, test_gpu};
use std::path::PathBuf;
use std::time::Duration;
use waywe_daemon::wallpaper::{Wallpaper, optimized::video::VideoWallpaper};
use waywe_runtime::testing::DEFAULT_SIZE;

fn fixture_mp4() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/solid-blue.mp4")
}

#[test]
fn video_wallpaper_renders_blue_frame() {
    let gpu = test_gpu();
    let mut wallpaper =
        VideoWallpaper::new(fixture_mp4(), &gpu, default_config()).expect("open video");

    // Decode at least one frame
    wallpaper.advance_time(Duration::from_millis(0));
    let frame = render_wallpaper(&gpu, &mut wallpaper, DEFAULT_SIZE);

    // Soft-decoded solid blue; allow generous tolerance for YUV roundtrip
    assert_avg_color(
        &frame,
        Rect::full(DEFAULT_SIZE),
        [0, 0, 255, 255],
        80,
        "video_solid_blue",
    );
}
