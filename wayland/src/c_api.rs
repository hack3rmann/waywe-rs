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

#[link(name = "wayland-client")]
unsafe extern "C" {
    static wl_registry_interface: wl_interface;
    static wl_compositor_interface: wl_interface;
    static wl_surface_interface: wl_interface;

    /// Connect to Wayland display on an already open fd.
    ///
    /// The [`wl_display`] takes ownership of the fd and will close
    /// it when the display is destroyed. The fd will also be closed in case of failure.
    pub fn wl_display_connect_to_fd(fd: RawFd) -> *mut wl_display;

    /// Close a connection to a Wayland display.
    ///
    /// Close the connection to display. The [`wl_proxy`] and `wl_event_queue`
    /// objects need to be manually destroyed by the caller before disconnecting.
    pub fn wl_display_disconnect(display: *mut wl_display);

    /// Block until all pending request are processed by the server.
    ///
    /// This function blocks until the server has processed all currently
    /// issued requests by sending a request to the display server
    /// and waiting for a reply before returning.
    ///
    /// This function uses `wl_display_dispatch_queue()` internally. It is not
    /// allowed to call this function while the thread is being prepared for
    /// reading events, and doing so will cause a dead lock.
    ///
    /// # Note
    ///
    /// This function may dispatch other events being received on the default queue.
    pub fn wl_display_roundtrip(display: *mut wl_display) -> c_int;

    /// Get the protocol object version of a proxy object.
    ///
    /// Gets the protocol object version of a proxy object, or `0`
    /// if the proxy was created with unversioned API.
    ///
    /// A returned value of `0` means that no version information is available,
    /// so the caller must make safe assumptions about the object's real version.
    ///
    /// [`wl_display`]'s version will always return `0`.
    pub fn wl_proxy_get_version(proxy: *mut wl_proxy) -> u32;

    /// Prepare a request to be sent to the compositor.
    ///
    /// # Params
    ///
    /// - `proxy` - The proxy object
    /// - `opcode` - Opcode of the request to be sent
    /// - `interface` - The interface to use for the new proxy
    /// - `version` - The protocol object version of the new proxy
    /// - `flags` - Flags that modify marshalling behaviour
    /// - `...` - Extra arguments for the given request
    ///
    /// # Return value
    ///
    /// A new [`wl_proxy`] for the `new_id` argument or [`ptr::null_mut`] on error
    ///
    /// Translates the request given by `opcode` and the extra arguments into the
    /// wire format and write it to the connection buffer.
    ///
    /// For new-id arguments, this function will allocate a new [`wl_proxy`] and send
    /// the ID to the server. The new [`wl_proxy`] will be returned on success or NULL
    /// on error with errno set accordingly. The newly created proxy will have
    /// the version specified.
    ///
    /// The flag `WL_MARSHAL_FLAG_DESTROY` may be passed to ensure the proxy is
    /// destroyed atomically with the marshalling in order to prevent races that
    /// can occur if the display lock is dropped between the marshal and destroy
    /// operations.
    ///
    /// # Note
    ///
    /// This should not normally be used by non-generated code.
    pub fn wl_proxy_marshal_flags(
        proxy: *mut wl_proxy,
        opcode: u32,
        interface: *const wl_interface,
        version: u32,
        flags: u32,
        ...
    ) -> *mut wl_proxy;

    /// Destroy a proxy object.
    ///
    /// # Safety
    ///
    /// `proxy` must not be a proxy wrapper.
    ///
    /// # Note
    ///
    /// This function will abort in response to egregious errors, and will do so
    /// with the display lock held. This means SIGABRT handlers must not perform
    /// any actions that would attempt to take that lock, or a deadlock would occur.
    pub fn wl_proxy_destroy(proxy: *mut wl_proxy);

    /// Set a proxy's listener.
    ///
    /// `proxy` must not be a proxy wrapper.
    ///
    /// Note: This function will abort in response to egregious errors, and will do
    /// so with the display lock held. This means SIGABRT handlers must not perform
    /// any actions that would attempt to take that lock, or a deadlock would occur.
    pub fn wl_proxy_add_listener(
        proxy: *mut wl_proxy,
        implementation: *mut extern "C" fn(),
        data: *mut c_void,
    ) -> c_int;

    /// Get the id of a proxy object.
    pub fn wl_proxy_get_id(proxy: *mut wl_proxy) -> u32;
}

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalWaylandContext {
    pub(crate) display: NonNull<wl_display>,
    pub(crate) registry: NonNull<wl_registry>,
    pub(crate) compositor: NonNull<wl_compositor>,
    pub(crate) surface: NonNull<wl_surface>,
}

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
        if -1
            == unsafe {
                wl_registry_add_listener(
                    registry.as_ptr(),
                    &raw const WL_REGISTRY_LISTENER,
                    data.get().cast(),
                )
            }
        {
            return Err(ExternalWaylandError::WlRegistryAddListenerFailed);
        }

        Ok(())
    })?;

    tracing::info!("wl_display_roundtrip()");

    // TODO(hack3rmann): handle errors
    match unsafe { wl_display_roundtrip(display.as_ptr()) } {
        -1 => return Err(ExternalWaylandError::WlDisplayRoundtripFailed),
        count => tracing::info!("wl_display_roundtrip() has handled {count} events"),
    }

    let compositor = REGISTY_DATA.with(|data| {
        let data = unsafe { data.get().as_ref().unwrap() };

        NonNull::new(data.wl_compositor)
            .ok_or(ExternalWaylandError::WlCompositorIsNull)
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

        // Safety: we have invalidate registry data to it is
        // safe to own it
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
