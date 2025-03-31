pub mod default_impl;
pub mod dispatch;
pub mod event_queue;
pub mod registry;

use super::{object_storage::WlObjectStorage, proxy::WlProxy, thin::ThinData, wire::MessageBuffer};
use crate::{
    ffi,
    interface::{ObjectParent, Request, send_request_raw},
    object::{HasObjectType, WlObjectId},
};
use dispatch::{
    Dispatch, WlDispatchData, WlDynDispatchData, dispatch_raw, is_empty_dispatch_data_allowed,
};
use std::{
    any::{self, TypeId},
    fmt, hash,
    marker::PhantomData,
    mem::{self, MaybeUninit, offset_of},
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::{self, NonNull},
};
use thiserror::Error;

/// A trait used to construct newly created object.
pub trait FromProxy: Sized {
    fn from_proxy(proxy: &WlProxy) -> Self;
}

/// Lightweight object handle with assocciated data type
pub struct WlObjectHandle<T> {
    id: WlObjectId,
    _p: PhantomData<T>,
}
static_assertions::assert_impl_all!(WlObjectHandle<()>: Send, Sync);

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

    /// Send the `request` to the server with several compile-time checks
    ///
    /// # Note
    ///
    /// Can be used only for requests which create no object
    pub fn request<'r, R>(
        self,
        buf: &mut impl MessageBuffer,
        storage: &WlObjectStorage<'_, T::State>,
        request: R,
    ) where
        T: Dispatch,
        R: Request<'r>,
    {
        const {
            assert!(
                T::OBJECT_TYPE as u32 == R::OBJECT_TYPE as u32,
                "request's parent interface should match the self type one's"
            );

            assert!(
                R::CHILD_TYPE.is_none(),
                "request's outgoing interface should be set to None",
            )
        };

        let proxy =
            unsafe { send_request_raw(request, buf, storage, storage.get_proxy(self.id).unwrap()) };

        debug_assert!(proxy.is_none());
    }

    /// Send the `request` to the server with several compile-time checks
    ///
    /// # Note
    ///
    /// Can be used only for object creating requests (e.g. wl_compositor::create_surface)
    pub fn create_object<'r, D, R>(
        self,
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage<'_, D::State>>,
        request: R,
    ) -> WlObjectHandle<D>
    where
        R: Request<'r> + ObjectParent,
        T: HasObjectType,
        D: Dispatch + FromProxy,
    {
        const {
            assert!(
                T::OBJECT_TYPE as u32 == R::OBJECT_TYPE as u32,
                "request's parent interface should match the self type one's"
            );

            match <R as Request>::CHILD_TYPE {
                Some(object_type) => assert!(
                    object_type as u32 == D::OBJECT_TYPE as u32,
                    "request's outgoing interface should match the return type one's"
                ),
                None => panic!("the request should have outgoing interface set to Some"),
            }
        };

        let proxy = unsafe {
            send_request_raw(
                request,
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
pub(crate) struct WlDynObject {
    pub(crate) proxy: WlProxy,
    pub(crate) type_id: TypeId,
}

impl WlDynObject {
    pub(crate) fn downcast_ref<T: Dispatch>(&self) -> Option<&WlObject<T>> {
        // # Safety
        //
        // - `WlDynObject` and `WlObject<T>` have the same header - `WlProxy`
        // - both structs are `repr(C)`
        (self.type_id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&WlDynObject, &WlObject<T>>(self) })
    }

    pub(crate) fn downcast_mut<T: Dispatch>(&mut self) -> Option<&mut WlObject<T>> {
        // # Safety
        //
        // - `WlDynObject` and `WlObject<T>` have the same header - `WlProxy`
        // - both structs are `repr(C)`
        (self.type_id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&mut WlDynObject, &mut WlObject<T>>(self) })
    }

    /// # Safety
    ///
    /// `storage` should point to a valid [`WlObjectStorage<S>`] with state `S`
    pub(crate) unsafe fn write_storage_location(&mut self, storage: *mut ()) {
        let Some(user_data_ptr) = NonNull::new(self.proxy.get_user_data()) else {
            return;
        };

        // # Safety
        //
        // - the `WlObject` always has valid user data being set if it was non-null
        // - we have exclusive access to the proxy object
        unsafe {
            user_data_ptr
                .as_ptr()
                .wrapping_byte_add(offset_of!(WlDynDispatchData, storage))
                .cast::<*mut ()>()
                .write(storage)
        };
    }
}

impl Drop for WlDynObject {
    fn drop(&mut self) {
        // Safety: `self.proxy` is a valid object produced by libwayland
        let Some(user_data) = NonNull::new(self.proxy.get_user_data()) else {
            return;
        };

        // # Safety
        //
        // - `user_data` points to `WlDispatchData<..>`
        // - `WlDynDispatchData` can be safely constructed from a pointer to `WlDispatchData<..>`
        let ptr = unsafe { WlDynDispatchData::from_ptr(user_data.cast()) };

        // Safety: `ptr` points to box-allocated memory
        _ = unsafe { Box::from_raw(ptr.as_ptr()) };
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
pub struct WlObject<T: Dispatch> {
    pub(crate) proxy: WlProxy,
    pub(crate) _p: PhantomData<T>,
}

unsafe impl<T: Dispatch + Send> Send for WlObject<T> {}
unsafe impl<T: Dispatch + Sync> Sync for WlObject<T> {}

impl<T: Dispatch> WlObject<T> {
    /// Constructs new [`WlObject`] with no data
    pub fn new_empty(proxy: WlProxy) -> Self {
        Self {
            proxy,
            _p: PhantomData,
        }
    }

    /// Constructs new [`WlObject`] from [`WlProxy`] object and data assocciated with it
    pub fn new(proxy: WlProxy, data: T) -> Self {
        if is_empty_dispatch_data_allowed::<T>() {
            return Self::new_empty(proxy);
        }

        let dispatch_data = Box::new(WlDispatchData::<T, T::State> {
            dispatch: T::dispatch,
            storage: None,
            state: None,
            data: ThinData::new(data),
        });

        let dispatch_data_ptr = Box::into_raw(dispatch_data);

        // Safety: `proxy` is a valid object provided by libwayland
        let result = unsafe {
            ffi::wl_proxy_add_dispatcher(
                proxy.as_raw().as_ptr(),
                dispatch_raw::<T, T::State>,
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

    pub(crate) fn write_storage_location(
        &mut self,
        storage: Pin<&mut WlObjectStorage<'_, T::State>>,
    ) {
        let user_data_ptr = self
            .proxy()
            .get_user_data()
            .cast::<WlDispatchData<T, T::State>>();

        // # Safety
        //
        // - the `WlObject` always has valid user data being set if it was non-null
        // - we have exclusive access to the proxy object
        let Some(user_data) = (unsafe { user_data_ptr.as_mut() }) else {
            return;
        };

        user_data.storage = Some(NonNull::from(unsafe { storage.get_unchecked_mut() }).cast());
    }

    pub(crate) fn write_state_location(&mut self, state: Pin<&mut T::State>) {
        let user_data_ptr = self
            .proxy()
            .get_user_data()
            .cast::<WlDispatchData<T, T::State>>();

        // # Safety
        //
        // - the `WlObject` always has valid user data being set if it was non-null
        // - we have exclusive access to the proxy object
        let Some(user_data) = (unsafe { user_data_ptr.as_mut() }) else {
            return;
        };

        user_data.state = Some(NonNull::from(unsafe { state.get_unchecked_mut() }));
    }

    /// Proxy
    pub fn proxy(&self) -> &WlProxy {
        &self.proxy
    }

    /// Proxy
    pub fn proxy_mut(&mut self) -> &mut WlProxy {
        &mut self.proxy
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
            type_id: TypeId::of::<T>(),
        }
    }

    /// Data assocciated with this object
    ///
    /// # Error
    ///
    /// Returns [`Err`] if no data was set and `T` is not ZST.
    pub fn data(&self) -> Result<&T, NonZstError> {
        let Some(user_data) = NonNull::new(self.proxy.get_user_data()) else {
            return zst_mut().map(|t| &*t);
        };

        // Safety: non-null `user_data` points to a valid instance of `WlDispatchData<T>`
        Ok(unsafe {
            &user_data
                .cast::<WlDispatchData<T, T::State>>()
                .as_ref()
                .data
                .inner
        })
    }

    /// Data assocciated with this object
    ///
    /// # Error
    ///
    /// Returns [`Err`] if no data was set and `T` is not ZST.
    pub fn data_mut(&mut self) -> Result<&mut T, NonZstError> {
        let Some(user_data) = NonNull::new(self.proxy.get_user_data()) else {
            return zst_mut();
        };

        // Safety: non-null `user_data` points to a valid instance of `WlDispatchData<T>`
        Ok(unsafe {
            &mut user_data
                .cast::<WlDispatchData<T, T::State>>()
                .as_mut()
                .data
                .inner
        })
    }
}

impl<T: Dispatch> Drop for WlObject<T> {
    fn drop(&mut self) {
        let Some(user_data) = NonNull::new(self.proxy.get_user_data()) else {
            return;
        };

        // # Safety
        //
        // - `user_data` points to a valid instance of `WlDispatchData<T>`
        // - drop called once on a valid instance
        unsafe {
            drop(Box::from_raw(
                user_data.as_ptr().cast::<WlDispatchData<T, T::State>>(),
            ))
        };
    }
}

/// Constructs mut reference to ZST type 'from a thin air'
const fn zst_mut<T>() -> Result<&'static mut T, NonZstError> {
    if mem::size_of::<T>() == 0 {
        // Safety: any non-null well-aligned reference is a valid reference to some ZST
        Ok(unsafe { NonNull::dangling().as_mut() })
    } else {
        Err(NonZstError)
    }
}

#[derive(Debug, Error)]
#[error("failed to construct reference to non-ZST value from nothing")]
pub struct NonZstError;

impl<T: Dispatch> Deref for WlObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data().unwrap()
    }
}

impl<T: Dispatch> DerefMut for WlObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut().unwrap()
    }
}

impl<T: fmt::Debug + Dispatch> fmt::Debug for WlObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("proxy", &self.proxy)
            .field("data", &self.data())
            .finish()
    }
}
