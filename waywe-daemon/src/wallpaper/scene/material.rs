use super::wallpaper::Wallpaper;
use crate::wallpaper::scene::{
    assets::{Asset, AssetHandle, AssetId, AssetsPlugin},
    plugin::Plugin,
};
use bevy_ecs::{
    prelude::*,
    system::{SystemParam, SystemParamItem},
};
use std::{any::TypeId, collections::HashMap};

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(AssetsPlugin::<RenderMaterial>::new_render());
        wallpaper.render.init_resource::<MaterialAssetMap>();
    }
}

#[derive(Clone, Debug)]
pub struct VertexFragmentShader {
    pub vertex: wgpu::ShaderModule,
    pub fragment: wgpu::ShaderModule,
}

impl Asset for VertexFragmentShader {}

pub trait Material: Asset + AsBindGroup {
    fn create_shader(device: &wgpu::Device) -> VertexFragmentShader;
}

pub trait AsBindGroup {
    type Param: SystemParam + 'static;

    const LABEL: Option<&'static str> = None;

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;

    fn create_bind_group(
        &self,
        param: &mut SystemParamItem<'_, '_, Self::Param>,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup;
}

#[derive(Clone, Debug)]
pub struct RenderMaterial {
    pub shader: VertexFragmentShader,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Asset for RenderMaterial {}

#[derive(Component, Clone, Copy, Debug)]
pub struct RenderMaterialHandle(pub AssetHandle<RenderMaterial>);

#[derive(Resource, Default)]
pub struct MaterialAssetMap(pub HashMap<(TypeId, AssetId), AssetHandle<RenderMaterial>>);

impl MaterialAssetMap {
    pub fn set<M: Material>(
        &mut self,
        handle: AssetHandle<M>,
        render_handle: AssetHandle<RenderMaterial>,
    ) {
        _ = self.0.insert((TypeId::of::<M>(), handle.id), render_handle);
    }

    pub fn get<M: Material>(&self, handle: AssetHandle<M>) -> Option<AssetHandle<RenderMaterial>> {
        self.0.get(&(TypeId::of::<M>(), handle.id)).copied()
    }
}
