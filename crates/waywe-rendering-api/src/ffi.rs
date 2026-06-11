use crate::api::{FfiFrameInfo, RenderSurfaceFd, Renderer};
use abi_stable::std_types::{ROption, RString};
use std::{any::Any, ffi::c_void, mem::MaybeUninit, panic, ptr};

pub(crate) type RenderFn = unsafe extern "C" fn(
    renderer: *mut c_void,
    frame_info: &mut MaybeUninit<FfiFrameInfo>,
) -> PanicPayload;

pub(crate) type SetSurfaceFn = unsafe extern "C" fn(
    renderer: *mut c_void,
    surface: RenderSurfaceFd,
    index: u32,
) -> PanicPayload;

pub(crate) type CycleBuffersFn = unsafe extern "C" fn(renderer: *mut c_void) -> PanicPayload;

pub(crate) type DropFn = unsafe extern "C" fn(renderer: *mut c_void) -> PanicPayload;

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct PanicPayload(pub ROption<RString>);

impl PanicPayload {
    pub const EMPTY: Self = Self(ROption::RNone);

    pub fn propagate_if_any(self) {
        if let ROption::RSome(payload_string) = self.0 {
            let s = String::from(payload_string);
            panic::resume_unwind(Box::new(s))
        }
    }

    pub fn map<T>(payload: Result<T, Box<dyn Any + Send>>, f: impl FnOnce(T)) -> Self {
        match payload {
            Ok(value) => {
                f(value);
                Self::EMPTY
            }
            Err(payload) => payload.into(),
        }
    }
}

impl From<Box<dyn Any + Send>> for PanicPayload {
    fn from(payload: Box<dyn Any + Send>) -> Self {
        let payload_string = if let Some(&s) = payload.downcast_ref::<&str>() {
            RString::from(s)
        } else if let Some(s) = payload.downcast_ref::<String>() {
            RString::from(s.as_str())
        } else {
            RString::from("non-string panic")
        };

        PanicPayload(ROption::RSome(payload_string))
    }
}

impl From<Result<(), Box<dyn Any + Send>>> for PanicPayload {
    fn from(value: Result<(), Box<dyn Any + Send>>) -> Self {
        match value {
            Ok(()) => Self::EMPTY,
            Err(payload) => payload.into(),
        }
    }
}

impl From<Option<Box<dyn Any + Send>>> for PanicPayload {
    fn from(value: Option<Box<dyn Any + Send>>) -> Self {
        match value {
            None => Self::EMPTY,
            Some(payload) => payload.into(),
        }
    }
}

pub(crate) unsafe extern "C" fn render<T: Renderer>(
    renderer: *mut c_void,
    frame_info: &mut MaybeUninit<FfiFrameInfo>,
) -> PanicPayload {
    let result = panic::catch_unwind(move || {
        let this = unsafe { renderer.cast::<T>().as_mut().unwrap_unchecked() };
        this.render()
    });

    PanicPayload::map(result, |info| {
        frame_info.write(info.into());
    })
}

pub(crate) unsafe extern "C" fn set_surface<T: Renderer>(
    renderer: *mut c_void,
    surface: RenderSurfaceFd,
    index: u32,
) -> PanicPayload {
    panic::catch_unwind(move || {
        let this = unsafe { renderer.cast::<T>().as_mut().unwrap_unchecked() };
        this.set_surface(surface, index);
    })
    .into()
}

pub(crate) unsafe extern "C" fn cycle_buffers<T: Renderer>(renderer: *mut c_void) -> PanicPayload {
    panic::catch_unwind(move || {
        let this = unsafe { renderer.cast::<T>().as_mut().unwrap_unchecked() };
        this.cycle_buffers();
    })
    .into()
}

pub(crate) unsafe extern "C" fn drop<T>(renderer: *mut c_void) -> PanicPayload {
    panic::catch_unwind(move || {
        unsafe { ptr::drop_in_place(renderer.cast::<T>()) };
    })
    .into()
}
