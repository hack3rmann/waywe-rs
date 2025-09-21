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

use super::wallpaper::Wallpaper;
use crate::wallpaper::scene::{
    asset_server::AssetId,
    assets::{Asset, RefAssets},
    plugin::Plugin,
};
use bevy_ecs::{
    prelude::*,
    system::{SystemParam, SystemParamItem},
};

/// Plugin for material functionality.
///
/// Adds systems and resources for managing materials.
pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper
            .render
            .init_resource::<RefAssets<RenderMaterial>>();
    }
}

/// Vertex and fragment shaders for rendering.
#[derive(Clone, Debug)]
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
    /// Create the shaders for this material.
    fn create_shader(device: &wgpu::Device) -> VertexFragmentShader;
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
#[derive(Clone, Debug)]
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
#[derive(Component, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderMaterialId(pub AssetId);
