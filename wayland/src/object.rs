use std::num::NonZeroU32;
use thiserror::Error;

/// Ids and names for Wayland objects
#[derive(Clone, Debug, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub struct ObjectId(pub NonZeroU32);

impl ObjectId {
    pub const WL_DISPLAY: ObjectId = ObjectId::new(1);
    pub const WL_REGISTRY: ObjectId = ObjectId::new(2);
    pub const WL_COMPOSITOR: ObjectId = ObjectId::new(3);
    pub const WL_SHM: ObjectId = ObjectId::new(4);
    pub const WP_VIEWPORTER: ObjectId = ObjectId::new(5);
    pub const ZWLR_LAYER_SHELL_V1: ObjectId = ObjectId::new(6);
    pub const WL_SHM_POOL: ObjectId = ObjectId::new(7);
    pub const WL_SURFACE: ObjectId = ObjectId::new(8);

    /// Makes new id from `u32`
    ///
    /// # Panic
    ///
    /// Panics if `value == 0`.
    pub const fn new(value: u32) -> Self {
        Self(NonZeroU32::new(value).unwrap())
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new(1)
    }
}

impl std::hash::Hash for ObjectId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(u32::from(*self));
    }
}

impl From<ObjectId> for u32 {
    fn from(value: ObjectId) -> Self {
        value.0.get()
    }
}

impl TryFrom<u32> for ObjectId {
    type Error = ZeroObjectIdError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(NonZeroU32::new(value).ok_or(ZeroObjectIdError)?))
    }
}

#[derive(Debug, Error)]
#[error("invalid zero `ObjectId`")]
pub struct ZeroObjectIdError;
