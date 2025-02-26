pub mod display;
pub mod ffi;
pub mod proxy;
pub mod wire;

use core::fmt;
use ffi::{wl_proxy, wl_proxy_destroy};
use std::{ffi::CStr, ptr::NonNull};

use crate::object::ObjectId;

pub trait FromProxy: Sized {
    fn from_proxy() -> Self;
}

pub struct WlObject {
    pub(crate) proxy: NonNull<wl_proxy>,
    pub(crate) interface: &'static Interface,
}

impl WlObject {
    pub const fn raw_proxy_ptr(&self) -> NonNull<wl_proxy> {
        self.proxy
    }
}

impl Drop for WlObject {
    fn drop(&mut self) {
        unsafe { wl_proxy_destroy(self.proxy.as_ptr()) };
    }
}

impl fmt::Debug for WlObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WlObject(\"{}\")",
            self.interface.interface_name().to_str().unwrap()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectType {
    #[default]
    Display = 1,
}

impl ObjectType {
    pub const fn integer_name(self) -> ObjectId {
        ObjectId::new(self as u32)
    }

    pub const fn interface_name(self) -> &'static CStr {
        match self {
            Self::Display => c"wl_display",
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.interface_name().to_str().unwrap())
    }
}

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct Interface {
    pub object_type: ObjectType,
    pub version: u32,
}

impl Interface {
    /// Interface for the `wl_display`
    ///
    /// # Note
    ///
    /// `wl_display` always has `version` set to `1`
    pub const DISPLAY: Self = Self {
        object_type: ObjectType::Display,
        version: 1,
    };

    pub const fn interface_name(self) -> &'static CStr {
        self.object_type.interface_name()
    }
}

impl Default for Interface {
    fn default() -> Self {
        Self::DISPLAY
    }
}
