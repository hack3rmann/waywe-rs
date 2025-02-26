#![allow(non_camel_case_types, clippy::missing_safety_doc)]

use crate::object::{ObjectId, ObjectIdMap};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use rustix::path::Arg as _;
use std::{
    cell::UnsafeCell,
    collections::HashMap,
    ffi::{CStr, c_char, c_int, c_void},
    mem::{MaybeUninit, offset_of},
    os::fd::RawFd,
    process,
    ptr::{self, NonNull},
};
use thiserror::Error;

pub use super::ffi::{
    WL_COMPOSITOR_CREATE_SURFACE, WL_DISPLAY_GET_REGISTRY, WL_REGISTRY_BIND, wl_compositor,
    wl_compositor_interface, wl_display, wl_display_connect_to_fd, wl_display_disconnect,
    wl_display_roundtrip, wl_interface, wl_message, wl_proxy, wl_proxy_add_listener,
    wl_proxy_destroy, wl_proxy_get_id, wl_proxy_get_version, wl_proxy_marshal_flags, wl_registry,
    wl_registry_interface, wl_surface, wl_surface_interface,
};

#[repr(C)]
pub struct wl_registry_listener {
    pub global: unsafe extern "C" fn(*mut c_void, *mut wl_registry, u32, *const c_char, u32),
    pub global_remove: unsafe extern "C" fn(*mut c_void, *mut wl_registry, u32),
}

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryDataItem {
    pub name: ObjectId,
    pub version: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct WlRegistryData {
    pub is_valid: bool,
    pub wl_compositor: *mut wl_compositor,
    pub globals: MaybeUninit<HashMap<String, WlRegistryDataItem>>,
}

impl Default for WlRegistryData {
    fn default() -> Self {
        Self {
            is_valid: true,
            wl_compositor: ptr::null_mut(),
            globals: MaybeUninit::new(HashMap::new()),
        }
    }
}

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
    let mut global_data = NonNull::new(data.cast::<WlRegistryData>()).unwrap_or_else(|| {
        tracing::error!("invalid null data pointer in registry callback");
        process::abort();
    });

    // Safety: as long as wayland-client calls this handler in-sync
    // all accesses to `WlGlobalData` are mutually excluded.
    let global_data = unsafe { global_data.as_mut() };

    // Safety: used by an argument below
    if !global_data.is_valid {
        return;
    }

    if interface.is_null() {
        tracing::error!("invalid null interface c-string");
        process::abort();
    }

    // Safety:
    // - wayland-client ensures the string is valid c-string
    let interface = unsafe { CStr::from_ptr(interface) };

    let interface = interface
        .as_str()
        .unwrap_or_else(|_| {
            tracing::error!("invalid non-UTF8 interface string");
            process::abort();
        })
        .to_owned();

    if interface == "wl_compositor" {
        global_data.wl_compositor = unsafe {
            wl_registry_bind(registry, name, &raw const wl_compositor_interface, version)
        };
    }

    let header = WlRegistryDataItem {
        name: ObjectId::try_from(name).unwrap_or_else(|_| {
            tracing::error!("invalid wayland global object name = 0 on '{interface}' interface");
            process::abort();
        }),
        version,
    };

    // Safety: `globals` initialized because global_data is valid due to the check above
    unsafe { global_data.globals.assume_init_mut() }.insert(interface, header);
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

thread_local! {
    static REGISTY_DATA: UnsafeCell<WlRegistryData> = UnsafeCell::new(WlRegistryData::default());
}

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalWaylandContext {
    pub(crate) display: NonNull<wl_display>,
    pub(crate) registry: NonNull<wl_registry>,
    pub(crate) compositor: NonNull<wl_compositor>,
    pub(crate) surface: NonNull<wl_surface>,
}

// FIXME(hack3rmann): destroy external wayland objects before panicking
pub(crate) unsafe fn initialize_wayland(
    wayland_socket_fd: RawFd,
) -> Result<(ExternalWaylandContext, ExternalObjectInformation), ExternalWaylandError> {
    let display = NonNull::new(unsafe { wl_display_connect_to_fd(wayland_socket_fd) })
        .ok_or(ExternalWaylandError::WlDisplayIsNull)?;

    tracing::info!("wl_display_get_registry()");

    let registry = NonNull::new(unsafe { wl_display_get_registry(display.as_ptr()) })
        .ok_or(ExternalWaylandError::WlRegistryIsNull)?;

    tracing::info!("wl_registry_add_listener()");

    REGISTY_DATA.with(|data| {
        let result = unsafe {
            wl_registry_add_listener(
                registry.as_ptr(),
                &raw const WL_REGISTRY_LISTENER,
                data.get().cast(),
            )
        };

        if result == -1 {
            Err(ExternalWaylandError::WlRegistryAddListenerFailed)
        } else {
            Ok(())
        }
    })?;

    tracing::info!("wl_display_roundtrip()");

    match unsafe { wl_display_roundtrip(display.as_ptr()) } {
        -1 => return Err(ExternalWaylandError::WlDisplayRoundtripFailed),
        count => tracing::info!("wl_display_roundtrip() has handled {count} events"),
    }

    let compositor = REGISTY_DATA.with(|data| {
        let data = unsafe { data.get().as_ref().unwrap() };

        NonNull::new(data.wl_compositor).ok_or(ExternalWaylandError::WlCompositorIsNull)
    })?;

    tracing::info!("wl_compositor_create_surface()");

    let surface = NonNull::new(unsafe { wl_compositor_create_surface(compositor.as_ptr()) })
        .ok_or(ExternalWaylandError::WlSurfaceIsNull)?;

    let mut mapped_names = ObjectIdMap::default();

    mapped_names.map(
        ObjectId::WL_DISPLAY,
        ObjectId::new(unsafe { wl_proxy_get_id(display.as_ptr()) }),
    );

    mapped_names.map(
        ObjectId::WL_REGISTRY,
        ObjectId::new(unsafe { wl_proxy_get_id(registry.as_ptr()) }),
    );

    mapped_names.map(
        ObjectId::WL_COMPOSITOR,
        ObjectId::new(unsafe { wl_proxy_get_id(compositor.as_ptr()) }),
    );

    mapped_names.map(
        ObjectId::WL_SURFACE,
        ObjectId::new(unsafe { wl_proxy_get_id(surface.as_ptr()) }),
    );

    let globals = REGISTY_DATA.with(|data| {
        let data = unsafe { data.get().as_mut().unwrap() };

        // Safety: we have made registry data invalid so it is safe to own it
        data.is_valid = false;
        unsafe { data.globals.assume_init_read() }
    });

    Ok((
        ExternalWaylandContext {
            display,
            registry,
            compositor,
            surface,
        },
        ExternalObjectInformation {
            globals,
            mapped_names,
        },
    ))
}

impl ExternalWaylandContext {
    pub unsafe fn raw_display_handle(self) -> RawDisplayHandle {
        RawDisplayHandle::Wayland(WaylandDisplayHandle::new(self.display))
    }

    pub unsafe fn raw_window_handle(self) -> RawWindowHandle {
        RawWindowHandle::Wayland(WaylandWindowHandle::new(self.surface))
    }

    pub unsafe fn close_connection(self) {
        unsafe { wl_proxy_destroy(self.surface.as_ptr()) };
        unsafe { wl_proxy_destroy(self.compositor.as_ptr()) };
        unsafe { wl_proxy_destroy(self.registry.as_ptr()) };
        unsafe { wl_display_disconnect(self.display.as_ptr()) };
    }
}

impl Default for ExternalWaylandContext {
    fn default() -> Self {
        Self {
            display: NonNull::dangling(),
            registry: NonNull::dangling(),
            compositor: NonNull::dangling(),
            surface: NonNull::dangling(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ExternalWaylandError {
    #[error("external wayland error: wl_display is null")]
    WlDisplayIsNull,
    #[error("external wayland error: wl_registry is null")]
    WlRegistryIsNull,
    #[error("external wayland error: wl_compositor is null")]
    WlCompositorIsNull,
    #[error("external wayland error: wl_surface is null")]
    WlSurfaceIsNull,
    #[error("external wayland error: wl_display_roundtrip failed")]
    WlDisplayRoundtripFailed,
    #[error("external wayland error: wl_registry_add_listener failed")]
    WlRegistryAddListenerFailed,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ExternalObjectInformation {
    pub globals: HashMap<String, WlRegistryDataItem>,
    pub mapped_names: ObjectIdMap,
}
