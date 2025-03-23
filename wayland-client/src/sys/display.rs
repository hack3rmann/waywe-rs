use super::{
    object::{WlObject, WlObjectHandle, dispatch::State, registry::WlRegistry},
    object_storage::WlObjectStorage,
    proxy::WlProxy,
    wire::MessageBuffer,
};
use crate::{
    init::{GetSocketPathError, connect_wayland_socket},
    interface::{Request, WlDisplayGetRegistryRequest, WlObjectType},
    object::HasObjectType,
    sys::object::dispatch::handle_dispatch_raw_panic,
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, RawDisplayHandle, WaylandDisplayHandle,
};
use std::{
    any, fmt,
    mem::ManuallyDrop,
    os::fd::{BorrowedFd, IntoRawFd, RawFd},
    pin::Pin,
    ptr::{self, NonNull},
    sync::atomic::{
        AtomicPtr,
        Ordering::{Acquire, Relaxed, Release},
    },
};
use thiserror::Error;
use wayland_sys::{
    wl_display, wl_display_connect_to_fd, wl_display_disconnect, wl_display_roundtrip,
};

/// A handle to the libwayland backend
pub struct WlDisplay<S: State> {
    proxy: ManuallyDrop<WlProxy>,
    state: NonNull<S>,
    storage: AtomicPtr<()>,
    raw_fd: RawFd,
}

impl<S: State> WlDisplay<S> {
    /// Connect to libwayland backend
    pub fn connect(state: Pin<&mut S>) -> Result<Self, DisplayConnectError> {
        let fd = unsafe { connect_wayland_socket()? };
        Ok(Self::connect_to_fd(state, fd)?)
    }

    /// Connect to Wayland display on an already open fd.
    /// The fd will be closed in case of failure.
    pub fn connect_to_fd(
        state: Pin<&mut S>,
        fd: impl IntoRawFd,
    ) -> Result<Self, DisplayConnectToFdError> {
        let raw_fd = fd.into_raw_fd();

        // Safety: calling this function on a valid file descriptor is ok
        let display = NonNull::new(unsafe { wl_display_connect_to_fd(raw_fd) })
            .ok_or(DisplayConnectToFdError)?;

        // Safety: `*mut wl_display` is compatible with `*mut wl_proxy`
        let proxy = ManuallyDrop::new(unsafe { WlProxy::from_raw(display.cast()) });

        Ok(Self {
            proxy,
            raw_fd,
            // Safety: constructing NonNull from pinned pointer is safe
            state: NonNull::from(unsafe { state.get_unchecked_mut() }),
            storage: AtomicPtr::new(ptr::null_mut()),
        })
    }

    /// File descriptor associated with the display
    pub fn fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.raw_fd) }
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
    pub fn create_storage(&self) -> WlObjectStorage<'_, S> {
        // Safety: storage has captured the lifetime of `&self`
        // therefore it will be dropped before the `WlDisplay`
        unsafe { WlObjectStorage::new(self.state) }
    }

    /// Creates `wl_registry` object and stores it in the storage
    pub fn create_registry(
        &self,
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage<'_, S>>,
    ) -> WlObjectHandle<WlRegistry<S>> {
        if !self.storage.load(Acquire).is_null() {
            panic!("error creating registry twice");
        }

        self.storage
            .store((&raw const *storage).cast_mut().cast(), Release);

        // Safety: parent interface matcher request's one
        let proxy = unsafe {
            WlDisplayGetRegistryRequest
                .send(buf, storage.as_ref().get_ref(), self.proxy())
                .unwrap()
        };

        storage.insert(WlObject::new(proxy, WlRegistry::default()))
    }

    /// # Safety
    ///
    /// - no one should access the object storage during this call
    /// - no one should access the state during this call
    pub unsafe fn roundtrip_unchecked(&self) -> i32 {
        unsafe { wl_display_roundtrip(self.as_raw_display_ptr().as_ptr()) }
    }

    /// Block until all pending requests are processed by the server.
    ///
    /// This function blocks until the server has processed all currently
    /// issued requests by sending a request to the display server
    /// and waiting for a reply before returning.
    pub fn roundtrip(&self, storage: Pin<&mut WlObjectStorage<'_, S>>, state: Pin<&mut S>) {
        assert_eq!(&raw const *state, self.state.as_ptr().cast_const());
        assert_eq!(
            &raw const *storage,
            self.storage.load(Relaxed).cast_const().cast()
        );

        let n_events_dispatched = unsafe { self.roundtrip_unchecked() };

        handle_dispatch_raw_panic();

        // Safety: `self.as_raw_display_ptr()` is a valid display object
        assert_ne!(-1, n_events_dispatched, "wl_display_roundtrip failed",);

        tracing::info!("WlDisplay::roundtrip has dispatched {n_events_dispatched} events");
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

impl<S: State> HasObjectType for WlDisplay<S> {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Display;
}

impl<S: State> Drop for WlDisplay<S> {
    fn drop(&mut self) {
        // Safety: `self.as_raw_display_ptr()` is a valid display object
        unsafe { wl_display_disconnect(self.as_raw_display_ptr().as_ptr()) };
    }
}

impl<S: State> fmt::Debug for WlDisplay<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("proxy", &*self.proxy)
            .finish()
    }
}

impl<S: State> HasDisplayHandle for WlDisplay<S> {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                self.as_raw_display_ptr(),
            )))
        })
    }
}
