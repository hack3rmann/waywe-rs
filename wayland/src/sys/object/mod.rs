pub mod dispatch;
pub mod registry;
pub mod default_impl;

use super::{object_storage::WlObjectStorage, proxy::WlProxy, wire::MessageBuffer};
use crate::{
    interface::{ObjectParent, Request},
    object::{HasObjectType, WlObjectId},
};
use dispatch::{Dispatch, WlDispatchData, dispatch_raw};
use std::{
    any::{self, TypeId},
    fmt, hash,
    marker::PhantomData,
    mem::{self, MaybeUninit, offset_of},
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::{self, NonNull},
};
use wayland_sys::wl_proxy_add_dispatcher;

/// A trait used to construct newly created object.
pub trait FromProxy: Sized {
    fn from_proxy(proxy: &WlProxy) -> Self;
}

/// Lightweight object handle with assocciated data type
pub struct WlObjectHandle<T> {
    id: WlObjectId,
    _p: PhantomData<T>,
}

impl<T> WlObjectHandle<T> {
    /// Creates a handle to [`WlObject<T>`]
    pub const fn new(id: WlObjectId) -> Self {
        Self {
            id,
            _p: PhantomData,
        }
    }

    /// An object id for this handle
    pub const fn id(self) -> WlObjectId {
        self.id
    }

    /// Send the `request` to the server with a several compile-time checks
    ///
    /// # Note
    ///
    /// Can be used only for requests which create no object
    pub fn request<'r, R>(self, buf: &mut impl MessageBuffer, storage: &WlObjectStorage, request: R)
    where
        T: Dispatch + HasObjectType,
        R: Request<'r>,
    {
        const {
            assert!(
                T::OBJECT_TYPE as u32 == R::OBJECT_TYPE as u32,
                "request's parent interface should match the self type one's"
            );

            assert!(
                R::OUTGOING_INTERFACE.is_none(),
                "request's outgoing interface should be set to None",
            )
        };

        let proxy = unsafe { request.send(buf, storage, storage.get_proxy(self.id).unwrap()) };
        debug_assert!(proxy.is_none());
    }

    /// Send the `request` to the server with a several compile-time checks
    ///
    /// # Note
    ///
    /// Can be used only for object creating requests (e.g. wl_compositor::create_surface)
    pub fn create_object<'r, D, R>(
        self,
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage>,
        request: R,
    ) -> WlObjectHandle<D>
    where
        R: Request<'r> + ObjectParent,
        T: Dispatch + HasObjectType,
        D: Dispatch + HasObjectType + FromProxy,
    {
        const {
            assert!(
                T::OBJECT_TYPE as u32 == R::OBJECT_TYPE as u32,
                "request's parent interface should match the self type one's"
            );

            match R::OUTGOING_INTERFACE {
                Some(object_type) => assert!(
                    object_type as u32 == D::OBJECT_TYPE as u32,
                    "request's outgoing interface should match the return type one's"
                ),
                None => panic!("the request should have outgoing interface set to Some"),
            }
        };

        let proxy = unsafe {
            request
                .send(
                    buf,
                    storage.as_ref().get_ref(),
                    storage.get_proxy(self.id).unwrap(),
                )
                .unwrap()
        };

        let data = D::from_proxy(&proxy);
        storage.insert(WlObject::new(proxy, data))
    }
}

impl<T> Default for WlObjectHandle<T> {
    fn default() -> Self {
        Self::new(WlObjectId::default())
    }
}

impl<T> hash::Hash for WlObjectHandle<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&self.id, state);
    }
}

impl<T> Clone for WlObjectHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WlObjectHandle<T> {}

impl<T> fmt::Debug for WlObjectHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("id", &self.id)
            .finish()
    }
}

impl<T> PartialEq for WlObjectHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for WlObjectHandle<T> {}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Copy)]
pub(crate) struct TypeInfo {
    pub(crate) id: TypeId,
    pub(crate) drop: unsafe fn(*mut ()),
}

impl TypeInfo {
    pub(crate) fn of<T: 'static>() -> TypeInfo {
        TypeInfo {
            id: TypeId::of::<T>(),
            drop: |ptr: *mut ()| unsafe {
                ptr.cast::<T>().drop_in_place();
            },
        }
    }
}

// HACK(hack3rmann): dropping this type may cause memory leak
// Drop implementation for this type drops `WlDispatchData<()>`
// instead of `WlDispatchData<T>` with correct type `T`.
#[repr(C)]
pub(crate) struct WlDynObject {
    pub(crate) proxy: WlProxy,
    pub(crate) type_info: TypeInfo,
}

impl WlDynObject {
    pub(crate) fn downcast_ref<T: 'static>(&self) -> Option<&WlObject<T>> {
        // # Safety
        //
        // - `WlDynObject` and `WlObject<T>` have the same header - `WlProxy`
        // - both structs are `repr(C)`
        (self.type_info.id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&WlDynObject, &WlObject<T>>(self) })
    }

    pub(crate) fn downcast_mut<T: 'static>(&mut self) -> Option<&mut WlObject<T>> {
        // # Safety
        //
        // - `WlDynObject` and `WlObject<T>` have the same header - `WlProxy`
        // - both structs are `repr(C)`
        (self.type_info.id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&mut WlDynObject, &mut WlObject<T>>(self) })
    }
}

impl Drop for WlDynObject {
    fn drop(&mut self) {
        // Safety: `self.proxy` is a valid object produced by libwayland
        let user_data = self.proxy.get_user_data();

        let data_ptr = user_data
            .wrapping_byte_add(offset_of!(WlDispatchData<()>, data))
            .cast::<()>();

        // # Safety
        //
        // - `data_ptr` points to a valid location of `T`
        // - `drop` called once
        unsafe { (self.type_info.drop)(data_ptr) }
    }
}

impl fmt::Debug for WlDynObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("proxy", &self.proxy)
            .finish_non_exhaustive()
    }
}

#[repr(C)]
pub struct WlObject<T> {
    pub(crate) proxy: WlProxy,
    pub(crate) _p: PhantomData<T>,
}

impl<T: Dispatch + HasObjectType> WlObject<T> {
    pub fn new(proxy: WlProxy, data: T) -> Self {
        let dispatch_data = Box::new(WlDispatchData {
            dispatch: T::dispatch,
            storage: None,
            data,
        });

        let dispatch_data_ptr = Box::into_raw(dispatch_data);

        // Safety: `proxy` is a valid object provided by libwayland
        let result = unsafe {
            wl_proxy_add_dispatcher(
                proxy.as_raw().as_ptr(),
                dispatch_raw::<T>,
                ptr::null(),
                dispatch_data_ptr.cast(),
            )
        };

        if -1 == result {
            // Safety: `WlObject` have not constructed yet
            // therefore we should take care of the `Box` ourselves
            drop(unsafe { Box::from_raw(dispatch_data_ptr) });
            panic!("`wl_proxy_add_dispatcher` failed");
        }

        Self {
            proxy,
            _p: PhantomData,
        }
    }

    pub fn write_storage_location(&mut self, mut storage: Pin<&mut WlObjectStorage>) {
        let user_data_ptr = self.proxy().get_user_data().cast::<WlDispatchData<T>>();

        // Safety: the `WlObject` always has valid user data being set
        let user_data = unsafe { user_data_ptr.as_mut().unwrap_unchecked() };

        user_data.storage = Some(NonNull::new(&raw mut *storage).unwrap().cast());
    }

    pub fn proxy(&self) -> &WlProxy {
        &self.proxy
    }

    pub(crate) fn upcast(self) -> WlDynObject {
        // NOTE(hack3rmann): we can use `MaybeUninit` to
        // move out of `Self` which implements the `Drop` trait
        let mut this = MaybeUninit::new(self);

        WlDynObject {
            // Safety: here we moving out of `WlObject` without the destructor being called
            proxy: unsafe {
                this.as_mut_ptr()
                    .wrapping_byte_add(offset_of!(Self, proxy))
                    .cast::<WlProxy>()
                    .read()
            },
            type_info: TypeInfo::of::<T>(),
        }
    }
}

impl<T> Drop for WlObject<T> {
    fn drop(&mut self) {
        let user_data = self.proxy.get_user_data();

        // # Safety
        //
        // - `user_data` points to a valid instance of `WlDispatchData<T>`
        // - drop called once on a valid instance
        unsafe { drop(Box::from_raw(user_data.cast::<WlDispatchData<T>>())) };
    }
}

impl<T> Deref for WlObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let user_data = self.proxy.get_user_data();

        // Safety: `user_data` points to a valid instance of `WlDispatchData<T>`
        unsafe {
            &user_data
                .cast::<WlDispatchData<T>>()
                .as_ref()
                .unwrap_unchecked()
                .data
        }
    }
}

impl<T> DerefMut for WlObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let user_data = self.proxy.get_user_data();

        // Safety: `user_data` points to a valid instance of `WlDispatchData<T>`
        unsafe {
            &mut user_data
                .cast::<WlDispatchData<T>>()
                .as_mut()
                .unwrap_unchecked()
                .data
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for WlObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("proxy", &self.proxy)
            .field("data", self.deref())
            .finish()
    }
}
