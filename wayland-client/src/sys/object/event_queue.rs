use super::dispatch::{NoState, State};
use crate::{WlDisplay, WlProxy};
use std::{
    borrow::Cow,
    ffi::CStr,
    marker::PhantomData,
    ops::Deref,
    ptr::{self, NonNull},
};
use thiserror::Error;
use wayland_sys::{wl_display_create_queue_with_name, wl_event_queue};

pub struct WlEventQueue<'s, 'd, S: State> {
    raw: NonNull<wl_event_queue>,
    // HACK(hack3rmann): maybe name will be unused
    _name: Option<Cow<'s, CStr>>,
    _p: PhantomData<(*const S, &'d WlDisplay<S>)>,
}

impl<'s, S: State> WlEventQueue<'s, '_, S> {
    /// # Safety
    ///
    /// Expected to be created by the [`WlDisplay`]
    pub unsafe fn new(display: &WlDisplay<S>) -> Result<Self, EventQueueCreateError> {
        unsafe { Self::new_with_name(display, None::<&'static CStr>) }
    }

    pub fn as_raw(&self) -> NonNull<wl_event_queue> {
        self.raw
    }

    /// # Safety
    ///
    /// Expected to be created by the [`WlDisplay`]
    pub unsafe fn new_with_name(
        display: &WlDisplay<S>,
        name: Option<impl Into<Cow<'s, CStr>>>,
    ) -> Result<Self, EventQueueCreateError> {
        let name = name.map(Into::<Cow<CStr>>::into);
        let name_ptr = name.as_ref().map(|n| n.as_ptr()).unwrap_or(ptr::null());

        let raw = NonNull::new(unsafe {
            wl_display_create_queue_with_name(display.as_raw_display_ptr().as_ptr(), name_ptr)
        })
        .ok_or(EventQueueCreateError)?;

        Ok(Self {
            _name: name,
            raw,
            _p: PhantomData,
        })
    }

    // TODO(hack3rmann): determine if it should use `mut` or not
    pub fn add_proxy(&mut self, proxy: WlProxy) -> WlEqueuedProxy<'_> {
        WlEqueuedProxy {
            inner: proxy,
            _p: PhantomData,
        }
    }
}

#[derive(Debug, Error)]
#[error("failed to create event queue")]
pub struct EventQueueCreateError;

pub struct WlEqueuedProxy<'q> {
    inner: WlProxy,
    _p: PhantomData<&'q WlEventQueue<'static, 'static, NoState>>,
}

impl Deref for WlEqueuedProxy<'_> {
    type Target = WlProxy;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
