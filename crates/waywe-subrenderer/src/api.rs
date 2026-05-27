use crate::{
    FfiTextureDescriptor,
    ffi::{self, DropFn, FrameFn, PanicPayload},
};
use std::{
    mem::MaybeUninit,
    os::{fd::OwnedFd, raw::c_void},
};

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct OpaqueRendererDesc {
    pub surface_desc: FfiTextureDescriptor<'static>,
}

pub type CreateOpaqueRendererFn = extern "C" fn(desc: &OpaqueRendererDesc) -> OpaqueRenderer;

pub const CREATE_OPAQUE_RENDERER_NAME: &str = "waywe_ffi_create_opaque_renderer";

#[repr(C)]
pub struct FrameImage {
    fd: OwnedFd,
    desc: FfiTextureDescriptor<'static>,
}

#[repr(C)]
pub struct OpaqueRenderer {
    frame: FrameFn,
    drop: DropFn,
    renderer: *mut c_void,
    panic_payload: PanicPayload,
}

impl OpaqueRenderer {
    pub fn frame(&mut self) -> FrameImage {
        let mut image = MaybeUninit::<FrameImage>::uninit();

        let panic = unsafe { (self.frame)(self.renderer, &mut image) };
        panic.propagate_if_any();

        unsafe { image.assume_init() }
    }
}

impl<T: Renderer> From<T> for OpaqueRenderer {
    fn from(value: T) -> Self {
        Self {
            frame: ffi::frame::<T>,
            drop: ffi::drop::<T>,
            renderer: Box::into_raw(Box::new(value)).cast(),
            panic_payload: PanicPayload::EMPTY,
        }
    }
}

impl Drop for OpaqueRenderer {
    fn drop(&mut self) {
        let panic = unsafe { (self.drop)(self.renderer) };
        panic.propagate_if_any();
    }
}

pub trait Renderer {
    fn frame(&mut self) -> FrameImage;
}
