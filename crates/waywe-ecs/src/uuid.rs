use bevy_platform::{collections::HashMap, hash::FixedHasher};

pub use waywe_uuid::TypeUuid;

/// A newtype wrapper for UUID bytes that implements Hash and Eq properly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UuidBytes(pub [u8; 16]);

/// A map using UUID bytes as keys instead of TypeId, for dynamic library safety
pub type UuidMap<V> = HashMap<UuidBytes, V, FixedHasher>;
