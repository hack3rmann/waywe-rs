//! Asset management system for wallpapers.
//!
//! This module provides a generic asset management system that handles
//! loading, storing, and extracting assets between the main and render worlds.
//!
//! # Core Types
//!
//! - [`Assets`]: Store and manage assets in the main world
//! - [`RenderAssets`]: GPU-ready versions of assets in the render world
//! - [`AssetHandle`]: Type-safe references to assets
//! - [`AssetId`]: Unique identifiers for assets
//!
//! # Plugins
//!
//! - [`AssetsPlugin`]: Manage assets of type T in the main world
//! - [`RenderAssetsPlugin`]: Manage GPU-ready assets of type T in the render world

use super::wallpaper::Wallpaper;
use crate::{
    PostExtract,
    asset_server::{
        AssetDropEvent, AssetHandle, AssetId, AssetIdGenerator, AssetIdHashMap, AssetServer,
        UntypedAssetHandle,
    },
    extract::Extract,
    plugin::{AddPlugins, Plugin},
    render::{Render, RenderSet, SceneExtract},
};
use crossbeam::channel::{self, Receiver, Sender};
use smallvec::SmallVec;
use std::marker::PhantomData;
use thiserror::Error;
use tracing::error;
use waywe_ecs::{
    prelude::*,
    system::{StaticSystemParam, SystemParam, SystemParamItem},
};
use waywe_uuid::TypeUuid;

pub type IdSmallVec = SmallVec<[AssetId; 4]>;

/// Collection of assets of a specific type.
///
/// Assets are stored with unique IDs and can be accessed by handle.
#[derive(Resource, TypeUuid)]
#[uuid = "ed441c11-eddf-4ce9-9a9c-cf1fc17b7b81"]
pub struct Assets<A: Asset> {
    map: AssetIdHashMap<A>,
    changed_ids: IdSmallVec,
    new_ids: IdSmallVec,
    remove_ids: IdSmallVec,
    drop_receiver: Receiver<AssetDropEvent>,
    drop_sender: Sender<AssetDropEvent>,
    id_generator: AssetIdGenerator,
}

impl<A: Asset> Assets<A> {
    pub fn new(id_generator: AssetIdGenerator) -> Self {
        let (drop_sender, drop_receiver) = channel::unbounded();

        Self {
            map: AssetIdHashMap::default(),
            changed_ids: SmallVec::new(),
            new_ids: SmallVec::new(),
            remove_ids: SmallVec::new(),
            drop_receiver,
            drop_sender,
            id_generator,
        }
    }

    fn next_handle(&self) -> AssetHandle<A> {
        let id = self.id_generator.next_id();
        AssetHandle::new(UntypedAssetHandle::new(id, self.drop_sender.clone()))
    }

    pub fn get_drop_sender(&self) -> Sender<AssetDropEvent> {
        self.drop_sender.clone()
    }

    pub fn insert(&mut self, id: AssetId, asset: A) {
        _ = self.map.insert(id, asset);
        self.new_ids.push(id);
    }

    /// Add an asset to the collection and return a handle to it.
    pub fn add(&mut self, asset: A) -> AssetHandle<A> {
        let handle = self.next_handle();
        self.insert(handle.id(), asset);
        handle
    }

    /// Get a reference to an asset by handle.
    pub fn get(&self, id: AssetId) -> Option<&A> {
        self.map.get(&id)
    }

    pub fn set_changed(&mut self, id: AssetId) {
        self.changed_ids.push(id);
    }

    /// Get a mutable reference to an asset by handle.
    pub fn get_mut(&mut self, id: AssetId) -> Option<&mut A> {
        self.set_changed(id);
        self.map.get_mut(&id)
    }

    /// Iterate over newly added assets.
    ///
    /// This is used during extraction to transfer new assets to the render world.
    pub fn new_assets(&self) -> impl ExactSizeIterator<Item = (AssetId, &A)> + '_ {
        self.new_ids.iter().map(|&id| (id, &self.map[&id]))
    }

    pub fn changed_assets(&self) -> impl ExactSizeIterator<Item = (AssetId, &A)> + '_ {
        self.changed_ids.iter().map(|&id| (id, &self.map[&id]))
    }

    pub fn removed_assets(&self) -> &[AssetId] {
        &self.remove_ids
    }

    /// Clear the list of new assets and remove droppped assets.
    ///
    /// This should be called after extracting new assets to the render world.
    pub fn flush(&mut self) {
        self.new_ids.clear();
        self.changed_ids.clear();

        for id in self.remove_ids.drain(..) {
            if self.map.remove(&id).is_none() {
                error!(
                    ?id,
                    asset_type = std::any::type_name::<A>(),
                    "trying to remove assets that does not exist"
                );
            }
        }
    }

    /// Iterate over all assets.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (AssetId, &A)> + '_ {
        self.map.iter().map(|(&id, asset)| (id, asset))
    }

    /// Iterate over all assets with mutable references.
    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = (AssetId, &mut A)> + '_ {
        self.map.iter_mut().map(|(&id, asset)| (id, asset))
    }

    pub fn remove_droppped(&mut self) {
        while let Ok(AssetDropEvent(id)) = self.drop_receiver.try_recv() {
            self.remove_ids.push(id);
        }
    }
}

impl<A: Asset> FromWorld for Assets<A> {
    fn from_world(world: &mut World) -> Self {
        world.resource::<AssetServer>().make_assets()
    }
}

#[derive(Resource, TypeUuid)]
#[uuid = "bcdc0211-c6ec-47e7-b812-82481c7c7650"]
pub struct RefAssets<A: Asset> {
    map: AssetIdHashMap<A>,
    removed_ids: IdSmallVec,
}

impl<A: Asset> RefAssets<A> {
    pub fn new() -> Self {
        Self {
            map: AssetIdHashMap::default(),
            removed_ids: SmallVec::new(),
        }
    }

    pub fn get(&self, id: AssetId) -> Option<&A> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: AssetId) -> Option<&mut A> {
        self.map.get_mut(&id)
    }

    pub fn insert(&mut self, id: AssetId, asset: A) {
        self.map.insert(id, asset);
    }

    pub fn insert_with(&mut self, id: AssetId, make_asset: impl FnOnce() -> A) {
        self.map.entry(id).or_insert_with(make_asset);
    }

    pub fn remove(&mut self, id: AssetId) {
        self.removed_ids.push(id);
    }

    pub fn removed_assets(&self) -> &[AssetId] {
        &self.removed_ids
    }

    pub fn flush(&mut self) {
        for id in self.removed_ids.drain(..) {
            if self.map.remove(&id).is_none() {
                error!(
                    ?id,
                    ref_asset_type = std::any::type_name::<A>(),
                    "trying to remove assets that does not exist"
                );
            }
        }
    }
}

impl<A: Asset> Default for RefAssets<A> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn flush_ref_assets<R: Asset>(mut assets: ResMut<RefAssets<R>>) {
    assets.flush();
}

pub fn cleanup_ref_assets<R: Asset, A: Asset>(
    assets: Extract<Res<Assets<A>>>,
    mut ref_assets: ResMut<RefAssets<R>>,
) {
    for &id in assets.removed_assets() {
        ref_assets.remove(id);
    }
}

pub fn cleanup_ref_to_ref_assets<R1: Asset, R2: Asset>(
    from: Res<RefAssets<R1>>,
    mut to: ResMut<RefAssets<R2>>,
) {
    for &id in from.removed_assets() {
        to.remove(id);
    }
}

pub struct RefAssetsPlugin<A> {
    _p: PhantomData<A>,
}

impl<A> RefAssetsPlugin<A> {
    pub const fn new() -> Self {
        Self { _p: PhantomData }
    }
}

impl<A> Default for RefAssetsPlugin<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Asset> Plugin for RefAssetsPlugin<A> {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper
            .render
            .init_resource::<RefAssets<A>>()
            .add_systems(Render, flush_ref_assets::<A>);
    }
}

pub struct RefAssetsDependencyPlugin<R, A> {
    _p: PhantomData<(R, A)>,
}

impl<R, A> RefAssetsDependencyPlugin<R, A> {
    pub const fn new() -> Self {
        Self { _p: PhantomData }
    }
}

impl<R, A> Default for RefAssetsDependencyPlugin<R, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Asset, A: Asset> Plugin for RefAssetsDependencyPlugin<R, A> {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.render.add_systems(
            SceneExtract,
            cleanup_ref_assets::<R, A>.in_set(AssetsExtract::AssetsToRef),
        );
    }
}

pub struct RefAssetsRefDependencyPlugin<R1, R2> {
    _p: PhantomData<(R1, R2)>,
}

impl<R1, R2> RefAssetsRefDependencyPlugin<R1, R2> {
    pub const fn new() -> Self {
        Self { _p: PhantomData }
    }
}

impl<R1, R2> Default for RefAssetsRefDependencyPlugin<R1, R2> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R1: Asset, R2: Asset> Plugin for RefAssetsRefDependencyPlugin<R1, R2> {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.render.add_systems(
            SceneExtract,
            cleanup_ref_to_ref_assets::<R1, R2>.in_set(AssetsExtract::RefToRef),
        );
    }
}

/// Trait for types that can be used as assets.
///
/// Assets must be sendable between threads and have a static lifetime.
pub trait Asset: TypeUuid + Send + Sync + 'static {}

/// Collection of GPU-ready assets.
#[derive(Resource, TypeUuid)]
#[uuid = "29e4afc4-1c6e-4ae5-8fa8-1ac428c6a707"]
pub struct RenderAssets<A: RenderAsset> {
    map: AssetIdHashMap<A>,
    removed_ids: IdSmallVec,
}

impl<A: RenderAsset> RenderAssets<A> {
    /// Create a new empty render asset collection.
    pub fn new() -> Self {
        Self {
            map: AssetIdHashMap::default(),
            removed_ids: SmallVec::new(),
        }
    }

    /// Add a render asset.
    pub fn add(&mut self, id: AssetId, asset: A) {
        _ = self.map.insert(id, asset);
    }

    /// Remove a render asset.
    pub fn remove(&mut self, id: AssetId) {
        self.removed_ids.push(id);
    }

    /// Get a reference to a render asset by handle.
    pub fn get(&self, id: AssetId) -> Option<&A> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: AssetId) -> Option<&mut A> {
        self.map.get_mut(&id)
    }

    pub fn flush(&mut self) {
        for id in self.removed_ids.drain(..) {
            if self.map.remove(&id).is_none() {
                error!(
                    ?id,
                    ref_asset_type = std::any::type_name::<A>(),
                    "trying to remove assets that does not exist"
                );
            }
        }
    }
}

impl<A: RenderAsset> Default for RenderAssets<A> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for assets that have GPU-ready counterparts.
pub trait RenderAsset: TypeUuid + Send + Sync + 'static {
    /// The source asset type.
    type Asset: Asset;
    /// System parameters needed for extraction.
    type Param: SystemParam + 'static;

    const REPLACE_ON_UPDATE: bool = true;

    /// Extract a render asset from a source asset.
    fn extract(
        source: &Self::Asset,
        item: &mut SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<Self, RenderAssetExtractError>
    where
        Self: Sized;

    #[expect(unused_variables)]
    fn update(&mut self, source: &Self::Asset, item: &mut SystemParamItem<'_, '_, Self::Param>) {
        debug_assert!(
            Self::REPLACE_ON_UPDATE,
            "unexpected unimplemented RenderAsset::update"
        );
    }
}

#[derive(Clone, Debug, Error)]
pub enum RenderAssetExtractError {
    #[error("this asset should be skipped during extract schedule")]
    Skip,
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AssetsExtract {
    #[default]
    MainToRender,
    AssetsToRef,
    RefToRef,
}

/// System to extract new render assets.
///
/// This system is automatically added by [`RenderAssetsPlugin`] and
/// transfers newly added assets from the main world to the render world.
pub fn extract_new_render_assets<A: RenderAsset>(
    assets: Extract<Res<Assets<A::Asset>>>,
    mut render_assets: ResMut<RenderAssets<A>>,
    mut param: StaticSystemParam<A::Param>,
) {
    let mut new_assets = assets.new_assets();
    let mut all_assets = Iterator::chain(assets.new_assets(), assets.changed_assets());

    let assets: &mut dyn Iterator<Item = (AssetId, &A::Asset)> = if A::REPLACE_ON_UPDATE {
        &mut all_assets
    } else {
        &mut new_assets
    };

    for (id, asset) in assets {
        let render_asset = match A::extract(asset, &mut param) {
            Ok(asset) => asset,
            Err(err @ RenderAssetExtractError::Skip) => panic!(
                "{} - cannot skip asset which can only be updated on insert",
                err
            ),
        };

        render_assets.add(id, render_asset);
    }
}

/// System to extract all render assets.
///
/// This system transfers all assets from the main world to the render world.
pub fn extract_all_render_assets<A: RenderAsset>(
    assets: Extract<Res<Assets<A::Asset>>>,
    mut render_assets: ResMut<RenderAssets<A>>,
    mut param: StaticSystemParam<A::Param>,
) {
    for (id, asset) in assets.iter() {
        let render_asset = match A::extract(asset, &mut param) {
            Ok(asset) => asset,
            Err(RenderAssetExtractError::Skip) => continue,
        };

        render_assets.add(id, render_asset);
    }
}

pub fn update_render_assets<A: RenderAsset>(
    assets: Extract<Res<Assets<A::Asset>>>,
    mut render_assets: ResMut<RenderAssets<A>>,
    mut param: StaticSystemParam<A::Param>,
) {
    for (id, asset) in assets.changed_assets() {
        let Some(render_asset) = render_assets.get_mut(id) else {
            // just insert new asset
            let render_asset = match A::extract(asset, &mut param) {
                Ok(asset) => asset,
                Err(RenderAssetExtractError::Skip) => continue,
            };
            render_assets.add(id, render_asset);
            continue;
        };

        render_asset.update(asset, &mut param);
    }
}

pub fn remove_render_assets<A: RenderAsset>(
    assets: Extract<Res<Assets<A::Asset>>>,
    mut render_assets: ResMut<RenderAssets<A>>,
) {
    for &id in assets.removed_assets() {
        render_assets.remove(id);
    }
}

pub fn flush_render_assets<A: RenderAsset>(mut assets: ResMut<RenderAssets<A>>) {
    assets.flush();
}

/// Plugin for managing assets in the main world.
pub struct AssetsPlugin<A: Asset> {
    add: AddPlugins,
    _p: PhantomData<A>,
}

impl<A: Asset> AssetsPlugin<A> {
    /// Create a new assets plugin for the main world.
    pub const fn new() -> Self {
        Self {
            add: AddPlugins::MAIN,
            _p: PhantomData,
        }
    }

    /// Create a new assets plugin for the render world.
    pub const fn new_render() -> Self {
        Self {
            add: AddPlugins::RENDER,
            _p: PhantomData,
        }
    }
}

impl<A: Asset> Default for AssetsPlugin<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Asset> Plugin for AssetsPlugin<A> {
    fn build(&self, wallpaper: &mut Wallpaper) {
        if self.add.contains(AddPlugins::MAIN) {
            wallpaper
                .main
                .add_systems(PostExtract, update_assets::<A>)
                .init_resource::<Assets<A>>();
        }

        if self.add.contains(AddPlugins::RENDER) {
            wallpaper.render.init_resource::<Assets<A>>();
        }
    }
}

/// System to flush new assets.
///
/// This clears the list of new assets after they've been extracted.
pub fn update_assets<A: Asset>(mut assets: ResMut<Assets<A>>) {
    assets.flush();
    assets.remove_droppped();
}

/// Plugin for managing GPU-ready assets in the render world.
pub struct RenderAssetsPlugin<A: RenderAsset> {
    do_extact_all: bool,
    _p: PhantomData<A>,
}

impl<A: RenderAsset> RenderAssetsPlugin<A> {
    /// Create a plugin that extracts only new assets.
    pub const fn extract_new() -> Self {
        Self {
            do_extact_all: false,
            _p: PhantomData,
        }
    }

    /// Create a plugin that extracts all assets.
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

impl<A: RenderAsset> Plugin for RenderAssetsPlugin<A> {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.render.init_resource::<RenderAssets<A>>();

        wallpaper
            .render
            .add_systems(
                SceneExtract,
                remove_render_assets::<A>.in_set(AssetsExtract::MainToRender),
            )
            .add_systems(Render, flush_render_assets::<A>.in_set(RenderSet::Update));

        if self.do_extact_all {
            wallpaper.render.add_systems(
                SceneExtract,
                extract_all_render_assets::<A>.in_set(AssetsExtract::MainToRender),
            );
        } else {
            wallpaper.render.add_systems(
                SceneExtract,
                extract_new_render_assets::<A>.in_set(AssetsExtract::MainToRender),
            );
        }

        if !A::REPLACE_ON_UPDATE {
            wallpaper.render.add_systems(
                SceneExtract,
                update_render_assets::<A>.in_set(AssetsExtract::MainToRender),
            );
        }
    }
}
