//! Object identifiers and types

use std::{ffi::CStr, fmt, num::NonZeroU32};
use thiserror::Error;

pub use crate::interface::generated::WlObjectType;

/// Ids and names for Wayland objects
#[derive(Clone, Debug, PartialEq, Eq, Copy, PartialOrd, Ord, Hash)]
pub struct WlObjectId(pub NonZeroU32);

impl WlObjectId {
    /// Makes new id from `u32`
    ///
    /// # Error
    ///
    /// Returns [`Err`] if `value` is `0`
    pub const fn new(value: u32) -> Option<Self> {
        if let Some(raw) = NonZeroU32::new(value) {
            Some(Self(raw))
        } else {
            None
        }
    }
}

impl Default for WlObjectId {
    fn default() -> Self {
        const { Self::new(1).unwrap() }
    }
}

impl From<WlObjectId> for u32 {
    fn from(value: WlObjectId) -> Self {
        value.0.get()
    }
}

impl TryFrom<u32> for WlObjectId {
    type Error = ZeroObjectIdError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(NonZeroU32::new(value).ok_or(ZeroObjectIdError)?))
    }
}

/// [`WlObjectId`] is zero
#[derive(Debug, Error)]
#[error("invalid zero `ObjectId`")]
pub struct ZeroObjectIdError;

impl fmt::Display for WlObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.interface_name())
    }
}

/// Assocciated `ObjectType`
pub trait HasObjectType {
    /// Type the object have
    const OBJECT_TYPE: WlObjectType;
}

/// The type and the integer name for the global object.
#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct InterfaceMessageArgument {
    /// Object type
    pub object_type: WlObjectType,
    /// Numerical name for the object
    pub name: WlObjectId,
}
static_assertions::assert_impl_all!(InterfaceMessageArgument: Send, Sync);

impl InterfaceMessageArgument {
    /// Interface name
    pub const fn interface(self) -> &'static CStr {
        self.object_type.interface().name
    }

    /// Minimal version supported by this crate
    pub const fn min_supported_version(self) -> NonZeroU32 {
        match self.object_type {
            WlObjectType::Shm => const { NonZeroU32::new(1).unwrap() },
            WlObjectType::LayerShell => const { NonZeroU32::new(4).unwrap() },
            _ => self.version(),
        }
    }

    /// Version stored in the interface
    pub const fn version(self) -> NonZeroU32 {
        self.object_type.interface().version
    }

    /// Integer name for this interface
    pub const fn name(self) -> WlObjectId {
        self.name
    }
}
