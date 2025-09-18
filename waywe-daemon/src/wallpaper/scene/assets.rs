use bevy_ecs::prelude::*;
use std::{collections::HashMap, fmt, marker::PhantomData};

#[derive(Resource)]
pub struct Assets<A: Asset> {
    last_id: AssetId,
    map: HashMap<AssetId, A>,
}

impl<A: Asset> Assets<A> {
    pub fn new() -> Self {
        Self {
            last_id: AssetId::DUMMY,
            map: HashMap::new(),
        }
    }

    pub fn add(&mut self, asset: A) -> AssetHandle<A> {
        self.last_id = self.last_id.next();
        self.map.insert(self.last_id, asset);
        AssetHandle::new(self.last_id)
    }

    pub fn get(&self, handle: AssetHandle<A>) -> Option<&A> {
        self.map.get(&handle.id)
    }

    pub fn get_mut(&mut self, handle: AssetHandle<A>) -> Option<&mut A> {
        self.map.get_mut(&handle.id)
    }
}

impl<A: Asset> Default for Assets<A> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Asset: Send + 'static {}

// TODO(hack3rmann): hash it faster
#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct AssetId(pub u32);

impl AssetId {
    pub const DUMMY: Self = Self(0);

    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

pub struct AssetHandle<A> {
    pub id: AssetId,
    _p: PhantomData<A>,
}

impl<A> AssetHandle<A> {
    pub const fn new(id: AssetId) -> Self {
        Self {
            id,
            _p: PhantomData,
        }
    }
}

impl<A> Clone for AssetHandle<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for AssetHandle<A> {}

impl<A> fmt::Debug for AssetHandle<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AssetHandle")
            .field(&self.id)
            .finish()
    }
}

impl<A> PartialEq for AssetHandle<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<A> Eq for AssetHandle<A> {}
