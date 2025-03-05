use super::{Dispatch, WlObject, WlObjectHandle};
use crate::{
    interface::{
        Event as _, Request, WlRegistryBindRequest, WlRegistryGlobalEvent,
        registry::request::HasInterface,
    },
    object::ObjectId,
    sys::{
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{Message, MessageBuffer},
    },
};
use std::{collections::HashMap, ffi::CString, ptr::NonNull};

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryGlobalInfo {
    pub name: ObjectId,
    pub version: u32,
}

#[derive(Debug)]
pub struct WlRegistry<'d> {
    // TODO(hack3rmann): make it faster
    pub interfaces: HashMap<CString, WlRegistryGlobalInfo>,
    pub storage: WlObjectStorage<'d>,
}

impl WlObject<WlRegistry<'_>> {
    pub fn bind<T>(&mut self, buf: &mut impl MessageBuffer, object: T) -> Option<WlObjectHandle<T>>
    where
        T: HasInterface + Dispatch + 'static,
    {
        let raw_proxy = unsafe { WlRegistryBindRequest::<T>::new().send_raw(&self.proxy, buf) };
        let proxy = unsafe { WlProxy::from_raw(NonNull::new(raw_proxy)?) };

        Some(self.storage.insert(WlObject::new(proxy, object)))
    }

    pub fn bind_default<T>(&mut self, buf: &mut impl MessageBuffer) -> Option<WlObjectHandle<T>>
    where
        T: HasInterface + Dispatch + Default + 'static,
    {
        self.bind(buf, T::default())
    }
}

impl Dispatch for WlRegistry<'_> {
    // TODO(hack3rmann): handle all events
    fn dispatch(&mut self, message: Message<'_>) {
        let Some(event) = WlRegistryGlobalEvent::from_message(message) else {
            return;
        };

        self.interfaces.insert(
            event.interface.to_owned(),
            WlRegistryGlobalInfo {
                name: event.name,
                version: event.version,
            },
        );
    }
}
