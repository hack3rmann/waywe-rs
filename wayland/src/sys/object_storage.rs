use super::{
    display::WlDisplay,
    object::{Dispatch, WlDynObject, WlObject, WlObjectHandle},
    proxy::WlProxy,
};
use crate::object::ObjectId;
use std::{collections::HashMap, marker::PhantomData, pin::Pin};

#[derive(Debug)]
pub struct WlObjectStorageEntry {
    pub object: WlDynObject,
}

#[derive(Debug)]
pub struct WlObjectStorage<'d> {
    // NOTE(hack3rmann): this is a fast map as long as `ObjectId` hashes to itself
    pub objects: HashMap<ObjectId, WlObjectStorageEntry>,
    pub acquired_object: Option<ObjectId>,
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
            acquired_object: None,
            _display: PhantomData,
        }
    }

    pub fn insert<T: Dispatch + 'static>(
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

    pub fn get_object<T: Dispatch + 'static>(
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
            .and_then(|o| o.downcast_ref())
    }

    pub fn object<T: Dispatch + 'static>(&self, handle: WlObjectHandle<T>) -> &WlObject<T> {
        self.get_object(handle).unwrap()
    }

    pub fn get_proxy(&self, id: ObjectId) -> Option<&WlProxy> {
        self.objects.get(&id).map(|e| &e.object.proxy)
    }

    pub fn get_object_mut<T: Dispatch + 'static>(
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
            .and_then(|o| o.downcast_mut())
    }

    pub fn object_mut<T: Dispatch + 'static>(&mut self, handle: WlObjectHandle<T>) -> &WlObject<T> {
        self.get_object_mut(handle).unwrap()
    }

    pub fn with_object_data_acquired(
        mut self: Pin<&mut Self>,
        id: ObjectId,
        f: impl FnOnce(Pin<&mut Self>),
    ) {
        _ = self.acquired_object.insert(id);
        f(self.as_mut());
        _ = self.acquired_object.take();
    }
}
