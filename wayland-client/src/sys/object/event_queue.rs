//! Wrapper around libwayland `wl_event_queue`

use super::dispatch::State;
use crate::{WlDisplay, WlObjectStorage};
use std::{fmt, mem::ManuallyDrop, pin::Pin, ptr::NonNull};
use thiserror::Error;
use wayland_sys::{DisplayErrorCode, wl_event_queue, wl_event_queue_destroy};

/// Owned event queue
pub struct WlEventQueue<S> {
    /// `None` for main event queue, `Some` for different event queue
    raw: Option<NonNull<wl_event_queue>>,
    storage: ManuallyDrop<WlObjectStorage<S>>,
}

unsafe impl<S> Send for WlEventQueue<S> {}
unsafe impl<S: Sync> Sync for WlEventQueue<S> {}

impl<S> WlEventQueue<S> {
    /// Creates side event queue
    pub fn side_from_display(display: &WlDisplay<S>) -> Result<Self, CreateQueueError>
    where
        S: State,
    {
        let raw = NonNull::new(display.create_event_queue_raw())
            .ok_or_else(|| CreateQueueError::BackendFailed(display.get_error_code().unwrap()))?;

        let mut storage = display.create_storage();

        unsafe { storage.set_raw_queue(raw) };

        Ok(Self {
            raw: Some(raw),
            storage: ManuallyDrop::new(storage),
        })
    }

    /// Creates main queue assocciated with `display`
    ///
    /// # Safety
    ///
    /// Should be called only once
    pub unsafe fn main_from_display(display: &WlDisplay<S>) -> Self
    where
        S: State,
    {
        Self {
            raw: None,
            storage: ManuallyDrop::new(display.create_storage()),
        }
    }

    /// Raw event queue.
    ///
    /// # Note
    ///
    /// Returns [`None`] if the queue is main.
    pub const fn as_raw(&self) -> Option<NonNull<wl_event_queue>> {
        self.raw
    }

    /// The queue is main (the display queue) or not
    pub const fn is_main(&self) -> bool {
        self.raw.is_none()
    }

    /// Projects pin of [`WlEventQueue`] to [`WlObjectStorage`]
    pub fn storage(self: Pin<&Self>) -> Pin<&WlObjectStorage<S>> {
        unsafe { Pin::map_unchecked(self, |this| &*this.storage) }
    }

    /// Projects mutable pin of [`WlEventQueue`] to [`WlObjectStorage`]
    pub fn storage_mut(self: Pin<&mut Self>) -> Pin<&mut WlObjectStorage<S>> {
        unsafe { Pin::map_unchecked_mut(self, |this| &mut *this.storage) }
    }
}

impl<S> Drop for WlEventQueue<S> {
    fn drop(&mut self) {
        // drop all proxies first
        unsafe { ManuallyDrop::drop(&mut self.storage) };

        if let Some(raw_queue) = self.raw {
            unsafe { wl_event_queue_destroy(raw_queue.as_ptr()) };
        }
    }
}

impl<S> fmt::Debug for WlEventQueue<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WlEventQueue")
            .field("storage", &*self.storage)
            .field("is_main", &self.is_main())
            .finish_non_exhaustive()
    }
}

/// Error creating an event queue
#[derive(Debug, Error)]
pub enum CreateQueueError {
    /// Error from libwayland
    #[error("`wl_display_create_queue` failed: {0:?}")]
    BackendFailed(DisplayErrorCode),
    /// Main queue taken already
    #[error("main queue is already taken")]
    MainTakenTwice,
}
