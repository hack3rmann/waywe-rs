use crate::{
    Monitor,
    glam::UVec2,
    gpu::Gpu,
    prelude::{Wallpaper, WallpaperBuilder},
    wallpaper::PreparedWallpaper,
};
use std::{panic, sync::Arc};
use waywe_rendering_api::{
    api::{OpaqueRenderer, OpaqueRendererDesc, RenderSurfaceFd, Renderer},
    import_fd_as_texture,
};
use waywe_runtime::frame::FrameInfo;

pub fn create_opaque_renderer<W: WallpaperBuilder + Default>(
    desc: &OpaqueRendererDesc,
) -> OpaqueRenderer {
    let gpu = Arc::new(pollster::block_on(Gpu::new()));

    let mut wallpaper = Wallpaper::new(Arc::clone(&gpu), desc.config.into());
    W::default().build(&mut wallpaper);

    OpaqueRenderer::new(SceneRenderer {
        gpu,
        scene: PreparedWallpaper::prepare(wallpaper),
        surface: None,
    })
}

#[derive(Default, Debug)]
pub struct SceneRenderer {
    gpu: Arc<Gpu>,
    scene: PreparedWallpaper,
    surface: Option<wgpu::Texture>,
}

impl Renderer for SceneRenderer {
    fn render(&mut self) -> FrameInfo {
        let Some(surface) = self.surface.as_ref() else {
            panic!("surface is unset");
        };

        let surface_view = surface.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());

        let info = self.scene.frame(surface_view, &mut encoder);

        // TODO(hack3rmann): double buffer this stuff
        let index = self.gpu.queue.submit([encoder.finish()]);
        self.gpu
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: None,
            })
            .unwrap();

        info
    }

    fn set_surface(&mut self, surface: RenderSurfaceFd) {
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

        self.surface = Some(surface);
    }
}
