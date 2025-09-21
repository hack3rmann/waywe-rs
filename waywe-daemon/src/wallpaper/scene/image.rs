//! Image loading and rendering components.
//!
//! This module provides components and systems for loading and displaying
//! images in wallpapers.
//!
//! # Components
//!
//! - [`Image`]: Raw image data
//! - [`ImageMaterial`]: Material that displays an image
//! - [`RenderImage`]: GPU-ready image data
//!
//! # Plugins
//!
//! - [`ImagePlugin`]: Adds image functionality to a wallpaper

use super::wallpaper::Wallpaper;
use crate::{
    runtime::gpu::Wgpu,
    wallpaper::scene::{
        assets::{
            Asset, AssetHandle, AssetId, Assets, AssetsPlugin, RenderAsset, RenderAssets,
            RenderAssetsPlugin, extract_new_render_assets,
        },
        extract::Extract,
        material::{AsBindGroup, Material, MaterialAssetMap, RenderMaterial, VertexFragmentShader},
        plugin::Plugin,
        render::{RenderGpu, SceneExtract},
    },
};
use bevy_ecs::{
    prelude::*,
    system::{StaticSystemParam, SystemParamItem, lifetimeless::SRes},
};
use derive_more::{Deref, DerefMut};
use wgpu::util::DeviceExt;

/// Handle to the default 1x1 white image.
pub const DEFAULT_IMAGE: AssetHandle<Image> = AssetHandle::new(AssetId::new(1));
/// Handle to the default image material.
pub const DEFAULT_IMAGE_MATERIAL: AssetHandle<ImageMaterial> = AssetHandle::new(AssetId::new(1));

/// Plugin for image functionality.
///
/// Adds systems and resources for loading and displaying images.
pub struct ImagePlugin;

impl Plugin for ImagePlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins((
            AssetsPlugin::<Image>::new(),
            AssetsPlugin::<ImageMaterial>::new(),
            RenderAssetsPlugin::<RenderImage>::extract_new(),
        ));

        let mut image_assets = wallpaper.main.resource_mut::<Assets<Image>>();
        let default_image = image_assets.add(Image::new_white_1x1());
        debug_assert_eq!(default_image, DEFAULT_IMAGE);

        let mut material_assets = wallpaper.main.resource_mut::<Assets<ImageMaterial>>();
        let default_material = material_assets.add(ImageMaterial {
            image: default_image,
        });
        debug_assert_eq!(default_material, DEFAULT_IMAGE_MATERIAL);

        wallpaper
            .render
            .add_systems(
                SceneExtract,
                extract_image_materials.after(extract_new_render_assets::<RenderImage>),
            )
            .init_resource::<ImagePipeline>();
    }
}

/// Image asset containing raw pixel data.
#[derive(Debug, Deref, DerefMut)]
pub struct Image {
    /// The underlying image data.
    pub image: image::RgbaImage,
}

impl Image {
    /// Create a new 1x1 white image.
    pub fn new_white_1x1() -> Self {
        let mut image = image::RgbaImage::new(1, 1);
        image.get_pixel_mut(0, 0).0 = [255; 4];

        Self { image }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self::new_white_1x1()
    }
}

/// System to extract image materials for rendering.
///
/// Converts [`ImageMaterial`] assets into GPU-ready [`RenderMaterial`] assets.
pub fn extract_image_materials(
    materials: Extract<Res<Assets<ImageMaterial>>>,
    mut render_materials: ResMut<Assets<RenderMaterial>>,
    image_params: StaticSystemParam<<ImageMaterial as AsBindGroup>::Param>,
    mut handle_map: ResMut<MaterialAssetMap>,
    pipeline: Res<ImagePipeline>,
    gpu: Res<RenderGpu>,
) {
    let mut image_params = image_params.into_inner();

    for (id, material) in materials.new_assets() {
        let bind_group_layout = ImageMaterial::bind_group_layout(&gpu.device);
        let bind_group =
            material.create_bind_group(&mut image_params, &gpu.device, &bind_group_layout);

        let render_id = render_materials.add(RenderMaterial {
            bind_group_layout,
            bind_group,
            shader: pipeline.shader.clone(),
        });

        handle_map.set(id, render_id);
    }
}

impl Asset for Image {}

/// GPU pipeline for rendering images.
#[derive(Resource)]
pub struct ImagePipeline {
    /// Sampler for texture filtering.
    pub sampler: wgpu::Sampler,
    /// Shaders for vertex and fragment processing.
    pub shader: VertexFragmentShader,
}

impl ImagePipeline {
    /// Create a new image pipeline.
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("image-material"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let shader = ImageMaterial::create_shader(device);

        Self { sampler, shader }
    }
}

impl FromWorld for ImagePipeline {
    fn from_world(world: &mut World) -> Self {
        let gpu = world.resource::<RenderGpu>();
        Self::new(&gpu.device)
    }
}

/// Material that displays an image.
pub struct ImageMaterial {
    /// The image to display.
    pub image: AssetHandle<Image>,
}

impl Asset for ImageMaterial {}

impl Material for ImageMaterial {
    fn create_shader(device: &wgpu::Device) -> VertexFragmentShader {
        let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../shaders/scene-image-vertex.glsl").into(),
                stage: wgpu::naga::ShaderStage::Vertex,
                defines: Default::default(),
            },
        });

        let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../shaders/scene-image-fragment.glsl").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: Default::default(),
            },
        });

        VertexFragmentShader { vertex, fragment }
    }
}

/// GPU-ready image data.
pub struct RenderImage {
    /// The GPU texture.
    pub texture: wgpu::Texture,
    /// A view of the texture for rendering.
    pub view: wgpu::TextureView,
}

impl RenderImage {
    /// Create a new render image from image data.
    pub fn new(image: &Image, gpu: &Wgpu) -> Self {
        let texture = gpu.device.create_texture_with_data(
            &gpu.queue,
            &wgpu::TextureDescriptor {
                label: ImageMaterial::LABEL,
                size: wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            image,
        );

        let view = texture.create_view(&Default::default());

        Self { texture, view }
    }
}

impl RenderAsset for RenderImage {
    type Asset = Image;
    type Param = SRes<RenderGpu>;

    fn extract(image: &Self::Asset, gpu: &mut SystemParamItem<'_, '_, Self::Param>) -> Self {
        Self::new(image, gpu)
    }
}

impl AsBindGroup for ImageMaterial {
    type Param = (SRes<ImagePipeline>, SRes<RenderAssets<RenderImage>>);

    const LABEL: Option<&'static str> = Some("image-material");

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Self::LABEL,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        // TODO(hack3rmann): config fiter mode
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    fn create_bind_group(
        &self,
        (pipeline, image_assets): &mut SystemParamItem<'_, '_, Self::Param>,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let view = &image_assets.get(self.image).unwrap().view;

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Self::LABEL,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                },
            ],
        })
    }
}
