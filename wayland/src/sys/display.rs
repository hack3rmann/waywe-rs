use super::{
    ffi::{wl_display, wl_display_connect_to_fd, wl_display_disconnect},
    object::{registry::WlRegistry, WlObject, WlObjectHandle},
    object_storage::WlObjectStorage,
    proxy::WlProxy,
    wire::MessageBuffer,
};
use crate::{
    interface::{Request, WlDisplayGetRegistryRequest},
    sys::ffi::wl_display_roundtrip,
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, RawDisplayHandle, WaylandDisplayHandle,
};
use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    os::fd::{IntoRawFd, OwnedFd},
    ptr::NonNull,
};

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlDisplayBound<'d>(pub PhantomData<&'d WlDisplay>);

impl WlDisplayBound<'_> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// A handle to libwayland backend
pub struct WlDisplay {
    proxy: ManuallyDrop<WlProxy>,
}

impl WlDisplay {
    pub fn connect_to_fd(wayland_file_desc: OwnedFd) -> Self {
        // FIXME(hack3rmann): deal with errors
        let display =
            NonNull::new(unsafe { wl_display_connect_to_fd(wayland_file_desc.into_raw_fd()) })
                .expect("failed to connect wl_display");

        // Safety: `*mut wl_display` is compatible with `*mut wl_proxy`
        let proxy = ManuallyDrop::new(unsafe { WlProxy::from_raw(display.cast()) });

        // Safety: `storage` is dropped before `wl_display` disconnects
        // see `Drop` impl for `WlDisplay`
        Self { proxy }
    }

    pub fn as_raw_display_ptr(&self) -> NonNull<wl_display> {
        self.proxy.as_raw().cast()
    }

    pub fn create_storage(&self) -> WlObjectStorage<'_> {
        unsafe { WlObjectStorage::new() }
    }

    pub fn create_registry(
        &self,
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage<'_>,
    ) -> WlObjectHandle<WlRegistry> {
        // Safety: parent interface matcher request's one
        let raw_proxy = unsafe { WlDisplayGetRegistryRequest.send_raw(&self.proxy, buf) };

        // Safety: resulting proxy is a valid object provided by libwayland
        let proxy = unsafe { WlProxy::from_raw(NonNull::new(raw_proxy).unwrap()) };

        storage.insert(WlObject::new(proxy, WlRegistry::default()))
    }

    pub fn sync_all(&self) {
        // Safety: `self.as_raw_display_ptr()` is a valid display object
        assert_ne!(-1, unsafe {
            wl_display_roundtrip(self.as_raw_display_ptr().as_ptr())
        });
    }
}

impl Drop for WlDisplay {
    fn drop(&mut self) {
        // Safety: `self.as_raw_display_ptr()` is a valid display object
        unsafe { wl_display_disconnect(self.as_raw_display_ptr().as_ptr()) };
    }
}

impl HasDisplayHandle for WlDisplay {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                self.as_raw_display_ptr(),
            )))
        })
    }
}
