pub mod display;
pub mod ffi;
pub mod object;
pub mod object_storage;
pub mod proxy;
pub mod wire;

pub mod protocols {
    use wayland_scanner::include_wl_interfaces;

    include_wl_interfaces!("wayland-protocols/wayland.xml");

    include_wl_interfaces!("wayland-protocols/stable/xdg-shell/xdg-shell.xml");

    include_wl_interfaces!(
        "wayland-protocols/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml"
    );
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

#[cfg(test)]
mod tests {
    use std::{ffi::CStr, slice};
    use wayland_sys::{wl_interface, wl_message};

    fn get_n_arguments_from_signature(signature: &CStr) -> usize {
        signature
            .to_bytes()
            .iter()
            .filter(|&&byte| byte != b'?' && !byte.is_ascii_digit())
            .count()
    }

    fn check_wl_messages_arrays_are_the_same(
        lhs_ptr: *const wl_message,
        lhs_len: i32,
        rhs_ptr: *const wl_message,
        rhs_len: i32,
    ) {
        assert!(lhs_len >= 0 && rhs_len == lhs_len);

        if !lhs_ptr.is_null() && !rhs_ptr.is_null() {
            let left_methods = unsafe { slice::from_raw_parts(lhs_ptr, lhs_len as usize) };
            let right_methods = unsafe { slice::from_raw_parts(rhs_ptr, rhs_len as usize) };

            for (left, right) in left_methods.iter().zip(right_methods) {
                check_wl_messages_are_the_same(left, right)
            }
        }
    }

    fn check_wl_messages_are_the_same(lhs: &wl_message, rhs: &wl_message) {
        assert!(!lhs.name.is_null() && !rhs.name.is_null());

        let left_name = unsafe { CStr::from_ptr(lhs.name) };
        let right_name = unsafe { CStr::from_ptr(rhs.name) };

        assert_eq!(left_name, right_name, "same names");

        if lhs.signature.is_null() && rhs.signature.is_null() {
            return;
        }

        assert!(
            !lhs.signature.is_null() && !rhs.signature.is_null(),
            "both signatures are present in the {left_name:?} message",
        );

        let left_signature = unsafe { CStr::from_ptr(lhs.signature) };
        let right_signature = unsafe { CStr::from_ptr(rhs.signature) };

        assert_eq!(
            left_signature, right_signature,
            "same signatures for the {left_name:?} message",
        );

        let signature = left_signature;

        let n_args = get_n_arguments_from_signature(signature);

        if lhs.types.is_null() && rhs.types.is_null() {
            return;
        }

        assert!(
            !lhs.types.is_null() && !rhs.types.is_null(),
            "both type arrays are present for the {left_name:?} message, left is {:?}, right is {:?}",
            lhs.types,
            rhs.types,
        );

        let left_interfaces = unsafe { slice::from_raw_parts(lhs.types, n_args) };
        let right_interfaces = unsafe { slice::from_raw_parts(rhs.types, n_args) };

        for (&left, &right) in left_interfaces.iter().zip(right_interfaces) {
            if left.is_null() && right.is_null() {
                continue;
            }

            let left = unsafe { left.as_ref().expect("left interface is null") };
            let right = unsafe { right.as_ref().expect("right interface is null") };

            check_wl_interfaces_are_the_same(left, right, CheckDepth::Shallow);
        }
    }

    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    enum CheckDepth {
        Shallow,
        #[default]
        Deep,
    }

    fn check_wl_interfaces_are_the_same(lhs: &wl_interface, rhs: &wl_interface, depth: CheckDepth) {
        assert!(
            !lhs.name.is_null() && !rhs.name.is_null(),
            "both names exist"
        );

        let left_name = unsafe { CStr::from_ptr(lhs.name) };
        let right_name = unsafe { CStr::from_ptr(rhs.name) };

        assert_eq!(left_name, right_name, "same names");
        assert_eq!(lhs.version, rhs.version, "same versions");

        if let CheckDepth::Deep = depth {
            eprintln!("checking requests are the same in {left_name:?} interface");

            check_wl_messages_arrays_are_the_same(
                lhs.methods,
                lhs.method_count,
                rhs.methods,
                rhs.method_count,
            );

            eprintln!("checking events are the same in {left_name:?} interface");

            check_wl_messages_arrays_are_the_same(
                lhs.events,
                lhs.event_count,
                rhs.events,
                rhs.event_count,
            );
        }
    }

    macro_rules! define_interface_tests {
        ( $( $interface:ident ),* $(,)? ) => {
            $(
                ::paste::paste! {
                    #[test]
                    fn [< $interface _the_same >] () {
                        let external = unsafe { &::wayland_sys:: [< $interface _interface >] };
                        let internal = & $crate ::sys::protocols:: $interface ::WL_INTERFACE;

                        check_wl_interfaces_are_the_same(external, internal, CheckDepth::Deep);
                    }
                }
            )*
        };
    }

    define_interface_tests!(
        wl_display,
        wl_registry,
        wl_callback,
        wl_compositor,
        wl_shm_pool,
        wl_shm,
        wl_buffer,
        wl_data_offer,
        wl_data_source,
        wl_data_device,
        wl_data_device_manager,
        wl_shell,
        wl_shell_surface,
        wl_surface,
        wl_seat,
        wl_pointer,
        wl_keyboard,
        wl_touch,
        wl_output,
        wl_region,
        wl_subcompositor,
        wl_subsurface,
    );
}
