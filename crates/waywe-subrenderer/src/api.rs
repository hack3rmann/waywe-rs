use crate::{
    FfiTextureDescriptor,
    ffi::{self, DropFn, RenderFn, SetSurfaceFn},
};
use std::os::{fd::OwnedFd, raw::c_void};

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct OpaqueRendererDesc {
    pub surface_desc: FfiTextureDescriptor<'static>,
}

pub type CreateOpaqueRendererFn = extern "C" fn(desc: &OpaqueRendererDesc) -> OpaqueRenderer;

pub const CREATE_OPAQUE_RENDERER_NAME: &str = "waywe_ffi_create_opaque_renderer";

#[repr(C)]
pub struct RenderSurfaceFd {
    fd: OwnedFd,
    desc: FfiTextureDescriptor<'static>,
}

#[repr(C)]
pub struct OpaqueRenderer {
    render: RenderFn,
    set_surface: SetSurfaceFn,
    drop: DropFn,
    renderer: *mut c_void,
}

impl OpaqueRenderer {
    pub fn new<T: Renderer>(renderer: T) -> Self {
        Self {
            render: ffi::render::<T>,
            set_surface: ffi::set_surface::<T>,
            drop: ffi::drop::<T>,
            renderer: Box::into_raw(Box::new(renderer)).cast(),
        }
    }
}

impl Renderer for OpaqueRenderer {
    fn render(&mut self) {
        let panic = unsafe { (self.render)(self.renderer) };
        panic.propagate_if_any();
    }

    fn set_surface(&mut self, surface: RenderSurfaceFd) {
        let panic = unsafe { (self.set_surface)(self.renderer, surface) };
        panic.propagate_if_any();
    }
}

impl Drop for OpaqueRenderer {
    fn drop(&mut self) {
        let panic = unsafe { (self.drop)(self.renderer) };
        panic.propagate_if_any();
    }
}

pub trait Renderer {
    fn render(&mut self);
    fn set_surface(&mut self, surface: RenderSurfaceFd);
}
