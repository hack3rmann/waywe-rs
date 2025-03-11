pub mod display;
pub mod ffi;
pub mod object;
pub mod object_storage;
pub mod proxy;
pub mod wire;

pub mod protocols {
    use wayland_scanner::include_wl_interfaces;

    include_wl_interfaces!("wayland-protocols/wayland.xml");

    // TODO(hack3rmann): uncomment
    //
    // include_wl_interfaces!("wayland-protocols/stable/xdg-shell/xdg-shell.xml");
    // include_wl_interfaces!(
    //     "wayland-protocols/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml"
    // );
}

use crate::object::ObjectId;
use core::fmt;
use ffi::wl_interface;
use std::ffi::CStr;
use wayland_sys::Interface as FfiInterface;

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub enum InterfaceObjectType {
    #[default]
    Display = 1,
    Registry = 2,
    Compositor = 3,
    ShmPool = 4,
    Shm = 5,
    Buffer = 6,
    Surface = 13,
    Region = 19,
    Callback,
}

impl InterfaceObjectType {
    pub const fn integer_name(self) -> ObjectId {
        ObjectId::new(self as u32)
    }

    pub const fn backend_ffi_interface(self) -> &'static FfiInterface<'static> {
        // FIXME(hack3rmann): add statics for zwlr_layer_shell_v1 and zwlr_layer_surface_v1
        match self {
            Self::Display => &protocols::wl_display::INTERFACE,
            Self::Surface => &protocols::wl_surface::INTERFACE,
            Self::Region => &protocols::wl_region::INTERFACE,
            Self::Callback => &protocols::wl_callback::INTERFACE,
            Self::Registry => &protocols::wl_registry::INTERFACE,
            Self::ShmPool => &protocols::wl_shm_pool::INTERFACE,
            Self::Compositor => &protocols::wl_compositor::INTERFACE,
            Self::Shm => &protocols::wl_shm::INTERFACE,
            Self::Buffer => &protocols::wl_buffer::INTERFACE,
        }
    }

    pub const fn backend_interface(self) -> &'static wl_interface {
        // FIXME(hack3rmann): add statics for zwlr_layer_shell_v1 and zwlr_layer_surface_v1
        match self {
            Self::Display => &protocols::wl_display::WL_INTERFACE,
            Self::Surface => &protocols::wl_surface::WL_INTERFACE,
            Self::Region => &protocols::wl_region::WL_INTERFACE,
            Self::Callback => &protocols::wl_callback::WL_INTERFACE,
            Self::Registry => &protocols::wl_registry::WL_INTERFACE,
            Self::ShmPool => &protocols::wl_shm_pool::WL_INTERFACE,
            Self::Compositor => &protocols::wl_compositor::WL_INTERFACE,
            Self::Shm => &protocols::wl_shm::WL_INTERFACE,
            Self::Buffer => &protocols::wl_buffer::WL_INTERFACE,
        }
    }

    pub const fn interface_name(self) -> &'static CStr {
        self.backend_ffi_interface().name
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
