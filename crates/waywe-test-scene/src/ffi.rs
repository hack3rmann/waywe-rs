use crate::SceneTestWallpaper;
use std::{mem::MaybeUninit, panic};
use waywe_rendering_api::{
    api::{OpaqueRenderer, OpaqueRendererDesc},
    ffi::PanicPayload,
};
use waywe_scene::ffi::create_opaque_renderer;

#[unsafe(no_mangle)]
pub extern "C" fn waywe_ffi_create_opaque_renderer(
    desc: &OpaqueRendererDesc,
    out_renderer: &mut MaybeUninit<OpaqueRenderer>,
) -> PanicPayload {
    let result = panic::catch_unwind(move || create_opaque_renderer::<SceneTestWallpaper>(desc));

    PanicPayload::map(result, |renderer| {
        out_renderer.write(renderer);
    })
}
