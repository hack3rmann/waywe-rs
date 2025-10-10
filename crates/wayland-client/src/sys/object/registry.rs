//! Wayland `wl_registry` implementation

use super::{Dispatch, FromProxy, WlObject, WlObjectHandle, dispatch::State};
use crate::{
    NoState,
    interface::{
        WlObjectType, WlRegistryBindRequest, WlRegistryEvent, WlRegistryGlobalEvent,
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
use smallvec::{SmallVec, smallvec};
use static_assertions::assert_impl_all;
use std::{marker::PhantomData, mem, pin::Pin, str};

pub type WlRegistryDispatchFn<S> = fn(
    registry: &mut WlRegistry<S>,
    state: &S,
    storage: &mut WlObjectStorage<S>,
    event: WlRegistryEvent<'_>,
);

/// Numerical name and version of a global object
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryGlobalInfo {
    /// Numerical name for a global object
    pub name: WlObjectId,
    /// Version of a global object
    pub version: u32,
}
assert_impl_all!(WlRegistryGlobalInfo: Send, Sync);

pub type WlInterfaces = FxHashMap<WlObjectType, SmallVec<[WlRegistryGlobalInfo; 2]>>;
pub type WlNames = FxHashMap<WlObjectId, WlObjectType>;

/// Canonical `wl_registry` implementation
#[derive(Debug)]
pub struct WlRegistry<S> {
    pub(crate) interfaces: WlInterfaces,
    pub(crate) names: WlNames,
    pub(crate) dispatchers: SmallVec<[WlRegistryDispatchFn<S>; 2]>,
    pub(crate) handle: WlObjectHandle<Self>,
    pub(crate) _p: PhantomData<fn() -> S>,
}
assert_impl_all!(WlRegistry<NoState>: Send, Sync);

impl<S> WlRegistry<S> {
    /// Constructs new [`WlRegistry`]
    pub fn new(handle: WlObjectHandle<Self>) -> Self {
        Self {
            interfaces: WlInterfaces::default(),
            names: WlNames::default(),
            dispatchers: smallvec![],
            handle,
            _p: PhantomData,
        }
    }

    pub fn handle(&self) -> WlObjectHandle<Self> {
        self.handle
    }

    /// Adds dispatch function to registry
    pub fn with_dispatcher(&mut self, dispatcher: WlRegistryDispatchFn<S>) -> &mut Self {
        self.dispatchers.push(dispatcher);
        self
    }

    /// Interfaces of all registered global objects
    pub fn interfaces(&self) -> &WlInterfaces {
        &self.interfaces
    }

    /// Numerical name of global object of given type on a given index
    pub fn name_of_index(&self, object_type: WlObjectType, index: usize) -> Option<WlObjectId> {
        self.interfaces
            .get(&object_type)
            .and_then(|globals| globals.get(index))
            .map(|global| global.name)
    }

    /// Return the number of global objects of given type
    pub fn count_of(&self, object_type: WlObjectType) -> usize {
        self.interfaces
            .get(&object_type)
            .map(|globals| globals.len())
            .unwrap_or(0)
    }

    /// Numerical name of global object of given type on the first index (which always exists)
    pub fn name_of(&self, object_type: WlObjectType) -> Option<WlObjectId> {
        self.interfaces
            .get(&object_type)
            .map(|globals| {
                // Safety: `globals` contains at least one value
                unsafe { globals.first().unwrap_unchecked() }
            })
            .map(|global| global.name)
    }

    /// The type of global with id `name`
    pub fn type_of(&self, name: WlObjectId) -> Option<WlObjectType> {
        self.names.get(&name).copied()
    }

    /// # Safety
    ///
    /// `event.interface` should be a valid utf-8 string.
    pub(crate) unsafe fn handle_global_event(&mut self, event: WlRegistryGlobalEvent<'_>) {
        let interface = unsafe { str::from_utf8_unchecked(event.interface.to_bytes()) };

        let Some(ty) = WlObjectType::from_interface_name(interface) else {
            return;
        };

        let name = unsafe { WlObjectId::try_from(event.name).unwrap_unchecked() };
        let info = WlRegistryGlobalInfo {
            name,
            version: event.version,
        };

        self.interfaces
            .entry(ty)
            .and_modify(|globals| globals.push(info))
            .or_insert_with(|| smallvec![info]);

        _ = self.names.insert(name, ty);
    }

    pub(crate) fn handle_global_remove_event(&mut self, event: WlRegistryGlobalRemoveEvent) {
        let name = unsafe { WlObjectId::try_from(event.name).unwrap_unchecked() };

        let Some(&ty) = self.names.get(&name) else {
            return;
        };

        let Some(globals) = self.interfaces.get_mut(&ty) else {
            return;
        };

        match globals.len() {
            0 | 1 => _ = self.interfaces.remove(&ty),
            _ => {
                if let Some(index) = globals
                    .iter()
                    .enumerate()
                    .find_map(|(i, &info)| (info.name == name).then_some(i))
                {
                    _ = globals.swap_remove(index);
                }
            }
        }

        self.names.remove(&name);
    }
}

impl<S: State> WlObjectHandle<WlRegistry<S>> {
    /// Bind request on [`WlRegistry`]
    #[must_use]
    pub fn bind<T>(
        self,
        buf: &mut impl WlMessageBuffer,
        storage: Pin<&mut WlObjectStorage<S>>,
    ) -> Option<WlObjectHandle<T>>
    where
        T: Dispatch<State = S> + FromProxy,
    {
        self.bind_from_fn(buf, storage, 0, |_, _, proxy| T::from_proxy(proxy))
    }

    /// Bind request on [`WlRegistry`]
    #[must_use]
    pub fn bind_index<T>(
        self,
        buf: &mut impl WlMessageBuffer,
        storage: Pin<&mut WlObjectStorage<S>>,
        global_index: usize,
    ) -> Option<WlObjectHandle<T>>
    where
        T: Dispatch<State = S> + FromProxy,
    {
        self.bind_from_fn(buf, storage, global_index, |_, _, proxy| {
            T::from_proxy(proxy)
        })
    }

    pub fn bind_all<'s, T>(
        self,
        buf: &'s mut impl WlMessageBuffer,
        mut storage: Pin<&'s mut WlObjectStorage<S>>,
    ) -> Option<impl Iterator<Item = Option<WlObjectHandle<T>>> + 's>
    where
        T: Dispatch<State = S> + FromProxy,
    {
        let registry = storage.get_object(self)?;
        let count = registry.count_of(T::OBJECT_TYPE);

        Some((0..count).map(move |i| {
            self.bind_from_fn(buf, storage.as_mut(), i, |_, _, proxy| T::from_proxy(proxy))
        }))
    }

    /// Bind request on [`WlRegistry`] with given value
    #[must_use]
    pub fn bind_value<T>(
        self,
        buf: &mut impl WlMessageBuffer,
        storage: Pin<&mut WlObjectStorage<S>>,
        global_index: usize,
        object: T,
    ) -> Option<WlObjectHandle<T>>
    where
        T: Dispatch<State = S>,
    {
        self.bind_from_fn(buf, storage, global_index, |_, _, _| object)
    }

    /// Bind request on [`WlRegistry`] with given function providing value
    #[must_use]
    pub fn bind_from_fn<B, T, F>(
        self,
        buf: &mut B,
        storage: Pin<&mut WlObjectStorage<S>>,
        global_index: usize,
        make_data: F,
    ) -> Option<WlObjectHandle<T>>
    where
        B: WlMessageBuffer,
        T: Dispatch<State = S>,
        F: FnOnce(&mut B, Pin<&mut WlObjectStorage<S>>, &WlProxy) -> T,
    {
        let registry = storage.get_object(self)?;
        let global_name = registry.name_of_index(T::OBJECT_TYPE, global_index)?;

        self.bind_from_fn_by_id(buf, storage, global_name, make_data)
    }

    #[must_use]
    pub fn bind_from_fn_by_id<B, T, F>(
        self,
        buf: &mut B,
        mut storage: Pin<&mut WlObjectStorage<S>>,
        global_name: WlObjectId,
        make_data: F,
    ) -> Option<WlObjectHandle<T>>
    where
        B: WlMessageBuffer,
        T: Dispatch<State = S>,
        F: FnOnce(&mut B, Pin<&mut WlObjectStorage<S>>, &WlProxy) -> T,
    {
        let registry = storage.get_proxy(self.id()).unwrap();

        // Safety: `WlRegistry` is the parent for this request
        let proxy =
            unsafe { WlRegistryBindRequest::<T>::new().send::<S>(registry, buf, global_name)? };

        let data = make_data(buf, storage.as_mut(), &proxy);

        Some(storage.insert(WlObject::new(proxy, data)))
    }
}

impl<S> FromProxy for WlRegistry<S> {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self::new(WlObjectHandle::new(proxy.id()))
    }
}

impl<S> HasObjectType for WlRegistry<S> {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Registry;
}

impl<S: State> Dispatch for WlRegistry<S> {
    type State = S;

    fn dispatch(
        &mut self,
        state: &Self::State,
        storage: &mut WlObjectStorage<Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(event) = message.as_event::<WlRegistryEvent>() else {
            return;
        };

        let dispatchers = mem::take(&mut self.dispatchers);

        match event.clone() {
            WlRegistryEvent::Global(global) => {
                // Safety: `event.interface` is a valid utf-8 string,
                // it contains only valid ascii characters
                unsafe { self.handle_global_event(global) };

                for &dispatch in &dispatchers {
                    dispatch(self, state, storage, event.clone());
                }
            }
            WlRegistryEvent::GlobalRemove(remove) => {
                for &dispatch in &dispatchers {
                    dispatch(self, state, storage, event.clone());
                }

                self.handle_global_remove_event(remove);
            }
        }

        self.dispatchers = dispatchers;
    }
}
