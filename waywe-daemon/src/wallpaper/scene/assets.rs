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
//!
//! # Usage
//!
//! ```rust
//! use waywe_daemon::wallpaper::scene::{
//!     assets::{Assets, AssetHandle},
//!     image::Image,
//! };
//!
//! // Add an asset
//! // let mut images = wallpaper.main.resource_mut::<Assets<Image>>();
//! // let handle: AssetHandle<Image> = images.add(Image::new_white_1x1());
//!
//! // Use the asset
//! // wallpaper.main.world.spawn(handle);
//! ```

use super::wallpaper::Wallpaper;
use crate::wallpaper::scene::{
    PostExtract,
    asset_server::{
        AssetDropEvent, AssetHandle, AssetId, AssetIdGenerator, AssetServer, UntypedAssetHandle,
    },
    extract::Extract,
    plugin::{AddPlugins, Plugin},
    render::SceneExtract,
};
use bevy_ecs::{
    prelude::*,
    system::{StaticSystemParam, SystemParam, SystemParamItem},
};
use crossbeam::channel::{Receiver, Sender};
use smallvec::SmallVec;
use std::{collections::HashMap, marker::PhantomData};

/// Collection of assets of a specific type.
///
/// Assets are stored with unique IDs and can be accessed by handle.
#[derive(Resource)]
pub struct Assets<A: Asset> {
    map: HashMap<AssetId, A>,
    new_ids: SmallVec<[AssetId; 4]>,
    drop_receiver: Receiver<AssetDropEvent>,
    drop_sender: Sender<AssetDropEvent>,
    id_generator: AssetIdGenerator,
}

impl<A: Asset> Assets<A> {
    pub fn new(
        drop_receiver: Receiver<AssetDropEvent>,
        drop_sender: Sender<AssetDropEvent>,
        id_generator: AssetIdGenerator,
    ) -> Self {
        Self {
            map: HashMap::new(),
            new_ids: SmallVec::new(),
            drop_receiver,
            drop_sender,
            id_generator,
        }
    }

    fn next_handle(&self) -> AssetHandle<A> {
        let id = self.id_generator.next_id();
        AssetHandle::new(UntypedAssetHandle::new(id, self.drop_sender.clone()))
    }

    /// Add an asset to the collection and return a handle to it.
    pub fn add(&mut self, asset: A) -> AssetHandle<A> {
        let handle = self.next_handle();
        self.map.insert(handle.id(), asset);
        self.new_ids.push(handle.id());
        handle
    }

    /// Get a reference to an asset by handle.
    pub fn get(&self, id: AssetId) -> Option<&A> {
        self.map.get(&id)
    }

    /// Get a mutable reference to an asset by handle.
    pub fn get_mut(&mut self, id: AssetId) -> Option<&mut A> {
        self.map.get_mut(&id)
    }

    /// Iterate over newly added assets.
    ///
    /// This is used during extraction to transfer new assets to the render world.
    pub fn new_assets(&self) -> impl ExactSizeIterator<Item = (AssetId, &A)> + '_ {
        self.new_ids.iter().map(|&id| (id, &self.map[&id]))
    }

    /// Clear the list of new assets.
    ///
    /// This should be called after extracting new assets to the render world.
    pub fn flush(&mut self) {
        self.new_ids.clear();
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
        while let Ok(event) = self.drop_receiver.try_recv() {
            _ = self.map.remove(&event.0);
        }
    }
}

impl<A: Asset> FromWorld for Assets<A> {
    fn from_world(world: &mut World) -> Self {
        world.resource::<AssetServer>().make_assets()
    }
}

/// Trait for types that can be used as assets.
///
/// Assets must be sendable between threads and have a static lifetime.
pub trait Asset: Send + Sync + 'static {}

/// Collection of GPU-ready assets.
#[derive(Resource)]
pub struct RenderAssets<A: RenderAsset> {
    map: HashMap<AssetId, A>,
}

impl<A: RenderAsset> RenderAssets<A> {
    /// Create a new empty render asset collection.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Add a render asset.
    pub fn add(&mut self, id: AssetId, asset: A) {
        _ = self.map.insert(id, asset);
    }

    /// Remove a render asset.
    pub fn remove(&mut self, id: AssetId) -> Option<A> {
        self.map.remove(&id)
    }

    /// Get a reference to a render asset by handle.
    pub fn get(&self, id: AssetId) -> Option<&A> {
        self.map.get(&id)
    }
}

impl<A: RenderAsset> Default for RenderAssets<A> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for assets that have GPU-ready counterparts.
pub trait RenderAsset: Send + Sync + 'static {
    /// The source asset type.
    type Asset: Asset;
    /// System parameters needed for extraction.
    type Param: SystemParam + 'static;

    /// Extract a render asset from a source asset.
    fn extract(source: &Self::Asset, item: &mut SystemParamItem<'_, '_, Self::Param>) -> Self;
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
    for (id, asset) in assets.new_assets() {
        let render_asset = A::extract(asset, &mut param);
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
        let render_asset = A::extract(asset, &mut param);
        render_assets.add(id, render_asset);
    }
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

        if self.do_extact_all {
            wallpaper
                .render
                .add_systems(SceneExtract, extract_all_render_assets::<A>);
        } else {
            wallpaper
                .render
                .add_systems(SceneExtract, extract_new_render_assets::<A>);
        }
    }
}
