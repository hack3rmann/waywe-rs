use super::ffi::{wl_proxy, wl_proxy_destroy, wl_proxy_get_class, wl_proxy_get_id};
use crate::object::ObjectId;
use core::fmt;
use std::{
    ffi::CStr,
    mem::ManuallyDrop,
    ptr::NonNull,
    sync::atomic::{
        AtomicUsize,
        Ordering::{Acquire, Release},
    },
};

/// Represents a proxy object created on the libwayland backend
pub struct WlProxy {
    pub(crate) raw: NonNull<wl_proxy>,
    pub(crate) interface_name_length: AtomicUsize,
}

impl WlProxy {
    pub const unsafe fn from_raw(raw: NonNull<wl_proxy>) -> Self {
        Self {
            raw,
            interface_name_length: AtomicUsize::new(0),
        }
    }

    pub fn as_raw(&self) -> NonNull<wl_proxy> {
        self.raw
    }

    pub fn into_raw(self) -> NonNull<wl_proxy> {
        ManuallyDrop::new(self).raw
    }

    pub fn id(&self) -> ObjectId {
        // Safety: calling this on a valid object is safe
        let raw = unsafe { wl_proxy_get_id(self.raw.as_ptr()) };

        // Safety: any valid object in libwayland has nonzero id
        // `WlProxy`'s safety guarantees `self` is a valid object
        unsafe { ObjectId::try_from(raw).unwrap_unchecked() }
    }

    pub fn interface_name(&self) -> &str {
        // Safety: calling this on a valid object is safe
        let ptr = unsafe { wl_proxy_get_class(self.raw.as_ptr()) };

        let len = self.interface_name_length.load(Acquire);

        let string_bytes = if len == 0 {
            // Safety: interface name obtained from libwayland is a valid c-string
            let c_str = unsafe { CStr::from_ptr(ptr) };

            self.interface_name_length
                .store(c_str.count_bytes(), Release);

            c_str.to_bytes()
        } else {
            // Safety: there exactly `len` bytes in the string (excluding nul-terminator)
            unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) }
        };

        // Safety: interface name obtained from libwayland contains
        // only valid ASCII characters
        unsafe { std::str::from_utf8_unchecked(string_bytes) }
    }
}

impl Drop for WlProxy {
    fn drop(&mut self) {
        unsafe { wl_proxy_destroy(self.raw.as_ptr()) }
    }
}

impl PartialEq for WlProxy {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Eq for WlProxy {}

impl fmt::Debug for WlProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}

// TODO(hack3rmann): missing docs
#[derive(Clone, Copy, PartialEq)]
pub struct WlProxyQuery {
    // TODO(hack3rmann): determine a nice API for object querying
    raw: *const wl_proxy,
}

impl WlProxyQuery {
    pub const unsafe fn from_raw(raw: *const wl_proxy) -> Self {
        Self { raw }
    }

    pub const unsafe fn into_raw(self) -> *const wl_proxy {
        self.raw
    }
}

impl fmt::Debug for WlProxyQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}
impl<T> PartialEq for WlProxyQuery<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

#[derive(Clone, PartialEq, Copy)]
pub struct WlDynProxyQuery {
    // TODO(hack3rmann): determine a nice API for object querying
    raw: *const wl_proxy,
}

impl WlDynProxyQuery {
    pub const unsafe fn from_raw(raw: *const wl_proxy) -> Self {
        Self { raw }
    }
}

impl fmt::Debug for WlDynProxyQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}

macro_rules! define_proxies {
    ( $( $Proxy:ident ),* $(,)? ) => {
        $(
            pub struct $Proxy {
                pub(crate) proxy: WlProxy,
            }

            impl AsProxy for $Proxy {
                fn as_proxy(&self) -> WlProxyBorrow<'_> {
                    self.proxy.as_proxy()
                }
            }

            impl From<WlProxy> for $Proxy {
                fn from(proxy: WlProxy) -> Self {
                    Self { proxy }
                }
            }

            impl fmt::Debug for $Proxy {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(std::any::type_name::<Self>())
                        .finish_non_exhaustive()
                }
            }
        )*
    };
}

define_proxies! {
    WlRegistry, WlCompositor, WlRegion, WlSurface, WlOutput,
    WlShm, WlShmPool, ZwlrLayerShellV1, ZwlrLayerSurfaceV1, WlBuffer
}
