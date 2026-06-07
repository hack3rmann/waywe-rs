#![allow(unused)]

use crate::SceneTestWallpaper;
use std::{mem::MaybeUninit, panic, sync::Arc};
use waywe_runtime::frame::FrameInfo;
use waywe_scene::{
    Monitor,
    glam::UVec2,
    gpu::Gpu,
    prelude::{Wallpaper, WallpaperBuilder},
    wallpaper::PreparedWallpaper,
};
use waywe_subrenderer::{
    api::{OpaqueRenderer, OpaqueRendererDesc, RenderSurfaceFd, Renderer},
    ffi::PanicPayload,
    import_fd_as_texture,
};

fn create_opaque_renderer(desc: &OpaqueRendererDesc) -> OpaqueRenderer {
    let gpu = Arc::new(pollster::block_on(Gpu::new()));

    let mut wallpaper = Wallpaper::new(Arc::clone(&gpu), desc.config.into());
    SceneTestWallpaper.build(&mut wallpaper);

    OpaqueRenderer::new(SceneRenderer {
        gpu,
        scene: PreparedWallpaper::prepare(wallpaper),
        surface: None,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn waywe_ffi_create_opaque_renderer(
    desc: &OpaqueRendererDesc,
    out_renderer: &mut MaybeUninit<OpaqueRenderer>,
) -> PanicPayload {
    let result = panic::catch_unwind(move || create_opaque_renderer(desc));

    PanicPayload::map(result, |renderer| {
        out_renderer.write(renderer);
    })
}

struct SceneRenderer {
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
        self.gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(index),
            timeout: None,
        });

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

        self.surface = Some(surface);
    }
}
