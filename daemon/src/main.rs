// FIXME(hack3rmann):
#![allow(unused)]

pub mod wayland;

use c_wayland::WlGlobalData;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::ptr;
use std::{error::Error, os::unix::net::UnixStream};
use wayland::connect_wayland_socket;
use wayland::interface::{
    self, AnyEvent, Event, WlCallbackDoneEvent, WlDisplayDeleteIdEvent,
    WlDisplayGetRegistryRequest, WlDisplaySyncRequest, WlRegistryGlobalEvent,
};
use wayland::object::ObjectId;
use wayland::wire::{self, Message, MessageBuffer, MessageBuildError};

#[derive(Clone, Default, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
struct InterfaceDesc {
    object_name: ObjectId,
    version: u32,
}

fn sync(
    on: ObjectId,
    sock: &mut UnixStream,
    buf: &mut MessageBuffer,
) -> Result<(), MessageBuildError> {
    use interface::display::request::*;

    sync(Sync { callback: on }, buf)?.send(sock)?;

    let _done = interface::recv_event::<WlCallbackDoneEvent>(sock, buf)?;
    let remove_id = interface::recv_event::<WlDisplayDeleteIdEvent>(sock, buf)?;
    assert_eq!(remove_id.id, on);

    Ok(())
}

fn get_registry(
    sock: &mut UnixStream,
    buf: &mut MessageBuffer,
) -> Result<HashMap<String, InterfaceDesc>, MessageBuildError> {
    use interface::display::request::*;

    get_registry(
        GetRegistry {
            registry: ObjectId::WL_REGISTRY,
        },
        buf,
    )?
    .send(sock)?;

    let mut registry = HashMap::<String, InterfaceDesc>::new();

    sync(
        Sync {
            callback: ObjectId::WL_CALLBACK,
        },
        buf,
    )?
    .send(sock)?;

    loop {
        wire::read_message_into(sock, buf)?;
        let message = Message::from_u32_slice(buf.as_slice());

        let Some(global) = WlRegistryGlobalEvent::from_message(message) else {
            let Some(_done) = WlCallbackDoneEvent::from_message(message) else {
                panic!("wrong message");
            };

            break;
        };

        registry.insert(
            global.interface.to_owned(),
            InterfaceDesc {
                object_name: global.name,
                version: global.version,
            },
        );
    }

    let remove_id = WlDisplayDeleteIdEvent::recv(sock, buf)?;
    assert_eq!(remove_id.id, ObjectId::WL_CALLBACK);

    Ok(registry)
}

#[allow(non_camel_case_types, unused)]
mod c_wayland {
    use std::{
        cell::UnsafeCell,
        ffi::{c_char, c_int, c_void, CStr},
        mem::{offset_of, MaybeUninit},
        ptr::{self, NonNull},
    };

    pub type wl_display = c_void;
    pub type wl_registry = c_void;
    pub type wl_surface = c_void;
    pub type wl_message = c_void;
    pub type wl_compositor = c_void;
    pub type wl_proxy = c_void;

    #[repr(C)]
    pub struct wl_interface {
        pub name: *const c_char,
        pub version: c_int,
        pub method_count: c_int,
        pub methods: *const wl_message,
        pub eval_count: c_int,
        pub events: *const wl_message,
    }

    #[repr(C)]
    pub struct wl_registry_listener {
        pub global: unsafe extern "C" fn(*mut c_void, *mut wl_registry, u32, *const c_char, u32),
        pub global_remove: unsafe extern "C" fn(*mut c_void, *mut wl_registry, u32),
    }

    #[repr(C)]
    #[derive(Clone, Debug, PartialEq)]
    pub struct WlGlobalData {
        pub wl_compositor: *mut wl_compositor,
    }

    pub const WL_REGISTRY_BIND: u32 = 0;
    pub const WL_DISPLAY_GET_REGISTRY: u32 = 1;
    pub const WL_COMPOSITOR_CREATE_SURFACE: u32 = 0;

    pub unsafe extern "C" fn wl_display_get_registry(display: *mut wl_display) -> *mut wl_registry {
        let version = unsafe { wl_proxy_get_version(display.cast()) };

        unsafe {
            wl_proxy_marshal_flags(
                display.cast(),
                WL_DISPLAY_GET_REGISTRY,
                &wl_registry_interface,
                version,
                0,
                ptr::null_mut::<c_void>(),
            )
        }
        .cast()
    }

    pub unsafe extern "C" fn wl_registry_destroy(registry: *mut wl_registry) {
        wl_proxy_destroy(registry.cast());
    }

    pub unsafe extern "C" fn wl_registry_add_listener(
        registry: *mut wl_registry,
        listener: *const wl_registry_listener,
        data: *mut c_void,
    ) -> c_int {
        wl_proxy_add_listener(registry.cast(), listener.cast_mut().cast(), data)
    }

    pub unsafe extern "C" fn registry_handle_global(
        data: *mut c_void,
        registry: *mut wl_registry,
        name: u32,
        interface: *const c_char,
        version: u32,
    ) {
        if strcmp(interface, wl_compositor_interface.name) == 0 {
            let global_data = NonNull::new(data.cast::<WlGlobalData>())
                .expect("invalid data argument in registry global event handler");

            let wl_compositor =
                unsafe { wl_registry_bind(registry, name, &wl_compositor_interface, version) };

            global_data
                .add(offset_of!(WlGlobalData, wl_compositor))
                .write(WlGlobalData { wl_compositor });
        }
    }

    pub unsafe extern "C" fn registry_handle_global_remove(
        data: *mut c_void,
        registry: *mut wl_registry,
        name: u32,
    ) {
    }

    pub unsafe extern "C" fn wl_registry_bind(
        registry: *mut wl_registry,
        name: u32,
        interface: *const wl_interface,
        version: u32,
    ) -> *mut c_void {
        unsafe {
            wl_proxy_marshal_flags(
                registry.cast(),
                WL_REGISTRY_BIND,
                interface,
                version,
                0,
                interface.add(offset_of!(wl_interface, name)).read(),
                version,
                ptr::null_mut::<c_void>(),
            )
        }
        .cast()
    }

    pub unsafe extern "C" fn wl_compositor_create_surface(
        wl_compositor: *mut wl_compositor,
    ) -> *mut wl_compositor {
        let version = unsafe { wl_proxy_get_version(wl_compositor.cast()) };

        unsafe {
            wl_proxy_marshal_flags(
                wl_compositor.cast(),
                WL_COMPOSITOR_CREATE_SURFACE,
                &wl_surface_interface,
                version,
                0,
                ptr::null_mut::<c_void>(),
            )
        }
        .cast()
    }

    pub const WL_REGISTRY_LISTENER: wl_registry_listener = wl_registry_listener {
        global: registry_handle_global,
        global_remove: registry_handle_global_remove,
    };

    #[link(name = "c")]
    unsafe extern "C" {
        pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    }

    #[link(name = "wayland-client")]
    unsafe extern "C" {
        static wl_display_interface: wl_interface;
        static wl_registry_interface: wl_interface;
        static wl_compositor_interface: wl_interface;
        static wl_surface_interface: wl_interface;

        pub fn wl_display_connect_to_fd(fd: c_int) -> *mut wl_display;
        pub fn wl_display_disconnect(display: *mut wl_display);
        pub fn wl_display_roundtrip(display: *mut wl_display) -> c_int;

        pub fn wl_proxy_get_version(proxy: *mut wl_proxy) -> u32;
        pub fn wl_proxy_marshal_flags(
            proxy: *mut wl_proxy,
            opcode: u32,
            interface: *const wl_interface,
            version: u32,
            flags: u32,
            ...
        ) -> *mut wl_proxy;
        pub fn wl_proxy_destroy(proxy: *mut wl_proxy);
        pub fn wl_proxy_add_listener(
            proxy: *mut wl_proxy,
            implementation: *mut extern "C" fn(),
            data: *mut c_void,
        ) -> c_int;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut sock = UnixStream::from(unsafe { connect_wayland_socket()? });
    let mut buf = MessageBuffer::new();

    eprintln!("creating wl_display");
    let wl_display = unsafe { c_wayland::wl_display_connect_to_fd(sock.as_raw_fd()) };
    eprintln!("success");

    let registry = unsafe { c_wayland::wl_display_get_registry(wl_display) };
    let global_data = UnsafeCell::new(WlGlobalData {
        wl_compositor: ptr::null_mut(),
    });

    unsafe {
        c_wayland::wl_registry_add_listener(
            registry,
            &c_wayland::WL_REGISTRY_LISTENER,
            global_data.get().cast(),
        );
    }

    unsafe { c_wayland::wl_display_roundtrip(wl_display) };

    let wl_compositor = unsafe { global_data.get().read() }.wl_compositor;

    let wl_surface = unsafe { c_wayland::wl_compositor_create_surface(wl_compositor) };

    assert!(!wl_surface.is_null());

    unsafe { c_wayland::wl_registry_destroy(registry) };

    drop(sock);

    eprintln!("destroying wl_display");
    unsafe { c_wayland::wl_display_disconnect(wl_display) };
    eprintln!("success");

    Ok(())
}
