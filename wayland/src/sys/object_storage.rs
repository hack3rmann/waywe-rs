use super::{
    display::WlDisplay,
    object::{dispatch::Dispatch, WlDynObject, WlObject, WlObjectHandle},
    proxy::WlProxy,
};
use crate::object::ObjectId;
use std::{collections::HashMap, marker::PhantomData, pin::Pin};
use thiserror::Error;

#[derive(Debug)]
pub(crate) struct WlObjectStorageEntry {
    pub(crate) object: WlDynObject,
}

/// A storage for wayland's objects
#[derive(Debug)]
pub struct WlObjectStorage<'d> {
    // NOTE(hack3rmann): this is a fast map as long as `ObjectId` hashes to itself
    objects: HashMap<ObjectId, WlObjectStorageEntry>,
    acquired_object: Option<ObjectId>,
    _display: PhantomData<&'d WlDisplay>,
}

// Safety: empty drop implementation ensures that `WlObjectStorage` uses
// `_display` reference
impl Drop for WlObjectStorage<'_> {
    fn drop(&mut self) {}
}

impl WlObjectStorage<'_> {
    /// # Safety
    ///
    /// `WlObjectStorage` should be dropped before `WlDisplay`
    ///
    /// # Note
    ///
    /// The returned lifetime should be adjusted properly.
    pub unsafe fn new() -> Self {
        Self {
            objects: HashMap::new(),
            acquired_object: None,
            _display: PhantomData,
        }
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
    pub fn insert<T: Dispatch>(
        mut self: Pin<&mut Self>,
        mut object: WlObject<T>,
    ) -> WlObjectHandle<T> {
        let id = object.proxy().id();

        object.write_storage_location(self.as_mut());

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

    /// Searches an object in the storage by its handle.
    ///
    /// # Errors
    ///
    /// May return [`None`] if object does not contained by the storage or
    /// it had been acquired via a call to [`WlObjectStorage::with_object_data_acquired`]
    pub fn get_object<T: Dispatch>(
        &self,
        handle: WlObjectHandle<T>,
    ) -> Option<&WlObject<T>> {
        if let Some(id) = self.acquired_object {
            if id == handle.id {
                return None;
            }
        }

        self.objects
            .get(&handle.id)
            .map(|e| &e.object)
            .and_then(WlDynObject::downcast_ref)
    }

    /// The same as [`WlObjectStorage::get_object`] but unwraps for you.
    pub fn object<T: Dispatch>(&self, handle: WlObjectHandle<T>) -> &WlObject<T> {
        self.get_object(handle).unwrap()
    }

    /// Searches an object in the storage by its handle.
    ///
    /// # Errors
    ///
    /// May return [`None`] if object does not contained by the storage or
    /// it had been acquired via a call to [`WlObjectStorage::with_object_data_acquired`]
    pub fn get_object_mut<T: Dispatch>(
        &mut self,
        handle: WlObjectHandle<T>,
    ) -> Option<&mut WlObject<T>> {
        if let Some(id) = self.acquired_object {
            if id == handle.id {
                return None;
            }
        }

        self.objects
            .get_mut(&handle.id)
            .map(|e| &mut e.object)
            .and_then(WlDynObject::downcast_mut)
    }

    /// The same as [`WlObjectStorage::get_object_mut`] but unwraps for you.
    pub fn object_mut<T: Dispatch>(&mut self, handle: WlObjectHandle<T>) -> &WlObject<T> {
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
    pub fn get_proxy(&self, id: ObjectId) -> Option<&WlProxy> {
        self.objects.get(&id).map(|e| &e.object.proxy)
    }

    /// Releases all resources assocciated with given `handle`
    pub fn release<T: Dispatch>(
        &mut self,
        handle: WlObjectHandle<T>,
    ) -> Result<(), NoEntryError<T>> {
        self.objects
            .remove(&handle.id)
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
    /// - this object has acquired already.
    /// - acquired object id has corrupted after the `f` call.
    pub fn with_object_data_acquired(
        mut self: Pin<&mut Self>,
        id: ObjectId,
        f: impl FnOnce(Pin<&mut Self>),
    ) -> Result<(), ObjectDataAcquireError> {
        if self.acquired_object.replace(id).is_some() {
            return Err(ObjectDataAcquireError::AcquiredTwice);
        }

        f(self.as_mut());

        if self.acquired_object.take() != Some(id) {
            return Err(ObjectDataAcquireError::AcquiredIdCorruped);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ObjectDataAcquireError {
    #[error("error acquireing object data twice")]
    AcquiredTwice,
    #[error("acquired object id was corrupted")]
    AcquiredIdCorruped,
}

#[derive(Debug, Error)]
#[error("no entry for {name}, with id = {id}", name = std::any::type_name::<T>(), id = u32::from(self.0.id))]
pub struct NoEntryError<T>(pub WlObjectHandle<T>);
