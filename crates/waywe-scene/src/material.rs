//! Material system for defining surface appearance.
//!
//! This module provides a material system for defining how surfaces
//! should be rendered, including shaders and bind groups.
//!
//! # Core Types
//!
//! - [`Material`]: Trait for defining custom materials
//! - [`AsBindGroup`]: Trait for creating GPU bind groups
//! - [`RenderMaterial`]: GPU-ready material
//! - [`VertexFragmentShader`]: Shaders for rendering
//!
//! # Plugins
//!
//! - [`MaterialPlugin`]: Adds material functionality to a wallpaper

use crate::{
    asset_server::AssetId,
    assets::{Asset, RefAssetsPlugin},
    plugin::Plugin,
    wallpaper::Wallpaper,
};
use waywe_ecs::{
    prelude::*,
    system::{SystemParam, SystemParamItem},
};
use waywe_runtime::{gpu::Wgpu, shaders::ShaderDescriptor};
use waywe_uuid::TypeUuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord, Hash, SystemSet)]
pub enum MaterialSet {
    #[default]
    ExtractRender,
}

/// Plugin for material functionality.
///
/// Adds systems and resources for managing materials.
pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(RefAssetsPlugin::<RenderMaterial>::new());
    }
}

/// Vertex and fragment shaders for rendering.
#[derive(Clone, Debug, TypeUuid)]
#[uuid = "2c519aba-5567-4b85-9d5c-123f807355e1"]
pub struct VertexFragmentShader {
    /// Vertex shader module.
    pub vertex: wgpu::ShaderModule,
    /// Fragment shader module.
    pub fragment: wgpu::ShaderModule,
}

impl Asset for VertexFragmentShader {}

/// Trait for defining custom materials.
///
/// Materials define how surfaces should be rendered, including shaders
/// and bind groups.
pub trait Material: Asset + AsBindGroup {
    type VertexShader: ShaderDescriptor;
    type FragmentShader: ShaderDescriptor;

    fn create_shader(gpu: &Wgpu) -> VertexFragmentShader {
        gpu.require_shader::<Self::VertexShader>();
        gpu.require_shader::<Self::FragmentShader>();

        VertexFragmentShader {
            vertex: gpu
                .shader_cache
                .get::<Self::VertexShader>()
                .unwrap()
                .clone(),
            fragment: gpu
                .shader_cache
                .get::<Self::FragmentShader>()
                .unwrap()
                .clone(),
        }
    }
}

/// Trait for creating GPU bind groups.
///
/// Bind groups contain the resources (textures, samplers, etc.) that
/// shaders need to render a material.
pub trait AsBindGroup {
    /// System parameters needed for creating bind groups.
    type Param: SystemParam + 'static;

    /// Label for debugging.
    const LABEL: Option<&'static str> = None;

    /// Create the bind group layout for this material.
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;

    /// Create a bind group for this material.
    fn create_bind_group(
        &self,
        param: &mut SystemParamItem<'_, '_, Self::Param>,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup;
}

/// GPU-ready material.
#[derive(Clone, Debug, TypeUuid)]
#[uuid = "c208df8d-d65c-4668-b181-d40a5ae84dfc"]
pub struct RenderMaterial {
    /// Shaders for rendering.
    pub shader: VertexFragmentShader,
    /// Layout for the bind group.
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// Bind group containing resources.
    pub bind_group: wgpu::BindGroup,
}

impl Asset for RenderMaterial {}

/// Handle to a render material component.
#[derive(Component, TypeUuid, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[uuid = "26aa06f1-4bf6-4f54-859b-7553e12adc03"]
pub struct RenderMaterialId(pub AssetId);
