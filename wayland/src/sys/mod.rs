pub mod display;
pub mod ffi;
pub mod proxy;
pub mod wire;
pub mod proxy_registry;

use ffi::wl_interface;

use crate::object::ObjectId;
use core::fmt;
use std::ffi::CStr;

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub enum InterfaceObjectType {
    #[default]
    Display = 1,
    Surface,
    Region,
    Callback,
    Registry,
}

impl InterfaceObjectType {
    pub const fn integer_name(self) -> ObjectId {
        ObjectId::new(self as u32)
    }

    pub const fn backend_interface(self) -> &'static wl_interface {
        use crate::sys::ffi;

        match self {
            Self::Display => unsafe { &ffi::wl_display_interface },
            Self::Surface => unsafe { &ffi::wl_surface_interface },
            Self::Region => unsafe { &ffi::wl_region_interface },
            Self::Callback => unsafe { &ffi::wl_callback_interface },
            Self::Registry => unsafe { &ffi::wl_registry_interface },
        }
    }

    pub const fn interface_name(self) -> &'static CStr {
        match self {
            Self::Display => c"wl_display",
            Self::Surface => c"wl_surface",
            Self::Region => c"wl_region",
            Self::Callback => c"wl_callback",
            Self::Registry => c"wl_registry",
        }
    }
}

impl fmt::Display for InterfaceObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.interface_name().to_str().unwrap())
    }
}

/// Stores some information about some global object
#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct Interface {
    pub object_type: InterfaceObjectType,
    pub version: u32,
}

impl Interface {
    /// Interface for the `wl_display`
    ///
    /// # Note
    ///
    /// `wl_display` always has `version` set to `1`
    pub const DISPLAY: Self = Self {
        object_type: InterfaceObjectType::Display,
        version: 1,
    };

    /// Returns the string name of the interface
    pub const fn interface_name(self) -> &'static CStr {
        self.object_type.interface_name()
    }
}

impl Default for Interface {
    fn default() -> Self {
        Self::DISPLAY
    }
}
