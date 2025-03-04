use crate::object::ObjectId;

use super::{
    ffi::{wl_argument, wl_message, wl_proxy_add_dispatcher, wl_proxy_get_user_data},
    proxy::WlProxy,
    wire::Message,
};
use std::{
    any::{self, TypeId},
    ffi::{CStr, c_int, c_void},
    fmt,
    marker::PhantomData,
    mem::{self, MaybeUninit, offset_of},
    ops::{Deref, DerefMut},
    ptr, slice,
};

pub trait Dispatch {
    fn dispatch(&mut self, message: Message<'_>);
}

pub type WlDispatchFn<T> = fn(&mut T, Message<'_>);

pub struct WlDispatchData<T> {
    pub dispatch: WlDispatchFn<T>,
    pub data: T,
}

unsafe extern "C" fn dispatch_raw<T>(
    _impl: *const c_void,
    proxy: *mut c_void,
    opcode: u32,
    message: *const wl_message,
    arguments: *mut wl_argument,
) -> c_int {
    let data = unsafe { wl_proxy_get_user_data(proxy.cast()) };

    let Some(data) = (unsafe { data.cast::<WlDispatchData<T>>().as_mut() }) else {
        return 1;
    };

    let Ok(opcode) = u16::try_from(opcode) else {
        return 1;
    };

    let signature = unsafe { CStr::from_ptr((*message).signature) };
    let arguments = unsafe { slice::from_raw_parts(arguments, signature.count_bytes()) };

    let message = Message { opcode, arguments };

    // FIXME(hack3rmann): catch unwind here
    (data.dispatch)(&mut data.data, message);

    0
}

pub struct WlObjectHandle<T> {
    pub(crate) id: ObjectId,
    pub(crate) _p: PhantomData<T>,
}

impl<T> WlObjectHandle<T> {
    pub const fn new(id: ObjectId) -> Self {
        Self {
            id,
            _p: PhantomData,
        }
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

pub struct TypeInfo {
    pub id: TypeId,
    pub drop: unsafe fn(*mut ()),
}

impl TypeInfo {
    pub fn of<T: 'static>() -> TypeInfo {
        TypeInfo {
            id: TypeId::of::<T>(),
            drop: |ptr: *mut ()| unsafe {
                ptr.cast::<T>().drop_in_place();
            },
        }
    }
}

#[repr(C)]
pub struct WlDynObject {
    pub(crate) proxy: WlProxy,
    pub(crate) type_info: TypeInfo,
}

impl WlDynObject {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&WlObject<T>> {
        (self.type_info.id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&WlDynObject, &WlObject<T>>(self) })
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut WlObject<T>> {
        (self.type_info.id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&mut WlDynObject, &mut WlObject<T>>(self) })
    }
}

impl Drop for WlDynObject {
    fn drop(&mut self) {
        let user_data = unsafe { wl_proxy_get_user_data(self.proxy.as_raw().as_ptr()) };
        let data_ptr = user_data
            .wrapping_byte_add(offset_of!(WlDispatchData<()>, data))
            .cast::<()>();
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
    pub(crate) _p: PhantomData<Box<T>>,
}

impl<T: Dispatch + 'static> WlObject<T> {
    pub fn new(proxy: WlProxy, data: T) -> Self {
        let dispatch_data = Box::new(WlDispatchData {
            dispatch: T::dispatch,
            data,
        });

        let result = unsafe {
            wl_proxy_add_dispatcher(
                proxy.as_raw().as_ptr(),
                dispatch_raw::<T>,
                ptr::null(),
                Box::into_raw(dispatch_data).cast(),
            )
        };

        assert_ne!(result, -1, "`wl_proxy_add_dispatcher` failed");

        Self {
            proxy,
            _p: PhantomData,
        }
    }

    pub fn proxy(&self) -> &WlProxy {
        &self.proxy
    }

    pub fn upcast(self) -> WlDynObject {
        // NOTE(hack3rmann): we can use `MaybeUninit` to
        // move out of `Self` which implements the `Drop` trait
        let mut this = MaybeUninit::new(self);

        WlDynObject {
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
        let user_data = unsafe { wl_proxy_get_user_data(self.proxy.as_raw().as_ptr()) };
        unsafe { drop(Box::from_raw(user_data.cast::<WlDispatchData<T>>())) };
    }
}

impl<T> Deref for WlObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let user_data = unsafe { wl_proxy_get_user_data(self.proxy.as_raw().as_ptr()) };

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
        let user_data = unsafe { wl_proxy_get_user_data(self.proxy.as_raw().as_ptr()) };

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
