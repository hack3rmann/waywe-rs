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
    asset_server::{AssetHandle, AssetServerLoadPlugin, Load},
    assets::{
        Asset, Assets, AssetsExtract, AssetsPlugin, RefAssets, RefAssetsDependencyPlugin,
        RenderAsset, RenderAssets, RenderAssetsPlugin,
    },
    extract::Extract,
    material::{AsBindGroup, Material, MaterialSet, RenderMaterial, VertexFragmentShader},
    plugin::Plugin,
    render::{RenderGpu, SceneExtract},
};
use bevy_ecs::{
    prelude::*,
    system::{StaticSystemParam, SystemParamItem, lifetimeless::SRes},
};
use derive_more::{Deref, DerefMut};
use std::path::Path;
use waywe_runtime::{gpu::Wgpu, shaders::ShaderDescriptor};
use wgpu::util::DeviceExt;

/// Plugin for image functionality.
///
/// Adds systems and resources for loading and displaying images.
pub struct ImagePlugin;

impl Plugin for ImagePlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins((
            AssetsPlugin::<Image>::new(),
            AssetServerLoadPlugin::<Image>::new(),
            AssetsPlugin::<ImageMaterial>::new(),
            RenderAssetsPlugin::<RenderImage>::extract_new(),
            RefAssetsDependencyPlugin::<RenderMaterial, ImageMaterial>::new(),
        ));

        wallpaper
            .render
            .add_systems(
                SceneExtract,
                extract_image_materials
                    .in_set(MaterialSet::ExtractRender)
                    .after(AssetsExtract::MainToRender),
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

impl Load for Image {
    fn load(path: &Path) -> Self
    where
        Self: Sized,
    {
        let image = ::image::ImageReader::open(path)
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();

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
    mut render_materials: ResMut<RefAssets<RenderMaterial>>,
    image_params: StaticSystemParam<<ImageMaterial as AsBindGroup>::Param>,
    pipeline: Res<ImagePipeline>,
    gpu: Res<RenderGpu>,
) {
    let mut image_params = image_params.into_inner();

    for (id, material) in materials.new_assets() {
        let bind_group_layout = ImageMaterial::bind_group_layout(&gpu.device);
        let bind_group =
            material.create_bind_group(&mut image_params, &gpu.device, &bind_group_layout);

        render_materials.insert(
            id,
            RenderMaterial {
                bind_group_layout,
                bind_group,
                shader: pipeline.shader.clone(),
            },
        );
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
    pub fn new(gpu: &Wgpu) -> Self {
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("image-material"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let shader = ImageMaterial::create_shader(gpu);

        Self { sampler, shader }
    }
}

impl FromWorld for ImagePipeline {
    fn from_world(world: &mut World) -> Self {
        let gpu = world.resource::<RenderGpu>();
        Self::new(gpu)
    }
}

pub struct SceneImageVertexShader;

impl ShaderDescriptor for SceneImageVertexShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("shaders/scene-image-vertex.glsl").into(),
                stage: wgpu::naga::ShaderStage::Vertex,
                defines: Default::default(),
            },
        }
    }
}

pub struct SceneImageFragmentShader;

impl ShaderDescriptor for SceneImageFragmentShader {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("shaders/scene-image-fragment.glsl").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: Default::default(),
            },
        }
    }
}

/// Material that displays an image.
pub struct ImageMaterial {
    /// The image to display.
    pub image: AssetHandle<Image>,
}

impl Asset for ImageMaterial {}

impl Material for ImageMaterial {
    type VertexShader = SceneImageVertexShader;
    type FragmentShader = SceneImageFragmentShader;
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
        let view = &image_assets.get(self.image.id()).unwrap().view;

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
