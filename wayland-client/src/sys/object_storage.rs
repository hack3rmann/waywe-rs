//! Collection of [`WlObject`]s.

use super::{
    display::WlDisplay,
    object::{WlDynObject, WlObject, WlObjectHandle, dispatch::Dispatch},
    proxy::WlProxy,
};
use crate::object::WlObjectId;
use fxhash::FxHashMap;
use std::{
    fmt,
    panic::{self, AssertUnwindSafe},
    pin::Pin,
    ptr::{self, NonNull},
};
use thiserror::Error;
use wayland_sys::wl_event_queue;

#[derive(Debug)]
pub(crate) struct WlObjectStorageEntry {
    pub object: WlDynObject,
}
static_assertions::assert_impl_all!(WlObjectStorageEntry: Send, Sync);

/// A storage for wayland's objects
pub struct WlObjectStorage<S> {
    objects: FxHashMap<WlObjectId, WlObjectStorageEntry>,
    acquired_object: Option<WlObjectId>,
    queue: Option<NonNull<wl_event_queue>>,
    display: WlDisplay<S>,
}

unsafe impl<S> Send for WlObjectStorage<S> {}
unsafe impl<S: Sync> Sync for WlObjectStorage<S> {}

impl<S> WlObjectStorage<S> {
    /// # Safety
    ///
    /// `WlObjectStorage` should be dropped before `WlDisplay`
    ///
    /// # Note
    ///
    /// The returned lifetime should be adjusted properly.
    pub fn new(display: WlDisplay<S>) -> Self {
        Self {
            objects: FxHashMap::default(),
            acquired_object: None,
            display,
            queue: None,
        }
    }

    /// Sets raw event queue this storage belongs to
    ///
    /// # Safety
    ///
    /// `raw_queue` should uniquely point to a valid event queue
    pub(crate) unsafe fn set_raw_queue(&mut self, raw_queue: NonNull<wl_event_queue>) {
        self.queue = Some(raw_queue);
    }

    /// Inserts a new object into the storage.
    ///
    /// # Note
    ///
    /// `self` is required to be 'pinned' because `insert`
    /// will write a pointer to `self` into the object's data.
    ///
    /// # Panic
    ///
    /// Panics if the storage already contains object with the same id.
    pub fn insert<T: Dispatch<State = S>>(
        mut self: Pin<&mut Self>,
        mut object: WlObject<T>,
    ) -> WlObjectHandle<T> {
        let id = object.proxy().id();

        if let Some(queue) = self.queue {
            // Safety: queue is valid
            unsafe { object.proxy_mut().set_queue_raw(queue.as_ptr()) };
        }

        object.write_storage_location(self.as_mut());
        object.write_state_location(unsafe {
            Pin::new_unchecked(self.display.shared.state.as_ref())
        });

        if self
            .objects
            .insert(
                id,
                WlObjectStorageEntry {
                    object: object.upcast(),
                },
            )
            .is_some()
        {
            panic!("map should not contain any object with this id");
        }

        WlObjectHandle::new(id)
    }

    /// Moves an object from this storage to another.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if
    ///
    /// - source storage does not contain object corresponding to given `handle`
    /// - destination storage already contains this object
    pub fn move_object<T: Dispatch<State = S>>(
        &mut self,
        mut target: Pin<&mut Self>,
        handle: WlObjectHandle<T>,
    ) -> Result<(), MoveObjectError> {
        let WlObjectStorageEntry { mut object } = self
            .objects
            .remove(&handle.id())
            .ok_or(MoveObjectError::NoObject(handle.id()))?;

        let queue_ptr = self.queue.map(NonNull::as_ptr).unwrap_or(ptr::null_mut());

        // Safety: queue is valid
        unsafe { object.proxy.set_queue_raw(queue_ptr) };

        // Safety: this pointer points to a valid storage with the right state `S`
        unsafe {
            object.write_storage_location((&raw const *target.as_ref().get_ref()).cast_mut().cast())
        };

        if target
            .objects
            .insert(object.proxy.id(), WlObjectStorageEntry { object })
            .is_some()
        {
            return Err(MoveObjectError::AlreadyPresent(handle.id()));
        }

        Ok(())
    }

    /// Searches an object in the storage by its handle.
    ///
    /// # Errors
    ///
    /// May return [`None`] if the object is not contained by the storage or
    /// it had been acquired via a call to [`WlObjectStorage::with_object_data_acquired`]
    pub fn get_object<T: Dispatch<State = S>>(
        &self,
        handle: WlObjectHandle<T>,
    ) -> Option<&WlObject<T>> {
        if let Some(id) = self.acquired_object
            && id == handle.id()
        {
            return None;
        }

        self.objects
            .get(&handle.id())
            .map(|e| &e.object)
            .and_then(WlDynObject::downcast_ref)
    }

    /// The same as [`WlObjectStorage::get_object`] but unwraps for you.
    pub fn object<T: Dispatch<State = S>>(&self, handle: WlObjectHandle<T>) -> &WlObject<T> {
        self.get_object(handle).unwrap()
    }

    /// A shorthand for `.object(handle).data().unwrap()`
    pub fn object_data<T: Dispatch<State = S>>(&self, handle: WlObjectHandle<T>) -> &T {
        self.object(handle).data().unwrap()
    }

    /// Searches an object in the storage by its handle.
    ///
    /// # Errors
    ///
    /// May return [`None`] if object does not contained by the storage or
    /// it had been acquired via a call to [`WlObjectStorage::with_object_data_acquired`]
    pub fn get_object_mut<T: Dispatch<State = S>>(
        &mut self,
        handle: WlObjectHandle<T>,
    ) -> Option<&mut WlObject<T>> {
        if let Some(id) = self.acquired_object
            && id == handle.id()
        {
            return None;
        }

        self.objects
            .get_mut(&handle.id())
            .map(|e| &mut e.object)
            .and_then(WlDynObject::downcast_mut)
    }

    /// The same as [`WlObjectStorage::get_object_mut`] but unwraps for you.
    pub fn object_mut<T: Dispatch<State = S>>(
        &mut self,
        handle: WlObjectHandle<T>,
    ) -> &WlObject<T> {
        self.get_object_mut(handle).unwrap()
    }

    /// Searches an object in the storage by its id and returns an assocciated [`WlProxy`].
    ///
    /// # Errors
    ///
    /// May return [`None`] if object does not contained by the storage.
    ///
    /// # Note
    ///
    /// In contrast to [`WlObjectStorage::get_object`] it does not fail
    /// when the object was acquired via a call to [`WlObjectStorage::with_object_data_acquired`]
    pub fn get_proxy(&self, id: WlObjectId) -> Option<&WlProxy> {
        self.objects.get(&id).map(|e| &e.object.proxy)
    }

    /// Releases all resources assocciated with given `handle`
    pub fn release<T: Dispatch<State = S>>(
        &mut self,
        handle: WlObjectHandle<T>,
    ) -> Result<(), NoEntryError<T>> {
        self.objects
            .remove(&handle.id())
            .ok_or(NoEntryError(handle))
            .map(|_| ())
    }

    /// Acquires object's data so no one can access its data for
    /// the entire duration of the `f` call.
    ///
    /// # Errors
    ///
    /// Will return [`Err`] if
    ///
    /// - this object had been acquired already.
    /// - acquired object id was corrupted after the `f` call.
    pub fn with_object_data_acquired(
        &mut self,
        id: WlObjectId,
        f: impl FnOnce(&mut Self),
    ) -> Result<(), ObjectDataAcquireError> {
        if self.acquired_object.replace(id).is_some() {
            return Err(ObjectDataAcquireError::AcquiredTwice);
        }

        // NOTE(hack3rmann): panic can lead to invalid state for the storage here.
        // If `f` has panicked `self.acquired_object` will not be `Some` after
        // returning from this function
        let panic_result = panic::catch_unwind(AssertUnwindSafe(|| f(self)));

        if self.acquired_object.take() != Some(id) {
            return Err(ObjectDataAcquireError::AcquiredIdCorruped);
        }

        if let Err(panic_payload) = panic_result {
            panic::resume_unwind(panic_payload);
        }

        Ok(())
    }
}

impl<S> fmt::Debug for WlObjectStorage<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WlObjectStorage")
            .field("objects", &self.objects)
            .finish_non_exhaustive()
    }
}

/// Corruption when acquireing object data
#[derive(Debug, Error)]
pub enum ObjectDataAcquireError {
    /// Acquireing twice in a row without releasing the data
    #[error("error acquireing object data twice")]
    AcquiredTwice,
    /// ID was corrupted during the call
    #[error("acquired object id was corrupted")]
    AcquiredIdCorruped,
}

/// Error moving object from one storage to another
#[derive(Debug, Error)]
pub enum MoveObjectError {
    /// No object present in a source queue
    #[error("no object with {0:?} is in the source storage")]
    NoObject(WlObjectId),
    /// Object was already present in destination queue
    #[error("object with {0:?} was already present in the destinations storage")]
    AlreadyPresent(WlObjectId),
}

/// No entry for a handle
#[derive(Error)]
#[error("no entry for {name}, with id = {id}", name = std::any::type_name::<T>(), id = u32::from(self.0.id()))]
pub struct NoEntryError<T>(pub WlObjectHandle<T>);

impl<T> fmt::Debug for NoEntryError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NoEntryError").field(&self.0).finish()
    }
}
