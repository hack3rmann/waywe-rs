use super::{
    ffi::{wl_display, wl_display_connect_to_fd, wl_display_disconnect},
    object::{WlObject, WlObjectHandle, registry::WlRegistry},
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
    any, fmt, mem::ManuallyDrop, os::fd::{FromRawFd, IntoRawFd, OwnedFd}, pin::Pin, ptr::NonNull
};
use thiserror::Error;

/// A handle to the libwayland backend
pub struct WlDisplay {
    proxy: ManuallyDrop<WlProxy>,
}

impl WlDisplay {
    pub fn connect_to_fd(wayland_file_desc: OwnedFd) -> Result<Self, WaylandConnectionError> {
        let raw_fd = wayland_file_desc.into_raw_fd();

        let display = NonNull::new(unsafe { wl_display_connect_to_fd(raw_fd) })
            .ok_or_else(|| WaylandConnectionError(unsafe { OwnedFd::from_raw_fd(raw_fd) }))?;

        // Safety: `*mut wl_display` is compatible with `*mut wl_proxy`
        let proxy = ManuallyDrop::new(unsafe { WlProxy::from_raw(display.cast()) });

        // Safety: `storage` is dropped before `wl_display` disconnects
        // see `Drop` impl for `WlDisplay`
        Ok(Self { proxy })
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
        storage: Pin<&mut WlObjectStorage<'_>>,
    ) -> WlObjectHandle<WlRegistry> {
        // Safety: parent interface matcher request's one
        let proxy = unsafe {
            WlDisplayGetRegistryRequest
                .send(buf, storage.as_ref().get_ref(), &self.proxy)
                .unwrap()
        };

        storage.insert(WlObject::new(proxy, WlRegistry::default()))
    }

    pub fn sync_all(&self, _storage: Pin<&mut WlObjectStorage>) {
        // Safety: `self.as_raw_display_ptr()` is a valid display object
        assert_ne!(
            -1,
            unsafe { wl_display_roundtrip(self.as_raw_display_ptr().as_ptr()) },
            "wl_display_roundtrip failed",
        );
    }
}

#[derive(Debug, Error)]
#[error("failed to connect to wayland's socket")]
pub struct WaylandConnectionError(pub OwnedFd);

impl Drop for WlDisplay {
    fn drop(&mut self) {
        // Safety: `self.as_raw_display_ptr()` is a valid display object
        unsafe { wl_display_disconnect(self.as_raw_display_ptr().as_ptr()) };
    }
}

impl fmt::Debug for WlDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("proxy", &*self.proxy)
            .finish()
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
