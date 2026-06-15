use crate::{
    Monitor,
    glam::UVec2,
    gpu::Gpu,
    prelude::{Wallpaper, WallpaperBuilder},
    wallpaper::PreparedWallpaper,
};
use std::{panic, sync::Arc, time::Duration};
use waywe_rendering_api::{
    VecExt,
    api::{OpaqueRenderer, OpaqueRendererDesc, RenderSurfaceFd, Renderer},
    import_fd_as_texture,
};
use waywe_runtime::frame::FrameInfo;

pub fn create_opaque_renderer<W: WallpaperBuilder + Default>(
    desc: &OpaqueRendererDesc,
) -> OpaqueRenderer {
    let gpu = Arc::new(pollster::block_on(Gpu::new()));

    let mut wallpaper = Wallpaper::new(
        Arc::clone(&gpu),
        desc.config.into(),
        desc.working_directory.as_str().into(),
    );
    W::default().build(&mut wallpaper);

    OpaqueRenderer::new(SceneRenderer {
        gpu,
        scene: PreparedWallpaper::prepare(wallpaper),
        surfaces: vec![],
        submissions: vec![],
    })
}

#[derive(Default, Debug)]
pub struct SceneRenderer {
    gpu: Arc<Gpu>,
    scene: PreparedWallpaper,
    surfaces: Vec<wgpu::Texture>,
    submissions: Vec<Option<wgpu::SubmissionIndex>>,
}

impl Renderer for SceneRenderer {
    fn render(&mut self) -> FrameInfo {
        let Some(surface) = self.surfaces.first() else {
            panic!("surface is unset");
        };

        if let Some(index) = self.submissions.first_mut().and_then(Option::take) {
            self.gpu
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(index),
                    timeout: None,
                })
                .unwrap();
        }

        let surface_view = surface.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());

        let info = self.scene.frame(surface_view, &mut encoder);

        let index = self.gpu.queue.submit([encoder.finish()]);
        self.submissions[0] = Some(index);

        info
    }

    fn set_surface(&mut self, surface: RenderSurfaceFd, index: u32) {
        let surface = unsafe {
            import_fd_as_texture(
                &self.gpu.device,
                &self.gpu.adapter,
                surface.fd,
                surface.desc,
            )
        };

        let monitor = Monitor {
            size: UVec2::new(surface.size().width, surface.size().height),
            surface_format: surface.format(),
        };
        *self.scene.wallpaper.main.resource_mut::<Monitor>() = monitor;
        *self.scene.wallpaper.render.resource_mut::<Monitor>() = monitor;

        self.surfaces.set_or_push(index as usize, surface);
        self.submissions.set_or_push(index as usize, None);
    }

    fn cycle_buffers(&mut self) {
        self.surfaces.rotate_left(1);
        self.submissions.rotate_left(1);
    }

    fn advance_time(&mut self, delta: Duration) {
        self.scene.wallpaper.advance_time(delta);
    }
}
