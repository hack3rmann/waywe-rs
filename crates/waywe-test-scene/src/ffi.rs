#![allow(unused)]

use crate::SceneTestWallpaper;
use std::{mem::MaybeUninit, panic, sync::Arc};
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
};

fn create_opaque_renderer(desc: &OpaqueRendererDesc) -> OpaqueRenderer {
    let gpu = Arc::new(pollster::block_on(Gpu::new()));
    let monitor = Monitor {
        size: UVec2::new(
            desc.surface_desc.extent.width,
            desc.surface_desc.extent.height,
        ),
        surface_format: desc.surface_desc.wgpu_format,
    };

    let mut wallpaper = Wallpaper::new(Arc::clone(&gpu), monitor);
    SceneTestWallpaper.build(&mut wallpaper);

    OpaqueRenderer::new(SceneRenderer {
        gpu,
        scene: PreparedWallpaper::prepare(wallpaper),
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
}

impl Renderer for SceneRenderer {
    fn render(&mut self) {
        todo!()
    }

    fn set_surface(&mut self, surface: RenderSurfaceFd) {
        todo!()
    }
}
