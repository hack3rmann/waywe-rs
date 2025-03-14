use std::pin::Pin;

use super::{Dispatch, FromProxy, WlObject, WlObjectHandle};
use crate::{
    interface::{Event as _, WlRegistryBindRequest, WlRegistryGlobalEvent},
    object::ObjectId,
    sys::{
        HasObjectType, ObjectType,
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{Message, MessageBuffer},
    },
};
use fxhash::FxHashMap;

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryGlobalInfo {
    pub name: ObjectId,
    pub version: u32,
}

#[derive(Debug, Default)]
pub struct WlRegistry {
    pub interfaces: FxHashMap<ObjectType, WlRegistryGlobalInfo>,
}

impl WlRegistry {
    pub fn bind<T>(
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage>,
        registry: WlObjectHandle<WlRegistry>,
        object: T,
    ) -> Option<WlObjectHandle<T>>
    where
        T: HasObjectType + Dispatch + 'static,
    {
        let proxy =
            unsafe { WlRegistryBindRequest::<T>::new().send(storage.get_object(registry)?, buf)? };

        Some(storage.insert(WlObject::new(proxy, object)))
    }

    pub fn bind_default<T>(
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage>,
        registry: WlObjectHandle<WlRegistry>,
    ) -> Option<WlObjectHandle<T>>
    where
        T: HasObjectType + Dispatch + Default + 'static,
    {
        Self::bind(buf, storage, registry, T::default())
    }

    pub fn name_of(&self, object_type: ObjectType) -> Option<ObjectId> {
        self.interfaces.get(&object_type).map(|global| global.name)
    }
}

impl FromProxy for WlRegistry {
    fn from_proxy(_: &WlProxy) -> Self {
        Self::default()
    }
}

impl Dispatch for WlRegistry {
    // TODO(hack3rmann): handle all events
    fn dispatch(&mut self, _storage: Pin<&mut WlObjectStorage>, message: Message<'_>) {
        let Some(event) = WlRegistryGlobalEvent::from_message(message) else {
            return;
        };

        let name = event
            .interface
            .to_str()
            .expect("interface name expected to be a valid utf-8 string");

        if let Some(ty) = ObjectType::from_interface_name(name) {
            self.interfaces.insert(
                ty,
                WlRegistryGlobalInfo {
                    name: event.name,
                    version: event.version,
                },
            );
        }
    }
}
