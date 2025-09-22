use super::wallpaper::Wallpaper;
use crate::{
    box_ext::BoxExt,
    wallpaper::scene::{
        PostStartup, PreUpdate,
        assets::{Asset, Assets, AssetsExtract},
        plugin::Plugin,
        render::SceneExtract,
    },
};
use bevy_ecs::{
    entity::{EntityHash, EntityHasher},
    prelude::*,
};
use crossbeam::channel::Sender;
use smallvec::SmallVec;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt, hash,
    marker::PhantomData,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, RwLock,
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

#[derive(SystemSet, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssetServerSet {
    #[default]
    PopulateAssets,
}

pub struct AssetServerLoadPlugin<A> {
    _p: PhantomData<A>,
}

impl<A> AssetServerLoadPlugin<A> {
    pub const fn new() -> Self {
        Self { _p: PhantomData }
    }
}

impl<A> Default for AssetServerLoadPlugin<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Asset + Load> Plugin for AssetServerLoadPlugin<A> {
    fn build(&self, wallpaper: &mut Wallpaper) {
        let server = wallpaper.main.resource::<AssetServer>();
        let assets = wallpaper.main.resource::<Assets<A>>();
        server.register_assets(assets);

        wallpaper.main.add_systems(
            PreUpdate,
            populate_assets::<A>.in_set(AssetServerSet::PopulateAssets),
        );
        wallpaper.main.add_systems(
            PostStartup,
            populate_assets::<A>.in_set(AssetServerSet::PopulateAssets),
        );
    }
}

pub fn populate_assets<A: Asset + Load>(server: Res<AssetServer>, mut assets: ResMut<Assets<A>>) {
    server.populate_assets(&mut assets);
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
    loaded_assets: Mutex<HashMap<(PathBuf, TypeId), (AssetId, Box<dyn Any + Send>)>>,
    drop_senders: RwLock<HashMap<TypeId, Sender<AssetDropEvent>>>,
}

impl AssetServerInner {
    pub fn new() -> Self {
        let id_generator = AssetIdGenerator::new();

        Self {
            id_generator,
            loaded_assets: Mutex::default(),
            drop_senders: RwLock::default(),
        }
    }

    pub fn register_assets<A: Asset>(&self, assets: &Assets<A>) {
        let mut senders = self.drop_senders.write().unwrap();
        senders.insert(TypeId::of::<A>(), assets.get_drop_sender());
    }

    pub fn make_assets<A: Asset>(&self) -> Assets<A> {
        Assets::new(self.id_generator.clone())
    }

    pub fn load<A: Asset + Load>(&self, path: impl Into<PathBuf>) -> AssetHandle<A> {
        let path = path.into();
        let asset = A::load(&path);
        let id = self.id_generator.next_id();

        {
            let mut assets = self.loaded_assets.lock().unwrap();
            assets.insert((path, TypeId::of::<A>()), (id, Box::new(asset)));
        }

        let drop_sender = {
            let senders = self.drop_senders.read().unwrap();
            senders[&TypeId::of::<A>()].clone()
        };

        AssetHandle::new(UntypedAssetHandle::new(id, drop_sender))
    }

    pub fn populate_assets<A: Asset + Load>(&self, assets: &mut Assets<A>) {
        let mut loaded_assets = self.loaded_assets.lock().unwrap();

        let keys: SmallVec<[_; 16]> = loaded_assets
            .keys()
            .filter(|(_, id)| id == &TypeId::of::<A>())
            .cloned()
            .collect();

        for key in keys {
            let (id, asset) = loaded_assets.remove(&key).unwrap();
            let asset = asset.downcast::<A>().unwrap().into_inner();
            assets.insert(id, asset);
        }
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

pub trait Load: Any + Send {
    fn load(path: &Path) -> Self
    where
        Self: Sized;
}
