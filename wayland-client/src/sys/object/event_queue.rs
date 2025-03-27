use super::dispatch::State;
use crate::{WlDisplay, WlObjectStorage};
use std::{pin::Pin, ptr::NonNull};
use thiserror::Error;
use wayland_sys::{wl_event_queue, wl_event_queue_destroy};

pub struct WlEventQueue<'d, S: State> {
    /// `None` for main event queue, `Some` for different event queue
    raw: Option<NonNull<wl_event_queue>>,
    storage: WlObjectStorage<'d, S>,
}

impl<'d, S: State> WlEventQueue<'d, S> {
    pub unsafe fn from_display(display: &'d WlDisplay<S>) -> Result<Self, CreateQueueError> {
        let raw = NonNull::new(display.create_event_queue_unchecked()).ok_or(CreateQueueError)?;

        let mut storage = display.create_storage();

        unsafe { storage.set_raw_queue(raw) };

        Ok(Self {
            raw: Some(raw),
            storage,
        })
    }

    pub unsafe fn main_from_display(display: &'d WlDisplay<S>) -> Result<Self, CreateQueueError> {
        Ok(Self {
            raw: None,
            storage: display.create_storage(),
        })
    }

    pub fn as_raw(&self) -> Option<NonNull<wl_event_queue>> {
        self.raw
    }

    pub fn is_main(&self) -> bool {
        self.raw.is_none()
    }

    pub fn storage(self: Pin<&Self>) -> Pin<&WlObjectStorage<'d, S>> {
        unsafe { Pin::map_unchecked(self, |this| &this.storage) }
    }

    pub fn storage_mut(self: Pin<&mut Self>) -> Pin<&mut WlObjectStorage<'d, S>> {
        unsafe { Pin::map_unchecked_mut(self, |this| &mut this.storage) }
    }
}

impl<S: State> Drop for WlEventQueue<'_, S> {
    fn drop(&mut self) {
        if let Some(raw_queue) = self.raw {
            unsafe { wl_event_queue_destroy(raw_queue.as_ptr()) };
        }
    }
}

#[derive(Debug, Error)]
#[error("`wl_display_create_queue` failed")]
pub struct CreateQueueError;
