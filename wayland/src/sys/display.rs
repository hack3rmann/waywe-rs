use super::{
    object::{WlObject, WlObjectHandle, registry::WlRegistry},
    object_storage::WlObjectStorage,
    proxy::WlProxy,
    wire::MessageBuffer,
};
use crate::{
    init::{GetSocketPathError, connect_wayland_socket},
    interface::{Request, WlDisplayGetRegistryRequest},
    object::{HasObjectType, WlObjectType},
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, RawDisplayHandle, WaylandDisplayHandle,
};
use std::{any, fmt, mem::ManuallyDrop, os::fd::IntoRawFd, pin::Pin, ptr::NonNull};
use thiserror::Error;
use wayland_sys::{
    wl_display, wl_display_connect_to_fd, wl_display_disconnect, wl_display_roundtrip,
};

/// A handle to the libwayland backend
pub struct WlDisplay {
    proxy: ManuallyDrop<WlProxy>,
}

impl WlDisplay {
    /// Connect to libwayland backend
    pub fn connect() -> Result<Self, DisplayConnectError> {
        let fd = unsafe { connect_wayland_socket()? };
        Ok(Self::connect_to_fd(fd)?)
    }

    /// Connect to Wayland display on an already open fd.
    /// The fd will be closed in case of failure.
    pub fn connect_to_fd(fd: impl IntoRawFd) -> Result<Self, DisplayConnectToFdError> {
        let raw_fd = fd.into_raw_fd();

        // Safety: calling this function on a valid file descriptor is ok
        let display = NonNull::new(unsafe { wl_display_connect_to_fd(raw_fd) })
            .ok_or(DisplayConnectToFdError)?;

        // Safety: `*mut wl_display` is compatible with `*mut wl_proxy`
        let proxy = ManuallyDrop::new(unsafe { WlProxy::from_raw(display.cast()) });

        Ok(Self { proxy })
    }

    /// Proxy corresponding to `wl_display` object
    pub fn proxy(&self) -> &WlProxy {
        &self.proxy
    }

    /// Raw display pointer
    pub fn as_raw_display_ptr(&self) -> NonNull<wl_display> {
        self.proxy.as_raw().cast()
    }

    /// Creates a [`WlObjectStorage`] borrowing display for the lifetime of the storage
    pub fn create_storage(&self) -> WlObjectStorage<'_> {
        // Safety: storage has captured the lifetime of `&self`
        // therefore it will be dropped before the `WlDisplay`
        unsafe { WlObjectStorage::new() }
    }

    /// Creates `wl_registry` object and stores it in the storage
    pub fn create_registry(
        &self,
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage<'_>>,
    ) -> WlObjectHandle<WlRegistry> {
        // Safety: parent interface matcher request's one
        let proxy = unsafe {
            WlDisplayGetRegistryRequest
                .send(buf, storage.as_ref().get_ref(), self.proxy())
                .unwrap()
        };

        storage.insert(WlObject::new(proxy, WlRegistry::default()))
    }

    /// Block until all pending requests are processed by the server.
    ///
    /// This function blocks until the server has processed all currently
    /// issued requests by sending a request to the display server
    /// and waiting for a reply before returning.
    pub fn dispatch_all_pending(&self, _: Pin<&mut WlObjectStorage>) {
        // Safety: `self.as_raw_display_ptr()` is a valid display object
        assert_ne!(
            -1,
            unsafe { wl_display_roundtrip(self.as_raw_display_ptr().as_ptr()) },
            "wl_display_roundtrip failed",
        );
    }
}

#[derive(Debug, Error)]
pub enum DisplayConnectError {
    #[error(transparent)]
    ConnectWaylandSocket(#[from] GetSocketPathError),
    #[error(transparent)]
    ConnectToFd(#[from] DisplayConnectToFdError),
}

#[derive(Debug, Error)]
#[error("failed to connect to wayland's socket")]
pub struct DisplayConnectToFdError;

impl HasObjectType for WlDisplay {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Display;
}

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
