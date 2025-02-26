#![allow(non_camel_case_types)]

use std::{
    ffi::{c_char, c_int, c_void},
    mem::offset_of,
    os::fd::RawFd,
    ptr,
};

pub type wl_display = c_void;
pub type wl_registry = c_void;
pub type wl_surface = c_void;
pub type wl_compositor = c_void;
pub type wl_proxy = c_void;

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct wl_fixed_t(pub(crate) c_int);

impl wl_fixed_t {
    pub const fn into_double(self) -> f64 {
        // see <https://chromium.googlesource.com/external/wayland/wayland/+/refs/heads/master/src/wayland-util.h#235>
        let x = ((1023_i64 + 44_i64) << 52) + (1_i64 << 51) + self.0 as i64;
        f64::from_bits(x as u64) - (3_i64 << 43) as f64
    }

    pub const fn from_double(value: f64) -> Self {
        // see <https://chromium.googlesource.com/external/wayland/wayland/+/refs/heads/master/src/wayland-util.h#248>
        let shifted = value + (3_i64 << (51 - 8)) as f64;
        Self(shifted.to_bits() as i64 as c_int)
    }

    pub const fn into_int(self) -> i32 {
        self.0 / 256
    }

    pub const fn from_int(value: i32) -> Self {
        Self(value * 256)
    }

    pub const fn from_raw(value: c_int) -> Self {
        Self(value)
    }

    pub const fn into_raw(self) -> c_int {
        self.0
    }
}

impl From<f64> for wl_fixed_t {
    fn from(value: f64) -> Self {
        Self::from_double(value)
    }
}

impl From<wl_fixed_t> for f64 {
    fn from(value: wl_fixed_t) -> Self {
        value.into_double()
    }
}

impl From<i32> for wl_fixed_t {
    fn from(value: i32) -> Self {
        Self::from_int(value)
    }
}

impl From<wl_fixed_t> for i32 {
    fn from(value: wl_fixed_t) -> Self {
        value.into_int()
    }
}

#[repr(C)]
pub struct wl_object {
    pub interface: *const wl_interface,
    pub implementation: *const c_void,
    pub id: u32,
}

#[repr(C)]
pub struct wl_message {
    pub name: *const c_char,
    pub signature: *const c_char,
    pub types: *mut *const wl_interface,
}

#[repr(C)]
pub struct wl_array {
    pub size: usize,
    // NOTE(ArnoDarkrose): same as 'capacity'
    pub alloc: usize,
    pub data: *mut c_void,
}

impl wl_array {
    pub const fn new() -> Self {
        Self {
            size: 0,
            alloc: 0,
            data: ptr::null_mut(),
        }
    }

    /// # Safety
    ///
    /// `this` must point to a valid unallocated [`wl_array`]
    pub const unsafe fn init(this: *mut Self) {
        unsafe { this.write(Self::new()) }
    }

    /// # Safety
    ///
    /// `this` must be allocated by calls to malloc or related functions.
    /// It has to be valid or equal to null
    pub unsafe fn release(this: *mut Self) {
        // Safety
        // See safety for the function
        unsafe {
            free((*this).data);
        }
    }

    /// # Safety
    /// - `this` must point to a valid, allocated object
    /// - if `this.data` is not null it must be allocated by malloc or a similar function
    pub unsafe fn add(this: *mut Self, size: usize) -> *mut c_void {
        // Safety
        // `this` is valid (see the function safety)
        let upper_bound = unsafe { (*this).size + size };
        let array_alloc = unsafe { (*this).alloc };
        let new_data;

        let mut alloc = if array_alloc > 0 { array_alloc } else { 16 };

        while alloc < upper_bound {
            alloc *= 2;
        }

        if array_alloc < alloc {
            new_data = if array_alloc > 0 {
                unsafe { realloc((*this).data, alloc) }
            } else {
                unsafe { malloc(alloc) }
            };

            if new_data.is_null() {
                return ptr::null_mut();
            }

            // Safety
            // `this` is valid (see the function safety) and new_data
            // is valid, as it was successfully allocated above
            unsafe { (*this).data = new_data };
            unsafe { (*this).alloc = alloc };
        }

        // Safety
        // this.data points to an allocated object (see above)
        let res = unsafe { (*this).data.byte_add((*this).size) };
        unsafe { (*this).size += size };

        res
    }

    /// # Safety
    /// - `this` and `source` must point to valid objects
    /// - this.data and source.data must point to allocated, aligned objects
    ///     and its memory areas must not overlap
    /// - source.data must be valid for read for source.size bytes
    pub unsafe fn copy(this: *mut Self, source: *mut Self) -> bool {
        let array_size = unsafe { (*this).size };
        let source_size = unsafe { (*this).size };

        if array_size < source_size {
            let add_res = unsafe { wl_array::add(this, source_size - array_size) };

            if add_res.is_null() {
                return false;
            }
        } else {
            unsafe { (*this).size = source_size };
        }

        // Safety
        // this.data and source.data are valid, properly aligned and don't overlap (see the function safety)
        // Code above ensures that this.data is valid for write for source.size bytes
        unsafe {
            ptr::copy_nonoverlapping((*this).data, (*source).data, source_size);
        }

        true
    }
}

impl Default for wl_array {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
pub struct wl_list {
    pub prev: *mut Self,
    pub next: *mut Self,
    pub data: [u8; 0],
}

impl wl_list {
    /// # Safety
    ///
    /// `this` must point to a valid value of [`wl_list`]
    pub const unsafe fn init(this: *mut Self) {
        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(this)
        };
        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(this)
        };
    }

    /// # Safety
    ///
    /// both `this` and `element` must point to valid values of [`wl_list`]
    pub const unsafe fn insert(this: *mut Self, element: *mut Self) {
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(this)
        };
        let this_next = unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(this_next)
        };
        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(element)
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(element)
        };
    }

    /// # Safety
    ///
    /// - `element` should point to a valid value of [`wl_list`]
    /// - `element` should have valid previous and next elements
    pub const unsafe fn remove(element: *mut Self) {
        let element_next = unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(element_next)
        };
        let element_prev = unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .read()
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(element_prev)
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(ptr::null_mut())
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(ptr::null_mut())
        };
    }

    /// # Safety
    ///
    /// - `this` must point to a valid value of [`wl_list`]
    /// - `this` must have valid `next` and `prev` values
    pub unsafe fn length(this: *const Self) -> usize {
        let mut elem = unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
        };
        let mut count = 0_usize;

        while elem.cast_const() != this {
            elem = unsafe {
                elem.wrapping_byte_add(offset_of!(wl_list, next))
                    .cast::<*mut Self>()
                    .read()
            };
            count += 1;
        }

        count
    }

    /// # Safety
    ///
    /// - `this` must point to a valid value of [`wl_list`]
    /// - `this` must have a valid `next` value
    pub unsafe fn empty(this: *const Self) -> bool {
        this == unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
        }
    }

    /// # Safety
    /// - `this` must point to a valid value of [`wl_list`]
    /// - `other` must point to a valid value of [`wl_list`]
    /// - `this` must have a valid `next` value
    /// - `other` must have a valid `next` value
    /// - `other` must have a valid `prev` value
    pub unsafe fn insert_list(this: *mut Self, other: *mut Self) {
        if unsafe { Self::empty(other) } {
            return;
        }

        unsafe {
            other
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(this);
        }

        unsafe {
            other
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(
                    this.wrapping_byte_add(offset_of!(wl_list, next))
                        .cast::<*mut Self>()
                        .read(),
                );
        }

        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(
                    other
                        .wrapping_byte_add(offset_of!(wl_list, prev))
                        .cast::<*mut Self>()
                        .read(),
                );
        }

        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(
                    other
                        .wrapping_byte_add(offset_of!(wl_list, next))
                        .cast::<*mut Self>()
                        .read(),
                );
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union wl_argument {
    pub i: i32,
    pub u: u32,
    pub f: wl_fixed_t,
    pub s: *const c_char,
    pub o: *const wl_object,
    pub n: u32,
    pub a: *const wl_array,
    pub h: RawFd,
}

pub type wl_dispatcher_func_t = unsafe extern "C" fn(
    *const c_void,
    *mut c_void,
    u32,
    *const wl_message,
    *mut wl_argument,
) -> c_int;

#[repr(C)]
pub struct wl_interface {
    pub name: *const c_char,
    pub version: c_int,
    pub method_count: c_int,
    pub methods: *const wl_message,
    pub event_count: c_int,
    pub events: *const wl_message,
}

#[repr(C)]
pub struct wl_registry_listener {
    pub global: unsafe extern "C" fn(*mut c_void, *mut wl_registry, u32, *const c_char, u32),
    pub global_remove: unsafe extern "C" fn(*mut c_void, *mut wl_registry, u32),
}

pub const WL_REGISTRY_BIND: u32 = 0;
pub const WL_DISPLAY_GET_REGISTRY: u32 = 1;
pub const WL_COMPOSITOR_CREATE_SURFACE: u32 = 0;

#[link(name = "wayland-client")]
#[allow(dead_code)]
unsafe extern "C" {
    pub static wl_registry_interface: wl_interface;
    pub static wl_compositor_interface: wl_interface;
    pub static wl_surface_interface: wl_interface;

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

    /// Prepare a request to be sent to the compositor
    ///
    /// This function is similar to `wl_proxy_marshal_array_constructor()`, except
    /// it doesn't create proxies for new-id arguments.
    ///
    /// # Parameters
    ///
    /// - `proxy` - The proxy object
    /// - `opcode` - Opcode of the request to be sent
    /// - `args` - Extra arguments for the given request
    ///
    /// # Note
    ///
    /// This is intended to be used by language bindings and not in non-generated code.
    ///
    /// # See also
    ///
    /// wl_proxy_marshal()
    pub fn wl_proxy_marshal_array(proxy: *mut wl_proxy, opcode: u32, args: *mut wl_argument);

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

    /// Set a proxy's listener (with dispatcher)
    ///
    /// # Parameters
    ///
    /// - `proxy` - The proxy object
    /// - `dispatcher` - The dispatcher to be used for this proxy
    /// - `implementation` - The dispatcher-specific listener implementation
    /// - `data` - User data to be associated with the proxy
    ///
    /// # Returns
    ///
    /// `0` on success or `-1` on failure
    ///
    /// Set proxy's listener to use `dispatcher` as its dispatcher and
    /// `data` as its dispatcher-specific implementation and its user
    /// data to data. If a listener has already been set, this function fails
    /// and nothing is changed.
    ///
    /// The exact details of dispatcher_data depend on the dispatcher used. This
    /// function is intended to be used by language bindings, not user code.
    ///
    /// # Safety
    ///
    /// `proxy` must not be a proxy wrapper.
    pub fn wl_proxy_add_dispatcher(
        proxy: *mut wl_proxy,
        dispatcher: wl_dispatcher_func_t,
        implementation: *const c_void,
        data: *mut c_void,
    ) -> c_int;

    /// Get the id of a proxy object.
    pub fn wl_proxy_get_id(proxy: *mut wl_proxy) -> u32;
}

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void) -> c_void;
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
}
