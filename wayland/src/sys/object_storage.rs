use super::object::{Dispatch, WlDynObject, WlObject, WlObjectHandle};
use crate::object::ObjectId;
use std::collections::HashMap;

#[derive(Debug)]
pub struct WlObjectStorageEntry {
    pub object: WlDynObject,
}

#[derive(Default, Debug)]
pub struct WlObjectStorage {
    // NOTE(hack3rmann): this is a fast map as long as `ObjectId` hashes to itself
    objects: HashMap<ObjectId, WlObjectStorageEntry>,
}

impl WlObjectStorage {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn insert<T: Dispatch + 'static>(&mut self, object: WlObject<T>) {
        let _ = self.objects
            .insert(
                object.proxy().id(),
                WlObjectStorageEntry {
                    object: object.upcast(),
                },
            )
            .is_none_or(|_| panic!("map should not contain any object with this id"));
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

    pub fn object<T: Dispatch + 'static>(
        &self,
        handle: WlObjectHandle<T>,
    ) -> &WlObject<T> {
        self.get_object(handle).unwrap()
    }
}
