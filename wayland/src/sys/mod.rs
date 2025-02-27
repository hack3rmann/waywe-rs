pub mod display;
pub mod ffi;
pub mod proxy;
pub mod wire;

use crate::object::ObjectId;
use core::fmt;
use std::ffi::CStr;

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

/// Stores some information about some global object
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
