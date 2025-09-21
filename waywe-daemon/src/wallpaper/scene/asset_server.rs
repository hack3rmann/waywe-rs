use super::wallpaper::Wallpaper;
use crate::wallpaper::scene::{
    assets::{Asset, Assets, AssetsExtract},
    plugin::Plugin,
    render::SceneExtract,
};
use bevy_ecs::{
    entity::{EntityHash, EntityHasher},
    prelude::*,
};
use crossbeam::channel::Sender;
use std::{
    collections::HashMap,
    fmt, hash,
    marker::PhantomData,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering::*},
    },
};

pub struct AssetServerPlugin;

impl Plugin for AssetServerPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        let server = AssetServer::default();
        wallpaper.main.insert_resource(server.clone());
        wallpaper.render.insert_resource(server);
        wallpaper.render.configure_sets(
            SceneExtract,
            (
                AssetsExtract::MainToRender,
                AssetsExtract::AssetsToRef,
                AssetsExtract::RefToRef,
            )
                .chain(),
        );
    }
}

#[derive(Clone, Default, Debug)]
pub struct AssetIdGenerator {
    current: Arc<AtomicU64>,
}

impl AssetIdGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_id(&self) -> AssetId {
        let next = self.current.fetch_add(1, Relaxed);
        assert_ne!(next, u64::MAX, "runned out of available assets ids");
        AssetId(next)
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetId(pub u64);

#[derive(Resource, Debug, Default, Clone)]
pub struct AssetServer(Arc<AssetServerInner>);

impl Deref for AssetServer {
    type Target = AssetServerInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct AssetServerInner {
    id_generator: AssetIdGenerator,
}

impl AssetServerInner {
    pub fn new() -> Self {
        let id_generator = AssetIdGenerator::new();

        Self { id_generator }
    }

    pub fn make_assets<A: Asset>(&self) -> Assets<A> {
        Assets::new(self.id_generator.clone())
    }
}

impl Default for AssetServerInner {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AssetHandle<A> {
    untyped: UntypedAssetHandle,
    _p: PhantomData<A>,
}

impl<A> AssetHandle<A> {
    pub fn new(untyped: UntypedAssetHandle) -> Self {
        Self {
            untyped,
            _p: PhantomData,
        }
    }

    pub fn ref_count(&self) -> usize {
        self.untyped.ref_count()
    }

    pub fn into_untyped(self) -> UntypedAssetHandle {
        self.untyped
    }
}

impl<A> Deref for AssetHandle<A> {
    type Target = AssetHandleInner;

    fn deref(&self) -> &Self::Target {
        &self.untyped.0
    }
}

impl<A> Clone for AssetHandle<A> {
    fn clone(&self) -> Self {
        Self {
            untyped: self.untyped.clone(),
            _p: PhantomData,
        }
    }
}

impl<A> fmt::Debug for AssetHandle<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetHandle")
            .field("id", &self.id())
            .finish()
    }
}

impl<A> PartialEq for AssetHandle<A> {
    fn eq(&self, other: &Self) -> bool {
        self.untyped == other.untyped
    }
}

impl<A> Eq for AssetHandle<A> {}

impl<A> PartialOrd for AssetHandle<A> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A> Ord for AssetHandle<A> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.untyped.cmp(&other.untyped)
    }
}

impl<A> hash::Hash for AssetHandle<A> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.untyped.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct UntypedAssetHandle(Arc<AssetHandleInner>);

impl Deref for UntypedAssetHandle {
    type Target = AssetHandleInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UntypedAssetHandle {
    pub fn new(id: AssetId, drop_sender: Sender<AssetDropEvent>) -> Self {
        Self(Arc::new(AssetHandleInner::new(id, drop_sender)))
    }

    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

#[derive(Clone, Debug)]
pub struct AssetHandleInner {
    id: AssetId,
    drop_sender: Sender<AssetDropEvent>,
}

impl AssetHandleInner {
    pub fn new(id: AssetId, drop_sender: Sender<AssetDropEvent>) -> Self {
        Self { id, drop_sender }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }
}

impl Drop for AssetHandleInner {
    fn drop(&mut self) {
        // NOTE(hack3rmann): returns error if drop event has not received.
        // But it is fine, because it such case all asset resources have deallocated
        // at this point
        _ = self.drop_sender.send(AssetDropEvent(self.id));
    }
}

impl PartialEq for AssetHandleInner {
    fn eq(&self, other: &Self) -> bool {
        self.id.0 == other.id.0
    }
}

impl Eq for AssetHandleInner {}

impl Ord for AssetHandleInner {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.0.cmp(&other.id.0)
    }
}

impl PartialOrd for AssetHandleInner {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl hash::Hash for AssetHandleInner {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&self.id.0, state);
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetDropEvent(pub AssetId);

pub type AssetIdHash = EntityHash;
pub type AssetIdHasher = EntityHasher;
pub type AssetIdHashMap<T> = HashMap<AssetId, T, AssetIdHash>;
