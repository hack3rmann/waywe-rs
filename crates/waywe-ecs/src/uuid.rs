//! [`UuidBytes`] type

use bevy_platform::{collections::HashMap, hash::FixedHasher};
use sha2_const::Sha256;

pub use uuid::Uuid;
pub use waywe_uuid::{ConstTypeUuid, TypeUuid};

/// A newtype wrapper for UUID bytes that implements Hash and Eq properly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UuidBytes(pub [u8; 16]);

impl UuidBytes {
    /// Constructs [`UuidBytes`] for a given type
    pub fn of<T: TypeUuid>() -> Self {
        Self(T::uuid())
    }

    /// Constructs [`UuidBytes`] from [`Uuid`]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid.into_bytes())
    }

    /// Constructs [`Uuid`] from [`UuidBytes`]
    pub const fn to_uuid(self) -> Uuid {
        Uuid::from_bytes(self.0)
    }
}

#[derive(Clone)]
pub struct UuidBuilder {
    sha: Sha256,
}

impl UuidBuilder {
    pub const fn new(uuid: Uuid) -> Self {
        Self {
            sha: Sha256::new().update(uuid.as_bytes()),
        }
    }

    pub fn base<T: TypeUuid>() -> Self {
        Self::new(Uuid::from_bytes(T::uuid()))
    }

    pub fn add<T: TypeUuid>(mut self) -> Self {
        self.sha = self.sha.update(&T::uuid());
        self
    }

    pub fn add_from_type_id<T: 'static>(mut self) -> Self {
        let uuid = waywe_uuid::type_id_uuid_of::<T>();
        self.sha = self.sha.update(&uuid);
        self
    }

    pub fn build(self) -> Uuid {
        let hash = self.sha.finalize();

        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&hash[..16]);

        Uuid::from_bytes(bytes)
    }
}

/// A map using UUID bytes as keys instead of [`TypeId`](std::any::TypeId), for dynamic library safety
pub type UuidMap<V> = HashMap<UuidBytes, V, FixedHasher>;
