use crate::api::{RenderSurfaceFd, Renderer};
use abi_stable::std_types::{ROption, RString};
use std::{any::Any, ffi::c_void, panic, ptr};

pub(crate) type RenderFn = unsafe extern "C" fn(renderer: *mut c_void) -> PanicPayload;

pub(crate) type SetSurfaceFn =
    unsafe extern "C" fn(renderer: *mut c_void, surface: RenderSurfaceFd) -> PanicPayload;

pub(crate) type DropFn = unsafe extern "C" fn(renderer: *mut c_void) -> PanicPayload;

#[repr(transparent)]
#[derive(Clone, Debug)]
pub(crate) struct PanicPayload(pub ROption<RString>);

impl PanicPayload {
    pub const EMPTY: Self = Self(ROption::RNone);

    pub fn propagate_if_any(self) {
        if let ROption::RSome(payload_string) = self.0 {
            let s = String::from(payload_string);
            panic::resume_unwind(Box::new(s))
        }
    }
}

fn map_panic_payload(payload: Box<dyn Any + Send>) -> PanicPayload {
    let payload_string = if let Some(&s) = payload.downcast_ref::<&str>() {
        RString::from(s)
    } else if let Some(s) = payload.downcast_ref::<String>() {
        RString::from(s.as_str())
    } else {
        RString::from("non-string panic")
    };

    PanicPayload(ROption::RSome(payload_string))
}

pub(crate) unsafe extern "C" fn render<T: Renderer>(renderer: *mut c_void) -> PanicPayload {
    let unwind = panic::catch_unwind(move || {
        let this = unsafe { renderer.cast::<T>().as_mut().unwrap_unchecked() };
        this.render()
    });

    match unwind {
        Ok(()) => PanicPayload::EMPTY,
        Err(payload) => map_panic_payload(payload),
    }
}

pub(crate) unsafe extern "C" fn set_surface<T: Renderer>(
    renderer: *mut c_void,
    surface: RenderSurfaceFd,
) -> PanicPayload {
    let unwind = panic::catch_unwind(move || {
        let this = unsafe { renderer.cast::<T>().as_mut().unwrap_unchecked() };
        this.set_surface(surface);
    });

    match unwind {
        Ok(()) => PanicPayload::EMPTY,
        Err(payload) => map_panic_payload(payload),
    }
}

pub(crate) unsafe extern "C" fn drop<T>(renderer: *mut c_void) -> PanicPayload {
    let unwind = panic::catch_unwind(move || {
        unsafe { ptr::drop_in_place(renderer.cast::<T>()) };
    });

    match unwind {
        Ok(()) => PanicPayload::EMPTY,
        Err(payload) => map_panic_payload(payload),
    }
}
