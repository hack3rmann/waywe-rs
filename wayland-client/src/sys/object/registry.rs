//! Wayland `wl_registry` implementation

use super::{Dispatch, FromProxy, WlObject, WlObjectHandle, dispatch::State};
use crate::{
    NoState,
    interface::{
        Event, WlObjectType, WlRegistryBindRequest, WlRegistryGlobalEvent,
        WlRegistryGlobalRemoveEvent,
    },
    object::{HasObjectType, WlObjectId},
    sys::{
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{WlMessage, WlMessageBuffer},
    },
};
use fxhash::FxHashMap;
use std::{marker::PhantomData, pin::Pin};

/// Numerical name and version of a global object
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryGlobalInfo {
    /// Numerical name for a global object
    pub name: WlObjectId,
    /// Version of a global object
    pub version: u32,
}
static_assertions::assert_impl_all!(WlRegistryGlobalInfo: Send, Sync);

/// Canonical `wl_registry` implementation
#[derive(Debug)]
pub struct WlRegistry<S> {
    pub(crate) interfaces: FxHashMap<WlObjectType, WlRegistryGlobalInfo>,
    pub(crate) _p: PhantomData<fn() -> S>,
}
static_assertions::assert_impl_all!(WlRegistry<NoState>: Send, Sync);

impl<S: State> WlRegistry<S> {
    /// Constructs new [`WlRegistry`]
    pub fn new() -> Self {
        Self {
            interfaces: FxHashMap::default(),
            _p: PhantomData,
        }
    }

    /// Interfaces of all registered global objects
    pub fn interfaces(&self) -> &FxHashMap<WlObjectType, WlRegistryGlobalInfo> {
        &self.interfaces
    }

    /// Numerical name of global object of given type
    pub fn name_of(&self, object_type: WlObjectType) -> Option<WlObjectId> {
        self.interfaces.get(&object_type).map(|global| global.name)
    }

    /// # Safety
    ///
    /// `event.interface` should be a valid utf-8 string.
    pub(crate) unsafe fn handle_global_event(&mut self, event: WlRegistryGlobalEvent<'_>) {
        let interface = unsafe { std::str::from_utf8_unchecked(event.interface.to_bytes()) };

        let Some(ty) = WlObjectType::from_interface_name(interface) else {
            return;
        };

        self.interfaces.insert(
            ty,
            WlRegistryGlobalInfo {
                name: unsafe { WlObjectId::try_from(event.name).unwrap_unchecked() },
                version: event.version,
            },
        );
    }

    pub(crate) fn handle_global_remove_event(&mut self, event: WlRegistryGlobalRemoveEvent) {
        let Some(ty) = self
            .interfaces
            .iter()
            .find_map(|(&ty, &entry)| (event.name == entry.name.into()).then_some(ty))
        else {
            return;
        };

        self.interfaces.remove(&ty);
    }
}

impl<S: State> WlObjectHandle<WlRegistry<S>> {
    /// Bind request on [`WlRegistry`]
    pub fn bind<T>(
        self,
        buf: &mut impl WlMessageBuffer,
        storage: Pin<&mut WlObjectStorage<'_, S>>,
    ) -> Option<WlObjectHandle<T>>
    where
        T: Dispatch<State = S> + FromProxy,
    {
        self.bind_from_fn(buf, storage, |_, _, proxy| T::from_proxy(proxy))
    }

    /// Bind request on [`WlRegistry`] with given value
    pub fn bind_value<T>(
        self,
        buf: &mut impl WlMessageBuffer,
        storage: Pin<&mut WlObjectStorage<'_, S>>,
        object: T,
    ) -> Option<WlObjectHandle<T>>
    where
        T: Dispatch<State = S>,
    {
        self.bind_from_fn(buf, storage, |_, _, _| object)
    }

    /// Bind request on [`WlRegistry`] with given function providing value
    pub fn bind_from_fn<B, T, F>(
        self,
        buf: &mut B,
        mut storage: Pin<&mut WlObjectStorage<'_, S>>,
        make_data: F,
    ) -> Option<WlObjectHandle<T>>
    where
        B: WlMessageBuffer,
        T: Dispatch<State = S>,
        F: FnOnce(&mut B, Pin<&mut WlObjectStorage<'_, S>>, &WlProxy) -> T,
    {
        // Safety: `WlRegistry` is the parent for this request
        let proxy =
            unsafe { WlRegistryBindRequest::<T>::new().send(storage.get_object(self)?, buf)? };

        let data = make_data(buf, storage.as_mut(), &proxy);

        Some(storage.insert(WlObject::new(proxy, data)))
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

    fn dispatch(
        &mut self,
        _: &Self::State,
        _: &mut WlObjectStorage<'_, Self::State>,
        message: WlMessage<'_>,
    ) {
        match message.opcode {
            WlRegistryGlobalEvent::CODE => {
                let event = unsafe {
                    message
                        .as_event::<WlRegistryGlobalEvent>()
                        .unwrap_unchecked()
                };
                // Safety: `event.interface` is a valid utf-8 string,
                // it contains only valid ascii characters
                unsafe { self.handle_global_event(event) };
            }
            WlRegistryGlobalRemoveEvent::CODE => {
                let event = unsafe {
                    message
                        .as_event::<WlRegistryGlobalRemoveEvent>()
                        .unwrap_unchecked()
                };
                self.handle_global_remove_event(event);
            }
            _ => {}
        }
    }
}
