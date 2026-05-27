use crate::api::{FrameImage, Renderer};
use abi_stable::std_types::{ROption, RString};
use std::{any::Any, ffi::c_void, mem::MaybeUninit, panic, ptr};

pub(crate) type FrameFn = unsafe extern "C" fn(
    renderer: *mut c_void,
    image: &mut MaybeUninit<FrameImage>,
) -> PanicPayload;
pub(crate) type DropFn = unsafe extern "C" fn(renderer: *mut c_void) -> PanicPayload;

#[repr(transparent)]
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

pub(crate) unsafe extern "C" fn frame<T: Renderer>(
    renderer: *mut c_void,
    image: &mut MaybeUninit<FrameImage>,
) -> PanicPayload {
    let unwind = panic::catch_unwind(move || {
        let this = unsafe { renderer.cast::<T>().as_mut().unwrap_unchecked() };
        this.frame()
    });

    match unwind {
        Ok(frame) => {
            image.write(frame);
            PanicPayload::EMPTY
        }
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
