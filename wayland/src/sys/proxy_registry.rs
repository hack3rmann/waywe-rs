use super::proxy::WlProxy;
use crate::object::ObjectId;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub enum WlObjectType {
    Callback,
    Compositor,
    Display,
    Registry,
    Shm,
    ShmPool,
    Surface,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegistryEntry {
    pub proxy: WlProxy,
    pub object_type: WlObjectType,
}

#[derive(Debug, PartialEq, Default, Eq)]
pub struct ProxyRegistry {
    // NOTE(hack3rmann): this is a fast map as long as `ObjectId` hashes to itself
    proxies: HashMap<ObjectId, RegistryEntry>,
}

impl ProxyRegistry {
    pub fn new() -> Self {
        Self {
            proxies: HashMap::new(),
        }
    }

    pub fn insert(&mut self, object: WlProxy, ty: WlObjectType) {
        _ = self
            .proxies
            .insert(
                object.id(),
                RegistryEntry {
                    object_type: ty,
                    proxy: object,
                },
            )
            .is_none_or(|_| panic!("map should not contain any object with this id"));
    }

    pub fn get_proxy(&self, id: ObjectId) -> Option<&WlProxy> {
        self.proxies.get(&id).map(|e| &e.proxy)
    }

    pub fn get_type(&self, id: ObjectId) -> Option<WlObjectType> {
        self.proxies.get(&id).map(|e| e.object_type)
    }
}
