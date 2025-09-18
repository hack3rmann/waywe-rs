use super::render::SceneRenderer;
use crate::wallpaper::scene::{
    assets::{Asset, AssetHandle, AssetsPlugin, RenderAsset},
    render::RenderPlugin,
};
use bevy_ecs::{
    component::Component,
    system::{SystemParam, SystemParamItem},
};

pub struct MaterialPlugin;

impl RenderPlugin for MaterialPlugin {
    fn init(self, renderer: &mut SceneRenderer) {
        renderer.add_plugin(AssetsPlugin::<RenderMaterial>::new());
    }
}

pub struct VertexFragmentShader {
    pub vertex: wgpu::ShaderModule,
    pub fragment: wgpu::ShaderModule,
}

pub trait Material: Asset + AsBindGroup {
    type RenderAsset: RenderAsset<Asset = Self>;

    fn create_shader(device: &wgpu::Device) -> VertexFragmentShader;
}

pub trait AsBindGroup {
    type Param: SystemParam;

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
    pub bind_group: wgpu::BindGroup,
}

impl Asset for RenderMaterial {}

#[derive(Component, Clone, Copy, Debug)]
pub struct RenderMaterialHandle(pub AssetHandle<RenderMaterial>);
