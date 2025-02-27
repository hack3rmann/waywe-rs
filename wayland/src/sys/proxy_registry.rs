use super::proxy::{AsProxy, WlProxy, WlProxyBorrow};
use crate::object::ObjectId;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Default, Eq)]
pub struct ProxyRegistry {
    // NOTE(hack3rmann): this is a fast map as long as `ObjectId` hashes to itself
    map: HashMap<ObjectId, WlProxy>,
}

impl ProxyRegistry {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn insert(&mut self, object: WlProxy) {
        self.map.insert(object.id(), object)
            .expect("map should not contain any object with this id");
    }

    pub fn get(&self, id: ObjectId) -> Option<WlProxyBorrow<'_>> {
        self.map.get(&id).map(AsProxy::as_proxy)
    }
}
