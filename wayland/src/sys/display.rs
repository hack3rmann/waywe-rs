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
    mem::{ManuallyDrop, MaybeUninit},
    os::fd::{IntoRawFd, OwnedFd},
    pin::Pin,
    ptr::{self, NonNull},
    slice,
};

type ActualDispatcher = fn(Message<'_>, Pin<&mut WlAny>, Pin<&mut ProxyRegistry>);

pub trait Dispatch: ObjectDowncastChecked {
    fn dispatch(self: Pin<&mut Self>, message: Message<'_>, proxies: Pin<&mut ProxyRegistry>);

    fn dispatch_raw(
        message: Message<'_>,
        object: Pin<&mut WlAny>,
        mut proxies: Pin<&mut ProxyRegistry>,
    ) {
        let Some(this) = Self::downcast_mut(proxies.as_mut(), object) else {
            return;
        };

        let this = unsafe { this.map_unchecked_mut(WlObject::data_mut) };
        this.dispatch(message, proxies);
    }
}

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlAny {
    id: ObjectId,
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
        let mut this = Self::uninit();
        this.init(id, body);
        this
    }

    pub const fn uninit() -> Self {
        Self {
            head: None,
            body: MaybeUninit::uninit(),
        }
    }

    pub const fn init(&mut self, id: ObjectId, body: T) {
        assert!(self.head.is_none());

        self.head = Some(WlAny { id });
        self.body = MaybeUninit::new(body);
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

impl<T: ObjectType + fmt::Debug> fmt::Debug for WlObject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.head {
            Some(..) => {
                f.debug_struct(std::any::type_name::<WlObject<T>>())
                    .field("head", &self.head)
                    .field("body", self.data())
                    .finish()
            }
            None => {
                f.debug_tuple(std::any::type_name::<WlObject<T>>())
                    .field(&Option::<c_void>::None)
                    .finish()
            }
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
        object: Pin<&'o WlAny>,
    ) -> Option<Pin<&'o WlObject<Self>>> {
        if proxies.get_type(object.id)? != T::TYPE {
            None
        } else {
            Some(unsafe {
                Pin::new_unchecked(
                    (&raw const *object)
                        .cast::<WlObject<Self>>()
                        .as_ref()
                        .unwrap(),
                )
            })
        }
    }

    fn downcast_mut<'r, 'o: 'r>(
        proxies: Pin<&'r mut ProxyRegistry>,
        object: Pin<&'o mut WlAny>,
    ) -> Option<Pin<&'o mut WlObject<Self>>> {
        if proxies.get_type(object.id)? != T::TYPE {
            None
        } else {
            Some(unsafe {
                Pin::new_unchecked(
                    (&raw mut *object.get_unchecked_mut())
                        .cast::<WlObject<Self>>()
                        .as_mut()
                        .unwrap(),
                )
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
        object: Pin<&'o WlAny>,
    ) -> Option<Pin<&'o WlObject<Self>>>;

    fn downcast_mut<'r, 'o: 'r>(
        proxies: Pin<&'r mut ProxyRegistry>,
        object: Pin<&'o mut WlAny>,
    ) -> Option<Pin<&'o mut WlObject<Self>>>;
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
    object: Pin<&mut WlAny>,
    proxies: Pin<&mut ProxyRegistry>,
) {
    let mut registry = WlRegistry::downcast_mut(proxies, object).unwrap();
    let event = WlRegistryGlobalEvent::from_message(message).unwrap();

    registry
        .data_mut()
        .data
        .insert(event.name, event.interface.to_str().unwrap().to_owned());
}

#[derive(Clone, Copy)]
pub struct ProxyUserData {
    pub dispatcher: Option<ActualDispatcher>,
    pub object: Option<NonNull<WlAny>>,
    pub registry: NonNull<ProxyRegistry>,
}

impl ProxyUserData {
    pub fn new(proxies: Pin<&mut ProxyRegistry>) -> Self {
        Self {
            dispatcher: None,
            object: None,
            registry: NonNull::from(unsafe { proxies.get_unchecked_mut() }),
        }
    }

    pub unsafe fn from_raw<'s>(ptr: *mut Self) -> Pin<&'s mut Self> {
        unsafe { Pin::new_unchecked(ptr.as_mut().unwrap_unchecked()) }
    }

    pub fn set_object<T: ObjectType>(&mut self, object: Pin<&mut WlObject<T>>) {
        _ = self.object.replace(NonNull::from(
            unsafe { object.get_unchecked_mut() }.upcast_mut().unwrap(),
        ));
    }

    pub fn set_dispatcher(&mut self, dispatcher: ActualDispatcher) {
        _ = self.dispatcher.replace(dispatcher);
    }

    pub unsafe fn get_object<'a>(&mut self) -> Option<Pin<&'a mut WlAny>> {
        self.object
            .map(|mut o| unsafe { Pin::new_unchecked(o.as_mut()) })
    }

    pub unsafe fn get_registry<'r>(&mut self) -> Pin<&'r mut ProxyRegistry> {
        unsafe { Pin::new_unchecked(self.registry.as_mut()) }
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

    let mut proxy_data = unsafe { ProxyUserData::from_raw(data.cast()) };
    let object = unsafe { proxy_data.get_object() }.unwrap();
    let dispatch = proxy_data.dispatcher.unwrap();

    let signature = unsafe { CStr::from_ptr((*message).signature) };
    let n_arguments = signature.count_bytes();
    let arguments = unsafe { slice::from_raw_parts(arguments.cast_const(), n_arguments) };

    let message = Message {
        opcode: opcode as u16,
        arguments,
    };

    let registry = unsafe { proxy_data.get_registry() };

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
        mut user_data: Pin<&mut ProxyUserData>,
        mut out_object: Pin<&mut WlObject<WlRegistry>>,
    ) {
        let raw_proxy = NonNull::new(unsafe { WlDisplayGetRegistryRequest.send_raw(&self.proxy, buf) })
            .unwrap();

        let proxy = unsafe { WlProxy::from_raw(raw_proxy) };
        let proxy_id = proxy.id();

        proxies.insert(proxy, WlObjectType::Registry);

        out_object.init(proxy_id, WlRegistry::default());

        user_data.set_dispatcher(actual_registry_dispatcher);
        user_data.set_object(out_object.as_mut());

        let add_dispatcher_result = unsafe {
            wl_proxy_add_dispatcher(
                raw_proxy.as_ptr(),
                raw_registry_dispatcher,
                ptr::null(),
                (&raw mut *user_data.get_unchecked_mut()).cast(),
            )
        };

        assert_ne!(add_dispatcher_result, -1);
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
        sys::{ffi::wl_display_roundtrip, wire::SmallVecMessageBuffer},
    };
    use std::pin::pin;

    #[test]
    fn get_registry() {
        let wayland_sock = unsafe { connect_wayland_socket().unwrap() };
        let display = WlDisplay::connect_to_fd(wayland_sock);
        let mut buf = SmallVecMessageBuffer::<8>::new();

        let mut proxies = pin!(ProxyRegistry::new());
        let mut user_data = pin!(ProxyUserData::new(proxies.as_mut()));
        let mut registry = pin!(WlObject::uninit());

        display.create_registry(&mut buf, proxies.as_mut(), user_data.as_mut(), registry.as_mut());

        // FIXME(hack3rmann): invalid use of the safe api
        // display should own `user_data` after `create_registry` call
        // ```rust,no_run
        // user_data.set_object(pin!(WlObject::<WlRegistry>::uninit()));
        // ```
        // compiles and, therefore, invalidates rust's safety rules

        unsafe { wl_display_roundtrip(display.as_raw_display_ptr().as_ptr()) };

        dbg!(registry.as_mut());
    }
}
