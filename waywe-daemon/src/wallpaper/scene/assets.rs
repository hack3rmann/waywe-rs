use super::Scene;
use crate::wallpaper::scene::{
    render::{Extract, RenderPlugin, SceneExtract, SceneRenderer}, ScenePlugin, ScenePostExtract
};
use bevy_ecs::{
    prelude::*,
    system::{StaticSystemParam, SystemParam, SystemParamItem},
};
use smallvec::SmallVec;
use std::{collections::HashMap, fmt, hash, marker::PhantomData};

#[derive(Resource)]
pub struct Assets<A: Asset> {
    last_id: AssetId,
    map: HashMap<AssetId, A>,
    new_ids: SmallVec<[AssetId; 4]>,
}

impl<A: Asset> Assets<A> {
    pub fn new() -> Self {
        Self {
            last_id: AssetId::DUMMY,
            map: HashMap::new(),
            new_ids: SmallVec::new_const(),
        }
    }

    pub fn add(&mut self, asset: A) -> AssetHandle<A> {
        self.last_id = self.last_id.next();
        self.map.insert(self.last_id, asset);
        self.new_ids.push(self.last_id);
        AssetHandle::new(self.last_id)
    }

    pub fn get(&self, handle: AssetHandle<A>) -> Option<&A> {
        self.map.get(&handle.id)
    }

    pub fn get_mut(&mut self, handle: AssetHandle<A>) -> Option<&mut A> {
        self.map.get_mut(&handle.id)
    }

    pub fn new_assets(&self) -> impl ExactSizeIterator<Item = (AssetHandle<A>, &A)> + '_ {
        self.new_ids
            .iter()
            .map(|&id| (AssetHandle::new(id), &self.map[&id]))
    }

    pub fn flush(&mut self) {
        self.new_ids.clear();
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (AssetHandle<A>, &A)> + '_ {
        self.map
            .iter()
            .map(|(&id, asset)| (AssetHandle::new(id), asset))
    }

    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = (AssetHandle<A>, &mut A)> + '_ {
        self.map
            .iter_mut()
            .map(|(&id, asset)| (AssetHandle::new(id), asset))
    }
}

impl<A: Asset> Default for Assets<A> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Asset: Send + Sync + 'static {}

#[derive(Resource)]
pub struct RenderAssets<A: RenderAsset> {
    map: HashMap<AssetId, A>,
}

impl<A: RenderAsset> RenderAssets<A> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(&mut self, handle: AssetHandle<A::Asset>, asset: A) {
        _ = self.map.insert(handle.id, asset);
    }

    pub fn remove(&mut self, handle: AssetHandle<A::Asset>) -> Option<A> {
        self.map.remove(&handle.id)
    }

    pub fn get(&self, handle: AssetHandle<A::Asset>) -> Option<&A> {
        self.map.get(&handle.id)
    }
}

impl<A: RenderAsset> Default for RenderAssets<A> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait RenderAsset: Send + Sync + 'static {
    type Asset: Asset;
    type Param: SystemParam + 'static;

    fn extract(source: &Self::Asset, item: &mut SystemParamItem<'_, '_, Self::Param>) -> Self;
}

pub fn extract_new_render_assets<A: RenderAsset>(
    assets: Extract<Res<Assets<A::Asset>>>,
    mut render_assets: ResMut<RenderAssets<A>>,
    mut param: StaticSystemParam<A::Param>,
) {
    for (id, asset) in assets.new_assets() {
        let render_asset = A::extract(asset, &mut param);
        render_assets.add(id, render_asset);
    }
}

pub fn extract_all_render_assets<A: RenderAsset>(
    assets: Extract<Res<Assets<A::Asset>>>,
    mut render_assets: ResMut<RenderAssets<A>>,
    mut param: StaticSystemParam<A::Param>,
) {
    for (id, asset) in assets.iter() {
        let render_asset = A::extract(asset, &mut param);
        render_assets.add(id, render_asset);
    }
}

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
        f.debug_tuple("AssetHandle").field(&self.id).finish()
    }
}

impl<A> PartialEq for AssetHandle<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<A> hash::Hash for AssetHandle<A> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        <AssetId as hash::Hash>::hash(&self.id, state);
    }
}

impl<A> Eq for AssetHandle<A> {}

pub struct AssetsPlugin<A: Asset> {
    _p: PhantomData<A>,
}

impl<A: Asset> AssetsPlugin<A> {
    pub const fn new() -> Self {
        Self { _p: PhantomData }
    }
}

impl<A: Asset> Default for AssetsPlugin<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Asset> ScenePlugin for AssetsPlugin<A> {
    fn init(self, scene: &mut Scene) {
        scene.add_systems(ScenePostExtract, flush_assets::<A>);
        scene.world.init_resource::<Assets<A>>();
    }
}

impl<A: Asset> RenderPlugin for AssetsPlugin<A> {
    fn init(self, renderer: &mut SceneRenderer) {
        renderer.world.init_resource::<Assets<A>>();
    }
}

pub fn flush_assets<A: Asset>(mut assets: ResMut<Assets<A>>) {
    assets.flush();
}

pub struct RenderAssetsPlugin<A: RenderAsset> {
    do_extact_all: bool,
    _p: PhantomData<A>,
}

impl<A: RenderAsset> RenderAssetsPlugin<A> {
    pub const fn extract_new() -> Self {
        Self {
            do_extact_all: false,
            _p: PhantomData,
        }
    }

    pub const fn extract_all() -> Self {
        Self {
            do_extact_all: true,
            _p: PhantomData,
        }
    }
}

impl<A: RenderAsset> Default for RenderAssetsPlugin<A> {
    fn default() -> Self {
        Self::extract_new()
    }
}

impl<A: RenderAsset> RenderPlugin for RenderAssetsPlugin<A> {
    fn init(self, renderer: &mut SceneRenderer) {
        renderer.world.init_resource::<RenderAssets<A>>();

        if self.do_extact_all {
            renderer.add_systems(SceneExtract, extract_all_render_assets::<A>);
        } else {
            renderer.add_systems(SceneExtract, extract_new_render_assets::<A>);
        }
    }
}
