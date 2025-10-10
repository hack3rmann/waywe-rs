//! Safe wrapper around libwayland `wl_display` implementation

use super::{
    log,
    object::{
        WlObject, WlObjectHandle,
        dispatch::State,
        event_queue::{CreateQueueError, WlEventQueue},
        registry::WlRegistry,
    },
    object_storage::WlObjectStorage,
    proxy::WlProxy,
    wire::WlMessageBuffer,
};
use crate::{
    ffi,
    init::{ConnectWaylandSocketError, connect_wayland_socket},
    interface::{WlDisplayGetRegistryRequest, WlObjectType, send_request_raw},
    object::HasObjectType,
    sys::object::dispatch,
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, RawDisplayHandle, WaylandDisplayHandle,
};
use std::{
    fmt,
    mem::ManuallyDrop,
    os::fd::{AsFd, AsRawFd, BorrowedFd, IntoRawFd, RawFd},
    pin::Pin,
    ptr::NonNull,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::*},
    },
};
use thiserror::Error;
use wayland_sys::{DisplayErrorCode, wl_display, wl_event_queue};

pub(crate) struct WlDisplayInternal<S> {
    pub(crate) proxy: ManuallyDrop<WlProxy>,
    pub(crate) state: NonNull<S>,
    pub(crate) main_queue_taken: AtomicBool,
    pub(crate) raw_fd: RawFd,
}

impl<S> WlDisplayInternal<S> {
    /// Raw display pointer
    pub fn as_raw(&self) -> NonNull<wl_display> {
        self.proxy.as_raw().cast()
    }
}

impl<S> Drop for WlDisplayInternal<S> {
    fn drop(&mut self) {
        // Safety: `self.as_raw_display_ptr()` is a valid display object
        unsafe { ffi::wl_display_disconnect(self.as_raw().as_ptr()) };
    }
}

/// A handle to the libwayland backend
pub struct WlDisplay<S> {
    /// Raw display pointer. Used to reduce indirection.
    pub(crate) raw_display: NonNull<wl_display>,
    pub(crate) shared: Arc<WlDisplayInternal<S>>,
}

unsafe impl<S> Send for WlDisplay<S> {}
unsafe impl<S: Sync> Sync for WlDisplay<S> {}

impl<S> WlDisplay<S> {
    /// Connect to libwayland backend
    pub fn connect(state: Pin<&S>) -> Result<Self, DisplayConnectError>
    where
        S: State,
    {
        let fd = unsafe { connect_wayland_socket()? };
        Ok(Self::connect_to_fd(state, fd)?)
    }

    /// Connect to Wayland display on an already open fd.
    /// The fd will be closed in case of failure.
    pub fn connect_to_fd(
        state: Pin<&S>,
        fd: impl IntoRawFd,
    ) -> Result<Self, DisplayConnectToFdError>
    where
        S: State,
    {
        let raw_fd = fd.into_raw_fd();

        // Safety: calling this function on a valid file descriptor is ok
        let display = NonNull::new(unsafe { ffi::wl_display_connect_to_fd(raw_fd) })
            .ok_or(DisplayConnectToFdError)?;

        // Safety: `*mut wl_display` is compatible with `*mut wl_proxy`
        let proxy = ManuallyDrop::new(unsafe { WlProxy::from_raw(display.cast()) });

        log::setup();

        let internal = WlDisplayInternal {
            proxy,
            raw_fd,
            // Safety: constructing NonNull from pinned pointer is safe
            state: NonNull::from(state.get_ref()),
            main_queue_taken: AtomicBool::new(false),
        };

        Ok(Self {
            raw_display: internal.as_raw(),
            shared: Arc::new(internal),
        })
    }

    /// Proxy corresponding to `wl_display` object
    pub fn proxy(&self) -> &WlProxy {
        &self.shared.proxy
    }

    /// Raw display pointer
    pub fn as_raw(&self) -> NonNull<wl_display> {
        self.raw_display
    }

    /// Creates a [`WlObjectStorage`] borrowing display for the lifetime of the storage
    pub fn create_storage(&self) -> WlObjectStorage<S>
    where
        S: State,
    {
        // Safety: storage has captured the lifetime of `&self`
        // therefore it will be dropped before the `WlDisplay`
        WlObjectStorage::new(self.clone())
    }

    /// Takes the main event queue from this display
    ///
    /// # Error
    ///
    /// Returns [`Err`] with [`CreateQueueError::MainTakenTwice`] if called twice
    pub fn take_main_queue(&self) -> Result<WlEventQueue<S>, CreateQueueError>
    where
        S: State,
    {
        if !self.shared.main_queue_taken.fetch_or(true, Relaxed) {
            // Safety: we can ensure main queue is created once using `main_queue_taken` flag
            Ok(unsafe { WlEventQueue::main_from_display(self) })
        } else {
            Err(CreateQueueError::MainTakenTwice)
        }
    }

    /// Creates event queue
    pub fn create_queue(&self) -> Result<WlEventQueue<S>, CreateQueueError>
    where
        S: State,
    {
        WlEventQueue::side_from_display(self)
    }

    /// Creates `wl_registry` object and stores it in the storage
    pub fn create_registry<'s>(
        &self,
        buf: &mut impl WlMessageBuffer,
        mut storage: Pin<&'s mut WlObjectStorage<S>>,
    ) -> &'s mut WlRegistry<S>
    where
        S: State,
    {
        // Safety: parent interface matcher request's one
        let proxy = unsafe {
            send_request_raw(
                WlDisplayGetRegistryRequest,
                buf,
                storage.as_ref().get_ref(),
                self.proxy(),
            )
            .unwrap()
        };

        let handle = WlObjectHandle::new(proxy.id());

        _ = storage
            .as_mut()
            .insert(WlObject::new(proxy, WlRegistry::new(handle)));
        storage.get_mut().object_data_mut(handle)
    }

    /// Creates raw event queue
    pub(crate) fn create_event_queue_raw(&self) -> *mut wl_event_queue {
        // Safety: display is valid
        unsafe { ffi::wl_display_create_queue(self.as_raw().as_ptr()) }
    }

    /// # Safety
    ///
    /// - anyone mustn't access the object storage during this call
    /// - anyone mustn't access the state during this call
    pub(crate) unsafe fn roundtrip_unchecked(&self) -> i32 {
        unsafe { ffi::wl_display_roundtrip(self.as_raw().as_ptr()) }
    }

    /// # Safety
    ///
    /// - anyone mustn't access the object storage during this call
    /// - anyone mustn't access the state during this call
    pub(crate) unsafe fn roundtrip_queue_unchecked(&self, queue: &WlEventQueue<S>) -> i32
    where
        S: State,
    {
        if let Some(queue_ptr) = queue.as_raw() {
            unsafe { ffi::wl_display_roundtrip_queue(self.as_raw().as_ptr(), queue_ptr.as_ptr()) }
        } else {
            unsafe { self.roundtrip_unchecked() }
        }
    }

    /// Retrieve the last error that occurred on a display.
    ///
    /// # Error
    ///
    /// Returns [`None`] if no error occurred
    pub(crate) fn get_error_code(&self) -> Option<DisplayErrorCode> {
        // Safety: calling this function on a valid display is safe
        let raw_code = unsafe { ffi::wl_display_get_error(self.as_raw().as_ptr()) };

        (raw_code != 0).then_some({
            // Safety: non-zero code means it is a valid display error code
            unsafe { DisplayErrorCode::from_i32_unchecked(raw_code) }
        })
    }

    /// Block until all pending requests are processed by the server.
    ///
    /// This function blocks until the server has processed all currently
    /// issued requests by sending a request to the display server
    /// and waiting for a reply before returning.
    pub fn roundtrip(&self, queue: Pin<&mut WlEventQueue<S>>, state: Pin<&S>)
    where
        S: State,
    {
        assert_eq!(&raw const *state, self.shared.state.as_ptr().cast_const());

        let n_events_dispatched = unsafe { self.roundtrip_queue_unchecked(&queue) };

        dispatch::handle_panic();

        if n_events_dispatched == -1 {
            let error_code = self.get_error_code().unwrap();
            panic!("WlDisplay::roundtrip_queue failed: {error_code:?}");
        }
    }
}

impl<S> AsFd for WlDisplay<S> {
    /// File descriptor associated with the display
    fn as_fd(&self) -> BorrowedFd<'_> {
        // display's file descriptor is valid
        unsafe { BorrowedFd::borrow_raw(self.shared.raw_fd) }
    }
}

impl<S> AsRawFd for WlDisplay<S> {
    fn as_raw_fd(&self) -> RawFd {
        self.as_fd().as_raw_fd()
    }
}

impl<S> Clone for WlDisplay<S> {
    fn clone(&self) -> Self {
        Self {
            raw_display: self.raw_display,
            shared: Arc::clone(&self.shared),
        }
    }
}

/// Error connecting display
#[derive(Debug, Error)]
pub enum DisplayConnectError {
    /// Failed to connect to the wayland socket
    #[error(transparent)]
    ConnectWaylandSocket(#[from] ConnectWaylandSocketError),
    /// Failed to connect display to wayland fd
    #[error(transparent)]
    ConnectToFd(#[from] DisplayConnectToFdError),
}

/// Failed to connect display to wayland fd
#[derive(Debug, Error)]
#[error("failed to connect to wayland's socket")]
pub struct DisplayConnectToFdError;

impl<S> HasObjectType for WlDisplay<S> {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Display;
}

impl<S> fmt::Debug for WlDisplay<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WlDisplay")
            .field("proxy", &*self.shared.proxy)
            .finish_non_exhaustive()
    }
}

impl<S> HasDisplayHandle for WlDisplay<S> {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                self.as_raw(),
            )))
        })
    }
}
