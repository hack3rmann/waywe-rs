// FIXME(hack3rmann): remove `allow(missing_safety_doc)`
#![allow(clippy::missing_safety_doc)]

use super::{
    ffi::{
        wl_argument, wl_display, wl_display_connect_to_fd, wl_display_disconnect, wl_message,
        wl_proxy_add_dispatcher, wl_proxy_get_user_data,
    },
    proxy::WlProxy,
    proxy_registry::{ProxyRegistry, WlObjectType},
    wire::MessageBuffer,
};
use crate::{
    interface::{Event, Request, WlDisplayGetRegistryRequest, WlRegistryGlobalEvent},
    object::ObjectId,
    sys::wire::Message,
};
use core::fmt;
use std::{
    collections::HashMap,
    ffi::{CStr, c_int, c_void},
    mem::{self, ManuallyDrop, MaybeUninit, offset_of},
    ops::{Deref, DerefMut},
    os::fd::{IntoRawFd, OwnedFd},
    pin::Pin,
    ptr::{self, NonNull},
    slice,
};

type ActualDispatcher = fn(Message<'_>, &mut WlAny, Pin<&mut ProxyRegistry>);

pub trait Dispatch: ObjectDowncastChecked {
    fn dispatch(&mut self, message: Message<'_>, proxies: Pin<&mut ProxyRegistry>);

    fn dispatch_raw(
        message: Message<'_>,
        object: &mut WlAny,
        mut proxies: Pin<&mut ProxyRegistry>,
    ) {
        let Some(this) = Self::downcast_mut(proxies.as_mut(), object) else {
            return;
        };

        this.data_mut().dispatch(message, proxies);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WlAny {
    id: ObjectId,
    drop: unsafe fn(*mut c_void),
}

impl Drop for WlAny {
    fn drop(&mut self) {
        unsafe { (self.drop)((&raw mut *self).cast()) }
    }
}

#[repr(C)]
pub struct WlObject<T: ObjectType> {
    head: Option<WlAny>,
    body: MaybeUninit<T>,
}

impl<T: ObjectType> WlObject<T> {
    pub const fn upcast(&self) -> Option<&WlAny> {
        self.head.as_ref()
    }

    pub const fn upcast_unchecked(&self) -> &WlAny {
        unsafe { self.head.as_ref().unwrap_unchecked() }
    }

    pub const fn upcast_mut(&mut self) -> Option<&mut WlAny> {
        self.head.as_mut()
    }

    pub const fn upcast_mut_unchecked(&mut self) -> &mut WlAny {
        unsafe { self.head.as_mut().unwrap_unchecked() }
    }

    pub const fn new(id: ObjectId, body: T) -> Self {
        let drop = |this: *mut c_void| unsafe { this.cast::<T>().drop_in_place() };

        Self {
            head: Some(WlAny { id, drop }),
            body: MaybeUninit::new(body),
        }
    }

    pub const fn uninit() -> Self {
        Self {
            head: None,
            body: MaybeUninit::uninit(),
        }
    }

    pub const fn data(&self) -> &T {
        assert!(self.head.is_some());
        unsafe { self.body.assume_init_ref() }
    }

    pub const fn data_mut(&mut self) -> &mut T {
        assert!(self.head.is_some());
        unsafe { self.body.assume_init_mut() }
    }
}

impl<T: ObjectType> Deref for WlObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl<T: ObjectType> DerefMut for WlObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

impl<T: ObjectType + fmt::Debug> fmt::Debug for WlObject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.head {
            Some(..) => f
                .debug_struct(std::any::type_name::<WlObject<T>>())
                .field("head", &self.head)
                .field("body", self.data())
                .finish(),
            None => f
                .debug_tuple(std::any::type_name::<WlObject<T>>())
                .field(&Option::<c_void>::None)
                .finish(),
        }
    }
}

impl<T: ObjectType> Drop for WlObject<T> {
    fn drop(&mut self) {
        if self.head.is_some() {
            unsafe { self.body.assume_init_drop() };
        }
    }
}

unsafe impl<T: ObjectType> ObjectDowncastChecked for T {
    fn downcast<'r, 'o: 'r>(
        proxies: Pin<&'r ProxyRegistry>,
        object: &'o WlAny,
    ) -> Option<&'o WlObject<Self>> {
        if proxies.get_type(object.id)? != T::TYPE {
            None
        } else {
            Some(unsafe {
                (&raw const *object)
                    .cast::<WlObject<Self>>()
                    .as_ref()
                    .unwrap()
            })
        }
    }

    fn downcast_mut<'r, 'o: 'r>(
        proxies: Pin<&'r mut ProxyRegistry>,
        object: &'o mut WlAny,
    ) -> Option<&'o mut WlObject<Self>> {
        if proxies.get_type(object.id)? != T::TYPE {
            None
        } else {
            Some(unsafe {
                (&raw mut *object)
                    .cast::<WlObject<Self>>()
                    .as_mut()
                    .unwrap()
            })
        }
    }
}

pub trait ObjectType {
    const TYPE: WlObjectType;
}

pub unsafe trait ObjectDowncastChecked: ObjectType + Sized {
    fn downcast<'r, 'o: 'r>(
        proxies: Pin<&'r ProxyRegistry>,
        object: &'o WlAny,
    ) -> Option<&'o WlObject<Self>>;

    fn downcast_mut<'r, 'o: 'r>(
        proxies: Pin<&'r mut ProxyRegistry>,
        object: &'o mut WlAny,
    ) -> Option<&'o mut WlObject<Self>>;
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WlRegistry {
    data: HashMap<ObjectId, String>,
}

impl ObjectType for WlRegistry {
    const TYPE: WlObjectType = WlObjectType::Registry;
}

fn actual_registry_dispatcher(
    message: Message<'_>,
    object: &mut WlAny,
    proxies: Pin<&mut ProxyRegistry>,
) {
    let registry = WlRegistry::downcast_mut(proxies, object).unwrap();
    let event = WlRegistryGlobalEvent::from_message(message).unwrap();

    registry
        .data_mut()
        .data
        .insert(event.name, event.interface.to_str().unwrap().to_owned());
}

#[repr(C)]
pub struct ProxyUserData<T: ObjectType> {
    pub dispatcher: ActualDispatcher,
    pub registry: NonNull<ProxyRegistry>,
    pub object_size: usize,
    pub object: WlObject<T>,
}

impl<T: ObjectType> ProxyUserData<T> {
    pub fn upcast_mut(&mut self) -> &mut DynProxyUserData {
        unsafe {
            mem::transmute::<(usize, &mut Self), &mut DynProxyUserData>((self.object_size, self))
        }
    }

    pub fn upcast_box(self: Box<Self>) -> Box<DynProxyUserData> {
        unsafe {
            mem::transmute::<(usize, Box<Self>), Box<DynProxyUserData>>((self.object_size, self))
        }
    }
}

#[repr(C)]
pub struct DynProxyUserData {
    pub dispatcher: ActualDispatcher,
    pub registry: NonNull<ProxyRegistry>,
    pub object_size: usize,
    // HACK(hack3rmann): splitting object into `WlAny` and `[u8]` may cause allocation
    // optimizations issues
    pub object: WlAny,
    pub data: [u8],
}

impl DynProxyUserData {
    pub unsafe fn downcast_mut<T: ObjectType>(&mut self) -> &mut ProxyUserData<T> {
        assert_eq!(
            mem::size_of::<WlAny>() + mem::size_of_val(&self.data),
            mem::size_of::<WlObject<T>>()
        );

        let (_size, ptr) =
            unsafe { mem::transmute::<*mut Self, (usize, *mut ProxyUserData<T>)>(&raw mut *self) };

        unsafe { ptr.as_mut().unwrap_unchecked() }
    }

    pub unsafe fn get_registry(&mut self) -> Pin<&mut ProxyRegistry> {
        unsafe { Pin::new_unchecked(self.registry.as_mut()) }
    }

    pub unsafe fn from_raw_mut<'s>(ptr: *mut c_void) -> &'s mut Self {
        let object_size = unsafe {
            ptr.wrapping_byte_add(offset_of!(Self, object_size))
                .cast::<usize>()
                .read()
        };

        let this = unsafe { mem::transmute::<(usize, *mut c_void), *mut Self>((object_size, ptr)) };

        unsafe { this.as_mut().unwrap_unchecked() }
    }

    pub unsafe fn from_raw_owned(ptr: *mut c_void) -> Box<Self> {
        let object_size = unsafe {
            ptr.wrapping_byte_add(offset_of!(Self, object_size))
                .cast::<usize>()
                .read()
        };

        let this = unsafe { mem::transmute::<(usize, *mut c_void), *mut Self>((object_size, ptr)) };

        unsafe { Box::from_raw(this) }
    }
}

unsafe extern "C" fn raw_registry_dispatcher(
    _implementation: *const c_void,
    proxy: *mut c_void,
    opcode: u32,
    message: *const wl_message,
    arguments: *mut wl_argument,
) -> c_int {
    let data = unsafe { wl_proxy_get_user_data(proxy.cast()) };

    if data.is_null() {
        return 0;
    }

    let proxy_data = unsafe { DynProxyUserData::from_raw_mut(data.cast()) };
    let object = &mut proxy_data.object;
    let dispatch = proxy_data.dispatcher;

    let signature = unsafe { CStr::from_ptr((*message).signature) };
    let n_arguments = signature.count_bytes();
    let arguments = unsafe { slice::from_raw_parts(arguments.cast_const(), n_arguments) };

    let message = Message {
        opcode: opcode as u16,
        arguments,
    };

    let registry = unsafe { Pin::new_unchecked(proxy_data.registry.as_mut()) };

    dbg!("dispatch");

    dispatch(message, object, registry);

    0
}

/// A handle to libwayland backend
pub struct WlDisplay {
    pub proxy: ManuallyDrop<WlProxy>,
}

impl WlDisplay {
    pub fn connect_to_fd(wayland_file_desc: OwnedFd) -> Self {
        // FIXME(hack3rmann): deal with errors
        let display =
            NonNull::new(unsafe { wl_display_connect_to_fd(wayland_file_desc.into_raw_fd()) })
                .expect("failed to connect wl_display");

        // Safety: `*mut wl_display` is compatible with `*mut wl_proxy`
        let proxy = ManuallyDrop::new(unsafe { WlProxy::from_raw(display.cast()) });

        Self { proxy }
    }

    pub fn as_raw_display_ptr(&self) -> NonNull<wl_display> {
        self.proxy.as_raw().cast()
    }

    pub fn create_registry(
        &self,
        buf: &mut impl MessageBuffer,
        mut proxies: Pin<&mut ProxyRegistry>,
    ) -> ObjectId {
        let raw_proxy =
            NonNull::new(unsafe { WlDisplayGetRegistryRequest.send_raw(&self.proxy, buf) })
                .unwrap();

        let proxy = unsafe { WlProxy::from_raw(raw_proxy) };
        let proxy_id = proxy.id();

        proxies.insert(proxy, WlObjectType::Registry);

        let object = WlObject::new(proxy_id, WlRegistry::default());

        let user_data = Box::new(ProxyUserData {
            dispatcher: actual_registry_dispatcher,
            registry: NonNull::new(&raw mut *proxies).unwrap(),
            object_size: mem::size_of_val(&object) - mem::size_of::<WlAny>(),
            object,
        });

        let add_dispatcher_result = unsafe {
            wl_proxy_add_dispatcher(
                raw_proxy.as_ptr(),
                raw_registry_dispatcher,
                ptr::null(),
                Box::into_raw(user_data).cast(),
            )
        };

        assert_ne!(add_dispatcher_result, -1);

        proxy_id
    }
}

impl Drop for WlDisplay {
    fn drop(&mut self) {
        let display = self.proxy.as_raw().cast::<wl_display>().as_ptr();
        unsafe { wl_display_disconnect(display) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        init::connect_wayland_socket,
        sys::{ffi::wl_display_dispatch, wire::SmallVecMessageBuffer},
    };
    use std::pin::pin;

    #[test]
    fn get_registry() {
        let wayland_sock = unsafe { connect_wayland_socket().unwrap() };

        let display = WlDisplay::connect_to_fd(wayland_sock);

        let mut buf = SmallVecMessageBuffer::<8>::new();
        let mut proxies = pin!(ProxyRegistry::new());

        let registry_id = display.create_registry(&mut buf, proxies.as_mut());

        unsafe { wl_display_dispatch(display.as_raw_display_ptr().as_ptr()) };

        dbg!(proxies.get_object::<WlRegistry>(registry_id).unwrap());
    }
}
