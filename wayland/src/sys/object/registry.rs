use super::{Dispatch, FromProxy, WlObject, WlObjectHandle};
use crate::{
    interface::{Event as _, WlRegistryBindRequest, WlRegistryGlobalEvent},
    object::{HasObjectType, WlObjectId, WlObjectType},
    sys::{
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{WlMessage, MessageBuffer},
    },
};
use fxhash::FxHashMap;
use std::pin::Pin;

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryGlobalInfo {
    pub name: WlObjectId,
    pub version: u32,
}

#[derive(Debug, Default)]
pub struct WlRegistry {
    pub interfaces: FxHashMap<WlObjectType, WlRegistryGlobalInfo>,
}

impl WlRegistry {
    pub fn bind_value<T>(
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage>,
        registry: WlObjectHandle<WlRegistry>,
        object: T,
    ) -> Option<WlObjectHandle<T>>
    where
        T: HasObjectType + Dispatch,
    {
        let proxy =
            unsafe { WlRegistryBindRequest::<T>::new().send(storage.get_object(registry)?, buf)? };

        Some(storage.insert(WlObject::new(proxy, object)))
    }

    pub fn bind<T>(
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage>,
        registry: WlObjectHandle<WlRegistry>,
    ) -> Option<WlObjectHandle<T>>
    where
        T: HasObjectType + Dispatch + FromProxy,
    {
        let proxy =
            unsafe { WlRegistryBindRequest::<T>::new().send(storage.get_object(registry)?, buf)? };

        let data = T::from_proxy(&proxy);

        Some(storage.insert(WlObject::new(proxy, data)))
    }

    pub fn name_of(&self, object_type: WlObjectType) -> Option<WlObjectId> {
        self.interfaces.get(&object_type).map(|global| global.name)
    }
}

impl FromProxy for WlRegistry {
    fn from_proxy(_: &WlProxy) -> Self {
        Self::default()
    }
}

impl HasObjectType for WlRegistry {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Registry;
}

impl Dispatch for WlRegistry {
    // TODO(hack3rmann): handle all events
    fn dispatch(&mut self, _storage: Pin<&mut WlObjectStorage>, message: WlMessage<'_>) {
        let Some(event) = WlRegistryGlobalEvent::from_message(message) else {
            return;
        };

        let name = event
            .interface
            .to_str()
            .expect("interface name expected to be a valid utf-8 string");

        if let Some(ty) = WlObjectType::from_interface_name(name) {
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
