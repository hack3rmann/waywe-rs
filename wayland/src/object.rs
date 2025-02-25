use std::num::NonZeroU32;

use fxhash::FxHashMap;
use thiserror::Error;

/// Ids and names for Wayland objects
#[derive(Clone, Debug, PartialEq, Eq, Copy, PartialOrd, Ord, Hash)]
pub struct ObjectId(pub NonZeroU32);

impl ObjectId {
    pub const WL_DISPLAY: ObjectId = ObjectId::new(1);
    pub const WL_REGISTRY: ObjectId = ObjectId::new(2);
    pub const WL_COMPOSITOR: ObjectId = ObjectId::new(3);
    pub const WL_SHM: ObjectId = ObjectId::new(4);
    pub const WP_VIEWPORTER: ObjectId = ObjectId::new(5);
    pub const ZWLR_LAYER_SHELL_V1: ObjectId = ObjectId::new(6);
    pub const WL_SHM_POOL: ObjectId = ObjectId::new(7);

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

/// Unique identifier provider for Wayland objects
#[derive(Clone, Debug, PartialEq, Default, Eq, Copy, PartialOrd, Ord, Hash)]
pub struct ObjectIdProvider {
    pub last: ObjectId,
}

impl ObjectIdProvider {
    /// Creates new [`ObjectIdProvider`].
    pub const fn new(last_used: ObjectId) -> Self {
        Self {
            last: ObjectId::new(last_used.0.get()),
        }
    }

    /// Gives the next available id. Basically, `prev_id + 1`
    pub const fn next_id(&mut self) -> ObjectId {
        let result = self.last;
        self.last.0 = NonZeroU32::new(self.last.0.get().wrapping_add(1)).unwrap();
        result
    }

    pub fn from_mapped_ids(map: &ObjectIdMap) -> Self {
        let max_id = map.iter().map(|(&_name, &id)| id).max().unwrap();
        Self::new(max_id)
    }
}

/// A bijective map that maps object names to their ids
#[derive(Clone, Debug, PartialEq, Default, Eq)]
pub struct ObjectIdMap {
    pub(crate) name_to_id: FxHashMap<ObjectId, ObjectId>,
    pub(crate) id_to_name: FxHashMap<ObjectId, ObjectId>,
}

impl ObjectIdMap {
    /// Constructs new [`ObjectIdMap`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns object id by the object name
    pub fn get_id(&self, name: ObjectId) -> Option<ObjectId> {
        self.name_to_id.get(&name).copied()
    }

    /// Returns object name by the object id
    pub fn get_name(&self, id: ObjectId) -> Option<ObjectId> {
        self.id_to_name.get(&id).copied()
    }

    /// Inserts new name-id pair
    pub fn map(&mut self, name: ObjectId, id: ObjectId) {
        self.name_to_id.insert(name, id);
        self.id_to_name.insert(id, name);
    }

    /// Returns an iterator over `(name, id)` pairs
    pub fn iter(&self) -> <&'_ FxHashMap<ObjectId, ObjectId> as IntoIterator>::IntoIter {
        self.name_to_id.iter()
    }
}

impl IntoIterator for ObjectIdMap {
    type Item = (ObjectId, ObjectId);
    type IntoIter = <FxHashMap<ObjectId, ObjectId> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.name_to_id.into_iter()
    }
}

impl<'s> IntoIterator for &'s ObjectIdMap {
    type Item = (&'s ObjectId, &'s ObjectId);
    type IntoIter = <&'s FxHashMap<ObjectId, ObjectId> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.name_to_id.iter()
    }
}
