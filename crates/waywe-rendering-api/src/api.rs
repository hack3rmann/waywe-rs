use crate::{
    FfiTextureDescriptor,
    ffi::{self, CycleBuffersFn, DropFn, PanicPayload, RenderFn, SetSurfaceFn},
};
use abi_stable::std_types::{RDuration, ROption, RString};
use std::{
    mem::MaybeUninit,
    os::{fd::OwnedFd, raw::c_void},
};
use waywe_runtime::{WallpaperConfig, frame::FrameInfo};

pub type CreateOpaqueRendererFn = extern "C" fn(
    desc: &OpaqueRendererDesc,
    out_renderer: &mut MaybeUninit<OpaqueRenderer>,
) -> PanicPayload;

pub const CREATE_OPAQUE_RENDERER_NAME: &str = "waywe_ffi_create_opaque_renderer";

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct FfiFrameInfo {
    pub target_frame_time: ROption<RDuration>,
}

impl From<FrameInfo> for FfiFrameInfo {
    fn from(value: FrameInfo) -> Self {
        let target_frame_time = match value.target_frame_time {
            Some(duration) => ROption::RSome(duration.into()),
            None => ROption::RNone,
        };

        Self { target_frame_time }
    }
}

impl From<FfiFrameInfo> for FrameInfo {
    fn from(value: FfiFrameInfo) -> Self {
        let target_frame_time = match value.target_frame_time {
            ROption::RSome(duration) => Some(duration.into()),
            ROption::RNone => None,
        };

        Self { target_frame_time }
    }
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct OpaqueRendererDesc {
    pub config: WallpaperConfig,
    pub working_directory: RString,
    pub surface_buffer_count: u32,
}

#[repr(C)]
pub struct RenderSurfaceFd {
    pub fd: OwnedFd,
    pub desc: FfiTextureDescriptor<'static>,
}

#[repr(C)]
pub struct OpaqueRenderer {
    render: RenderFn,
    set_surface: SetSurfaceFn,
    cycle_buffers: CycleBuffersFn,
    drop: DropFn,
    renderer: *mut c_void,
}

impl OpaqueRenderer {
    pub fn new<T: Renderer>(renderer: T) -> Self {
        Self {
            render: ffi::render::<T>,
            set_surface: ffi::set_surface::<T>,
            cycle_buffers: ffi::cycle_buffers::<T>,
            drop: ffi::drop::<T>,
            renderer: Box::into_raw(Box::new(renderer)).cast(),
        }
    }
}

impl Renderer for OpaqueRenderer {
    fn render(&mut self) -> FrameInfo {
        let mut frame_info = MaybeUninit::uninit();

        let panic = unsafe { (self.render)(self.renderer, &mut frame_info) };
        panic.propagate_if_any();

        unsafe { frame_info.assume_init() }.into()
    }

    fn set_surface(&mut self, surface: RenderSurfaceFd, index: u32) {
        let panic = unsafe { (self.set_surface)(self.renderer, surface, index) };
        panic.propagate_if_any();
    }

    fn cycle_buffers(&mut self) {
        let panic = unsafe { (self.cycle_buffers)(self.renderer) };
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
    fn render(&mut self) -> FrameInfo;
    fn set_surface(&mut self, surface: RenderSurfaceFd, surface_index: u32);
    fn cycle_buffers(&mut self);
}
