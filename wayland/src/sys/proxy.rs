use super::ffi::{wl_proxy, wl_proxy_destroy};
use core::fmt;
use std::{marker::PhantomData, ptr::NonNull};

pub struct WlProxy {
    pub(crate) raw: NonNull<wl_proxy>,
}

impl Drop for WlProxy {
    fn drop(&mut self) {
        unsafe { wl_proxy_destroy(self.raw.as_ptr()) }
    }
}

impl AsProxy for WlProxy {
    fn as_proxy(&self) -> WlProxyBorrow<'_> {
        unsafe { WlProxyBorrow::from_raw(self.raw) }
    }
}

#[derive(Clone, Copy)]
pub struct WlProxyBorrow<'s> {
    pub raw: NonNull<wl_proxy>,
    pub _p: PhantomData<&'s WlProxy>,
}

impl WlProxyBorrow<'_> {
    pub const unsafe fn from_raw(value: NonNull<wl_proxy>) -> Self {
        Self {
            raw: value,
            _p: PhantomData,
        }
    }

    pub const fn as_raw(self) -> NonNull<wl_proxy> {
        self.raw
    }
}

impl AsProxy for WlProxyBorrow<'_> {
    fn as_proxy(&self) -> WlProxyBorrow<'_> {
        *self
    }
}

pub trait AsProxy {
    fn as_proxy(&self) -> WlProxyBorrow<'_>;
}

pub struct WlProxyQuery<T> {
    // TODO(hack3rmann): determine a nice API for object querying
    raw: *const wl_proxy,
    // TODO(hack3rmann): determine variance
    _p: PhantomData<fn() -> T>,
}

impl<T: AsProxy> WlProxyQuery<T> {
    pub const unsafe fn from_raw(raw: *const wl_proxy) -> Self {
        Self {
            raw,
            _p: PhantomData,
        }
    }
}

impl<T> Clone for WlProxyQuery<T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            _p: PhantomData,
        }
    }
}

impl<T> Copy for WlProxyQuery<T> {}

impl<T> fmt::Debug for WlProxyQuery<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WlProxyQuery<{}> {{ ... }}", std::any::type_name::<T>())
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
        f.write_str("WlDynProxyQuery {{ ... }}")
    }
}

pub struct WlRegistry {
    pub(crate) proxy: WlProxy,
}

impl AsProxy for WlRegistry {
    fn as_proxy(&self) -> WlProxyBorrow<'_> {
        self.proxy.as_proxy()
    }
}

pub struct WlCompositor {
    pub(crate) proxy: WlProxy,
}

impl AsProxy for WlCompositor {
    fn as_proxy(&self) -> WlProxyBorrow<'_> {
        self.proxy.as_proxy()
    }
}

pub struct WlRegion {
    pub(crate) proxy: WlProxy,
}

impl AsProxy for WlRegion {
    fn as_proxy(&self) -> WlProxyBorrow<'_> {
        self.proxy.as_proxy()
    }
}

pub struct WlBuffer {
    pub(crate) proxy: WlProxy,
}

impl AsProxy for WlBuffer {
    fn as_proxy(&self) -> WlProxyBorrow<'_> {
        self.proxy.as_proxy()
    }
}

pub struct WlSurface {
    pub(crate) proxy: WlProxy,
}

impl AsProxy for WlSurface {
    fn as_proxy(&self) -> WlProxyBorrow<'_> {
        self.proxy.as_proxy()
    }
}

pub struct WlOutput {
    pub(crate) proxy: WlProxy,
}

impl AsProxy for WlOutput {
    fn as_proxy(&self) -> WlProxyBorrow<'_> {
        self.proxy.as_proxy()
    }
}
