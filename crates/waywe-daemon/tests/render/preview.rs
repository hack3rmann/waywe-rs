use crate::harness::{Rect, assert_avg_color, default_config, solid_rgba_image, test_gpu};
use std::sync::mpsc;
use waywe_daemon::wallpaper::{
    optimized::{
        OptimizedWallpaper,
        image::{Color, ImageWallpaper},
    },
    preview::PreviewPipeline,
};
use waywe_runtime::testing::DEFAULT_SIZE;

#[test]
fn preview_pipeline_reads_solid_image() {
    let gpu = test_gpu();
    let image = solid_rgba_image(32, 32, [0, 255, 0, 255]);
    let mut wallpaper = OptimizedWallpaper::Image(ImageWallpaper::new(
        &gpu,
        &image,
        Color::BLACK,
        default_config(),
    ));

    let preview = PreviewPipeline::new(&gpu, default_config());
    let (tx, rx) = mpsc::channel();

    preview.render_async(&gpu, &mut wallpaper, move |result| {
        tx.send(result).unwrap();
    });

    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();

    let buffer = rx.recv().unwrap().expect("preview map");
    let view = buffer.get_mapped_range(..).expect("mapped preview");
    let packed: &[u32] = bytemuck::cast_slice(&view);
    let width = DEFAULT_SIZE.x as usize;
    let height = DEFAULT_SIZE.y as usize;
    assert_eq!(packed.len(), width * height);

    let mut image_buf = image::RgbaImage::new(DEFAULT_SIZE.x, DEFAULT_SIZE.y);
    for (i, pixel) in image_buf.pixels_mut().enumerate() {
        let p = packed[i];
        *pixel = image::Rgba([
            (p & 0xFF) as u8,
            ((p >> 8) & 0xFF) as u8,
            ((p >> 16) & 0xFF) as u8,
            ((p >> 24) & 0xFF) as u8,
        ]);
    }
    drop(view);
    buffer.unmap();

    assert_avg_color(
        &image_buf,
        Rect::full(DEFAULT_SIZE),
        [0, 255, 0, 255],
        30,
        "preview_green",
    );
}
