use super::{Dispatch, FromProxy, WlObject, WlObjectHandle, dispatch::State};
use crate::{
    interface::{Event as _, WlRegistryBindRequest, WlRegistryGlobalEvent},
    object::{HasObjectType, WlObjectId, WlObjectType},
    sys::{
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{MessageBuffer, WlMessage},
    },
};
use fxhash::FxHashMap;
use std::{marker::PhantomData, pin::Pin};

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryGlobalInfo {
    pub name: WlObjectId,
    pub version: u32,
}

#[derive(Debug)]
pub struct WlRegistry<S> {
    pub(crate) interfaces: FxHashMap<WlObjectType, WlRegistryGlobalInfo>,
    pub(crate) _p: PhantomData<*const S>,
}

impl<S: State> WlRegistry<S> {
    pub fn new() -> Self {
        Self {
            interfaces: FxHashMap::default(),
            _p: PhantomData,
        }
    }

    pub fn interfaces(&self) -> &FxHashMap<WlObjectType, WlRegistryGlobalInfo> {
        &self.interfaces
    }

    pub fn bind_value<T>(
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage<'_, S>>,
        registry: WlObjectHandle<WlRegistry<S>>,
        object: T,
    ) -> Option<WlObjectHandle<T>>
    where
        T: HasObjectType + Dispatch<State = S>,
    {
        let proxy =
            unsafe { WlRegistryBindRequest::<T>::new().send(storage.get_object(registry)?, buf)? };

        Some(storage.insert(WlObject::new(proxy, object)))
    }

    pub fn bind<T>(
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage<'_, S>>,
        registry: WlObjectHandle<WlRegistry<S>>,
    ) -> Option<WlObjectHandle<T>>
    where
        T: HasObjectType + Dispatch<State = S> + FromProxy,
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

impl<S: State> Default for WlRegistry<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: State> FromProxy for WlRegistry<S> {
    fn from_proxy(_: &WlProxy) -> Self {
        Self::new()
    }
}

impl<S> HasObjectType for WlRegistry<S> {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Registry;
}

impl<S: State> Dispatch for WlRegistry<S> {
    type State = S;

    // TODO(hack3rmann): handle all events
    fn dispatch(
        &mut self,
        _state: Pin<&mut Self::State>,
        _storage: Pin<&mut WlObjectStorage<'_, Self::State>>,
        message: WlMessage<'_>,
    ) {
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
