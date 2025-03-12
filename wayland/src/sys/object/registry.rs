use fxhash::FxHashMap;

use super::{Dispatch, WlObject, WlObjectHandle};
use crate::{
    interface::{
        Event as _, Request, WlRegistryBindRequest, WlRegistryGlobalEvent,
        registry::request::HasInterface,
    },
    object::ObjectId,
    sys::{
        ObjectType,
        object_storage::WlObjectStorage,
        wire::{Message, MessageBuffer},
    },
};

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
        storage: &mut WlObjectStorage,
        registry: WlObjectHandle<WlRegistry>,
        object: T,
    ) -> Option<WlObjectHandle<T>>
    where
        T: HasInterface + Dispatch + 'static,
    {
        let proxy = unsafe {
            WlRegistryBindRequest::<T>::new().send(storage.get_object(registry)?.proxy(), buf)?
        };

        Some(storage.insert(WlObject::new(proxy, object)))
    }

    pub fn bind_default<T>(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
        registry: WlObjectHandle<WlRegistry>,
    ) -> Option<WlObjectHandle<T>>
    where
        T: HasInterface + Dispatch + Default + 'static,
    {
        Self::bind(buf, storage, registry, T::default())
    }
}

impl Dispatch for WlRegistry {
    // TODO(hack3rmann): handle all events
    fn dispatch(&mut self, message: Message<'_>) {
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
