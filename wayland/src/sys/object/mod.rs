pub mod compositor;
pub mod registry;
pub mod surface;

use super::{
    ffi::{wl_argument, wl_message, wl_proxy_add_dispatcher, wl_proxy_get_user_data},
    proxy::WlProxy,
    wire::Message,
};
use crate::object::ObjectId;
use std::{
    any::{self, TypeId},
    ffi::{CStr, c_int, c_void},
    fmt, hash,
    marker::PhantomData,
    mem::{self, MaybeUninit, offset_of},
    ops::{Deref, DerefMut},
    process, ptr, slice,
};

pub trait Dispatch {
    fn dispatch(&mut self, message: Message<'_>);
}
static_assertions::assert_obj_safe!(Dispatch);

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
    std::panic::catch_unwind(|| {
        // Safety: `proxy` is valid object provided by libwayland
        let data = unsafe { wl_proxy_get_user_data(proxy.cast()) };

        // # Safety
        //
        // - `data` points to a valid box-allocated instance of `WlDispatchData`
        // - `data` only being used in dispatcher, libwayland provides exclusive access to the data
        let Some(data) = (unsafe { data.cast::<WlDispatchData<T>>().as_mut() }) else {
            return -1;
        };

        let Ok(opcode) = u16::try_from(opcode) else {
            return -1;
        };

        // # Safety
        //
        // - `message` points to a valid instance of `wl_message` (provided by libwayland)
        // - `message->signature` is a valid C-String (provided by libwayland)
        let signature = unsafe { CStr::from_ptr((*message).signature) };

        // Safety: libwayland provides all arguments according to the signature of
        // the event therefore there is exactly `signature.count_bytes()` arguments
        let arguments = unsafe { slice::from_raw_parts(arguments, signature.count_bytes()) };

        let message = Message { opcode, arguments };

        (data.dispatch)(&mut data.data, message);

        0
    })
    .unwrap_or_else(|_| {
        tracing::error!("panic in wl_dispatcher_func_t");
        process::abort();
    })
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

impl<T> Default for WlObjectHandle<T> {
    fn default() -> Self {
        Self::new(ObjectId::default())
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

#[derive(Clone, Debug, PartialEq, Copy)]
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
        // # Safety
        //
        // - `WlDynObject` and `WlObject<T>` have the same header - `WlProxy`
        // - both structs are `repr(C)`
        (self.type_info.id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&WlDynObject, &WlObject<T>>(self) })
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut WlObject<T>> {
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
        let user_data = unsafe { wl_proxy_get_user_data(self.proxy.as_raw().as_ptr()) };

        let data_ptr = user_data
            .wrapping_byte_add(offset_of!(WlDispatchData<()>, data))
            .cast::<()>();

        // # Safety
        //
        // - `data_ptr` points to valid `T` location
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

impl<T: Dispatch + 'static> WlObject<T> {
    pub fn new(proxy: WlProxy, data: T) -> Self {
        let dispatch_data = Box::new(WlDispatchData {
            dispatch: T::dispatch,
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

    pub fn proxy(&self) -> &WlProxy {
        &self.proxy
    }

    pub fn upcast(self) -> WlDynObject {
        // NOTE(hack3rmann): we can use `MaybeUninit` to
        // move out of `Self` which implements the `Drop` trait
        let mut this = MaybeUninit::new(self);

        WlDynObject {
            // Safety: here we moving out of `WlObject` without calling the destructor
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
        // Safety: `self.proxy` is valid object provided by libwayland
        let user_data = unsafe { wl_proxy_get_user_data(self.proxy.as_raw().as_ptr()) };

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
        // Safety: `self.proxy` is valid object provided by libwayland
        let user_data = unsafe { wl_proxy_get_user_data(self.proxy.as_raw().as_ptr()) };

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
        // Safety: `self.proxy` is valid object provided by libwayland
        let user_data = unsafe { wl_proxy_get_user_data(self.proxy.as_raw().as_ptr()) };

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

#[cfg(test)]
mod tests {
    use super::compositor::WlCompositor;
    use crate::{
        init::connect_wayland_socket,
        sys::{display::WlDisplay, wire::SmallVecMessageBuffer},
    };

    unsafe fn connect_display() -> WlDisplay {
        let wayland_sock = unsafe { connect_wayland_socket().unwrap() };
        WlDisplay::connect_to_fd(wayland_sock)
    }

    #[test]
    fn get_registry() {
        let mut buf = SmallVecMessageBuffer::<8>::new();

        // Safety: called once on the start of the program
        let display = unsafe { connect_display() };
        let registry = display.create_registry(&mut buf);

        display.dispatch_all();

        assert!(registry.interfaces.contains_key(c"wl_compositor"));
    }

    #[test]
    fn create_surface() {
        let mut buf = SmallVecMessageBuffer::<8>::new();

        // Safety: called once on the start of the program
        let display = unsafe { connect_display() };
        let mut registry = display.create_registry(&mut buf);

        display.dispatch_all();

        let compositor = registry.bind_default::<WlCompositor>(&mut buf).unwrap();
        let surface = WlCompositor::create_surface(&mut buf, &mut registry.storage, compositor);

        assert_eq!(
            registry.storage.object(surface).proxy().interface_name(),
            "wl_surface",
        );
    }
}
