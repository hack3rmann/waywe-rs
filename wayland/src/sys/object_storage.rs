use super::{
    display::WlDisplay,
    object::{Dispatch, WlDynObject, WlObject, WlObjectHandle},
};
use crate::object::ObjectId;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Debug)]
pub struct WlObjectStorageEntry {
    pub object: WlDynObject,
}

#[derive(Debug)]
pub struct WlObjectStorage<'d> {
    // NOTE(hack3rmann): this is a fast map as long as `ObjectId` hashes to itself
    pub objects: HashMap<ObjectId, WlObjectStorageEntry>,
    pub _display: PhantomData<&'d WlDisplay>,
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
    pub unsafe fn new() -> Self {
        Self {
            objects: HashMap::new(),
            _display: PhantomData,
        }
    }

    pub fn insert<T: Dispatch + 'static>(&mut self, object: WlObject<T>) -> WlObjectHandle<T> {
        let id = object.proxy().id();

        let _ = self
            .objects
            .insert(
                id,
                WlObjectStorageEntry {
                    object: object.upcast(),
                },
            )
            .is_none_or(|_| panic!("map should not contain any object with this id"));

        WlObjectHandle::new(id)
    }

    pub fn get_object<T: Dispatch + 'static>(
        &self,
        handle: WlObjectHandle<T>,
    ) -> Option<&WlObject<T>> {
        self.objects
            .get(&handle.id)
            .map(|e| &e.object)
            .and_then(|o| o.downcast_ref())
    }

    pub fn object<T: Dispatch + 'static>(&self, handle: WlObjectHandle<T>) -> &WlObject<T> {
        self.get_object(handle).unwrap()
    }

    pub fn get_object_mut<T: Dispatch + 'static>(
        &mut self,
        handle: WlObjectHandle<T>,
    ) -> Option<&mut WlObject<T>> {
        self.objects
            .get_mut(&handle.id)
            .map(|e| &mut e.object)
            .and_then(|o| o.downcast_mut())
    }

    pub fn object_mut<T: Dispatch + 'static>(&mut self, handle: WlObjectHandle<T>) -> &WlObject<T> {
        self.get_object_mut(handle).unwrap()
    }
}
