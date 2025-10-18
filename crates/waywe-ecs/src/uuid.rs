use bevy_platform::{collections::HashMap, hash::FixedHasher};
use uuid::Uuid;

pub use waywe_uuid::TypeUuid;

/// A newtype wrapper for UUID bytes that implements Hash and Eq properly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UuidBytes(pub [u8; 16]);

impl UuidBytes {
    pub const fn of<T: TypeUuid>() -> Self {
        Self(T::UUID)
    }

    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid.into_bytes())
    }

    pub const fn to_uuid(self) -> Uuid {
        Uuid::from_bytes(self.0)
    }
}

/// A map using UUID bytes as keys instead of TypeId, for dynamic library safety
pub type UuidMap<V> = HashMap<UuidBytes, V, FixedHasher>;
