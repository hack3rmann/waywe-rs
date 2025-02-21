#![allow(non_camel_case_types, clippy::missing_safety_doc)]

use std::{
    cell::UnsafeCell,
    ffi::{c_char, c_int, c_void},
    mem::offset_of,
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
    unsafe { wl_proxy_destroy(registry.cast()) };
}

pub unsafe extern "C" fn wl_registry_add_listener(
    registry: *mut wl_registry,
    listener: *const wl_registry_listener,
    data: *mut c_void,
) -> c_int {
    unsafe { wl_proxy_add_listener(registry.cast(), listener.cast_mut().cast(), data) }
}

pub unsafe extern "C" fn registry_handle_global(
    data: *mut c_void,
    registry: *mut wl_registry,
    name: u32,
    interface: *const c_char,
    version: u32,
) {
    if unsafe { strcmp(interface, wl_compositor_interface.name) } == 0 {
        let global_data = NonNull::new(data.cast::<WlGlobalData>())
            .expect("invalid data argument in registry global event handler");

        let wl_compositor =
            unsafe { wl_registry_bind(registry, name, &raw const wl_compositor_interface, version) };

        unsafe {
            global_data
                .add(offset_of!(WlGlobalData, wl_compositor))
                .write(WlGlobalData { wl_compositor });
        }
    }
}

pub unsafe extern "C" fn registry_handle_global_remove(
    _data: *mut c_void,
    _registry: *mut wl_registry,
    _name: u32,
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
            name,
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
            &raw const wl_surface_interface,
            version,
            0,
            ptr::null_mut::<c_void>(),
        )
    }
    .cast()
}

pub static WL_REGISTRY_LISTENER: wl_registry_listener = wl_registry_listener {
    global: registry_handle_global,
    global_remove: registry_handle_global_remove,
};

#[link(name = "c")]
unsafe extern "C" {
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
}

#[link(name = "wayland-client")]
unsafe extern "C" {
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

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct ExternWaylandContext {
    pub(crate) display: *mut wl_display,
    pub(crate) registry: *mut wl_registry,
    pub(crate) compositor: *mut wl_compositor,
    pub(crate) surface: *mut wl_surface,
}

impl ExternWaylandContext {
    // TODO: check all extern object are non-null
    pub unsafe fn from_raw_fd(wayland_socket_fd: c_int) -> Self {
        let display = unsafe { wl_display_connect_to_fd(wayland_socket_fd) };
        let registry = unsafe { wl_display_get_registry(display) };

        let global_data = UnsafeCell::new(WlGlobalData {
            wl_compositor: ptr::null_mut(),
        });

        unsafe {
            wl_registry_add_listener(registry, &raw const WL_REGISTRY_LISTENER, global_data.get().cast());
        }

        // TODO: replace with our implementation
        assert_ne!(-1, unsafe { wl_display_roundtrip(display) });

        let compositor = unsafe { global_data.get().read() }.wl_compositor;
        let surface = unsafe { wl_compositor_create_surface(compositor) };

        Self {
            display,
            registry,
            compositor,
            surface,
        }
    }

    pub unsafe fn close_connection(self) -> Result<(), rustix::io::Errno> {
        unsafe { wl_registry_destroy(self.registry) };
        unsafe { wl_display_disconnect(self.display) };
        Ok(())
    }
}

impl Default for ExternWaylandContext {
    fn default() -> Self {
        Self {
            display: ptr::null_mut(),
            registry: ptr::null_mut(),
            compositor: ptr::null_mut(),
            surface: ptr::null_mut(),
        }
    }
}
