use super::{
    object::{
        WlObject, WlObjectHandle,
        dispatch::State,
        event_queue::{CreateQueueError, WlEventQueue},
        registry::WlRegistry,
    },
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
    sync::atomic::{AtomicBool, AtomicPtr, Ordering::*},
};
use thiserror::Error;
use wayland_sys::{
    DisplayErrorCode, DisplayErrorCodeFromI32Error, wl_display, wl_display_connect_to_fd,
    wl_display_create_queue, wl_display_disconnect, wl_display_get_error, wl_display_roundtrip,
    wl_display_roundtrip_queue, wl_event_queue,
};

/// A handle to the libwayland backend
pub struct WlDisplay<S: State> {
    proxy: ManuallyDrop<WlProxy>,
    state: NonNull<S>,
    main_storage: AtomicPtr<()>,
    main_queue_taken: AtomicBool,
    raw_fd: RawFd,
}

unsafe impl<S: State> Send for WlDisplay<S> {}
unsafe impl<S: State> Sync for WlDisplay<S> {}

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
            main_storage: AtomicPtr::new(ptr::null_mut()),
            main_queue_taken: AtomicBool::new(false),
        })
    }

    /// File descriptor associated with the display
    pub fn fd(&self) -> BorrowedFd<'_> {
        // display's file descriptor is valid
        unsafe { BorrowedFd::borrow_raw(self.raw_fd) }
    }

    /// Proxy corresponding to `wl_display` object
    pub fn proxy(&self) -> &WlProxy {
        &self.proxy
    }

    /// Raw display pointer
    pub fn as_raw(&self) -> NonNull<wl_display> {
        self.proxy.as_raw().cast()
    }

    /// Creates a [`WlObjectStorage`] borrowing display for the lifetime of the storage
    pub fn create_storage(&self) -> WlObjectStorage<'_, S> {
        // Safety: storage has captured the lifetime of `&self`
        // therefore it will be dropped before the `WlDisplay`
        unsafe { WlObjectStorage::new(self.state) }
    }

    /// Takes the main event queue from this display
    ///
    /// # Error
    ///
    /// Returns [`Err`] with [`CreateQueueError::MainTakenTwice`] if called twice
    pub fn take_main_queue(&self) -> Result<WlEventQueue<'_, S>, CreateQueueError> {
        if !self.main_queue_taken.fetch_or(true, Relaxed) {
            // Safety: we can ensure main queue is created once using `main_queue_taken` flag
            Ok(unsafe { WlEventQueue::main_from_display(self) })
        } else {
            Err(CreateQueueError::MainTakenTwice)
        }
    }

    /// Creates event queue
    pub fn create_queue(&self) -> Result<WlEventQueue<'_, S>, CreateQueueError> {
        WlEventQueue::side_from_display(self)
    }

    /// Creates `wl_registry` object and stores it in the storage
    pub fn create_registry<'d>(
        &'d self,
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage<'d, S>>,
    ) -> WlObjectHandle<WlRegistry<S>> {
        // FIXME(hack3rmann): it should not assume that `storage` belongs to the main queue
        assert!(
            self.main_storage
                .swap((&raw const *storage).cast_mut().cast(), AcqRel)
                .is_null(),
            "error creating registry twice",
        );

        // Safety: parent interface matcher request's one
        let proxy = unsafe {
            WlDisplayGetRegistryRequest
                .send(buf, storage.as_ref().get_ref(), self.proxy())
                .unwrap()
        };

        storage.insert(WlObject::new(proxy, WlRegistry::default()))
    }

    /// Creates raw event queue
    pub(crate) fn create_event_queue_raw(&self) -> *mut wl_event_queue {
        // Safety: display is valid
        unsafe { wl_display_create_queue(self.as_raw().as_ptr()) }
    }

    /// # Safety
    ///
    /// - no one should access the object storage during this call
    /// - no one should access the state during this call
    pub(crate) unsafe fn roundtrip_unchecked(&self) -> i32 {
        unsafe { wl_display_roundtrip(self.as_raw().as_ptr()) }
    }

    /// # Safety
    ///
    /// - no one should access the object storage during this call
    /// - no one should access the state during this call
    pub(crate) unsafe fn roundtrip_queue_unchecked<'d>(
        &'d self,
        queue: &WlEventQueue<'d, S>,
    ) -> i32 {
        if let Some(queue_ptr) = queue.as_raw() {
            unsafe { wl_display_roundtrip_queue(self.as_raw().as_ptr(), queue_ptr.as_ptr()) }
        } else {
            unsafe { self.roundtrip_unchecked() }
        }
    }

    pub(crate) fn get_error_code(&self) -> Result<DisplayErrorCode, DisplayErrorCodeFromI32Error> {
        let raw_code = unsafe { wl_display_get_error(self.as_raw().as_ptr()) };
        DisplayErrorCode::try_from(raw_code)
    }

    /// Block until all pending requests are processed by the server.
    ///
    /// This function blocks until the server has processed all currently
    /// issued requests by sending a request to the display server
    /// and waiting for a reply before returning.
    pub fn roundtrip<'d>(&'d self, queue: Pin<&mut WlEventQueue<'d, S>>, state: Pin<&S>) {
        assert_eq!(&raw const *state, self.state.as_ptr().cast_const());

        if queue.is_main() {
            assert_eq!(
                &raw const *queue.as_ref().storage(),
                self.main_storage.load(Relaxed).cast_const().cast()
            );
        }

        let n_events_dispatched = unsafe { self.roundtrip_queue_unchecked(&queue) };

        handle_dispatch_raw_panic();

        if n_events_dispatched == -1 {
            let error_code = self.get_error_code().unwrap();
            panic!("WlDisplay::roundtrip_queue failed: {error_code:?}");
        }

        tracing::debug!("WlDisplay::roundtrip dispatched {n_events_dispatched} events");
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
        unsafe { wl_display_disconnect(self.as_raw().as_ptr()) };
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
                self.as_raw(),
            )))
        })
    }
}
