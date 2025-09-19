use super::{Scene, render::SceneRenderer};
use crate::{
    runtime::gpu::Wgpu,
    wallpaper::scene::{
        ScenePlugin, SceneUpdate, Time,
        assets::{
            Asset, AssetHandle, Assets, AssetsPlugin, RenderAsset, RenderAssets, RenderAssetsPlugin,
        },
        material::{AsBindGroup, Material, MaterialAssetMap, RenderMaterial, VertexFragmentShader},
        render::{Extract, RenderGpu, RenderPlugin, SceneExtract},
    },
};
use ash::vk::{self, PhysicalDeviceMemoryProperties};
use bevy_ecs::{
    prelude::*,
    system::{StaticSystemParam, SystemParamItem, lifetimeless::SRes},
};
use glam::UVec2;
use std::{ffi::CString, os::fd::IntoRawFd as _, ptr, time::Duration};
use video::{
    BackendError, Codec, CodecContext, FormatContext, Frame, MediaType, Packet, RatioI32,
    VideoPixelFormat, acceleration::VaSurfaceHandle,
};
use wgpu::wgc::api;

pub struct VideoPlugin;

impl ScenePlugin for VideoPlugin {
    fn init(self, scene: &mut Scene) {
        scene.add_plugin(AssetsPlugin::<Video>::new());
        scene.add_plugin(AssetsPlugin::<VideoMaterial>::new());
        scene.add_systems(SceneUpdate, advance_videos);
    }
}

impl RenderPlugin for VideoPlugin {
    fn init(self, renderer: &mut SceneRenderer) {
        renderer.add_plugin(RenderAssetsPlugin::<RenderVideo>::extract_all());
        renderer.world.init_resource::<VideoPipeline>();
        renderer.add_systems(SceneExtract, extract_video_materials);
    }
}

pub fn advance_videos(mut videos: ResMut<Assets<Video>>, time: Res<Time>) {
    for (_id, video) in videos.iter_mut() {
        let Some(duration) = video.frame.duration_in(video.time_base) else {
            video.next_frame();
            continue;
        };
        let duration = duration.to_duration();

        if video.update_delay + time.delta >= duration {
            video.next_frame();
            video.update_delay = video.update_delay + time.delta - duration;
        } else {
            video.update_delay += time.delta;
        }
    }
}

#[derive(Debug)]
pub struct Video {
    pub path: CString,
    pub format_context: FormatContext,
    pub codec_context: CodecContext,
    pub time_base: RatioI32,
    pub best_stream_index: usize,
    pub frame_time_fallback: Duration,
    pub packet: Option<Packet>,
    pub frame: Frame,
    pub do_loop_video: bool,
    pub update_delay: Duration,
}

impl Asset for Video {}

impl Video {
    pub fn new(path: impl Into<CString>) -> Result<Self, BackendError> {
        let path = path.into();
        let format_context = FormatContext::from_input(&path)?;
        let best_stream = format_context.find_best_stream(MediaType::Video)?;

        let time_base = best_stream.time_base();
        let best_stream_index = best_stream.index();
        let codec_parameters = best_stream.codec_parameters();
        let frame_rate = codec_parameters.frame_rate().unwrap();

        if !matches!(
            codec_parameters.format(),
            Some(video::AudioVideoFormat::Video(VideoPixelFormat::Yuv420p))
        ) {
            // FIXME(hack3rmann): handle error
            panic!("invalid video format");
        }

        let Some(decoder) = Codec::find_decoder_for_id(codec_parameters.codec_id()) else {
            return Err(BackendError::DECODER_NOT_FOUND);
        };

        let mut codec_context =
            CodecContext::from_parameters_with_hw_accel(codec_parameters, Some(decoder))?;

        codec_context.open(decoder)?;

        const FRAME_DURATION_60_FPS: Duration = RatioI32::new(1, 60).unwrap().to_duration_seconds();

        let frame_time_fallback = match frame_rate.inv() {
            Some(duration) => duration.to_duration_seconds(),
            None => FRAME_DURATION_60_FPS,
        };

        Ok(Self {
            format_context,
            codec_context,
            time_base,
            best_stream_index,
            frame_time_fallback,
            packet: None,
            frame: Frame::new(),
            path,
            do_loop_video: true,
            update_delay: Duration::ZERO,
        })
    }

    pub fn frame_size(&self) -> UVec2 {
        self.format_context.streams()[self.best_stream_index]
            .codec_parameters()
            .video_size()
            .unwrap()
    }

    pub fn frame_aspect_ratio(&self) -> f32 {
        let size = self.frame_size();
        size.y as f32 / size.x as f32
    }

    pub fn next_frame(&mut self) {
        loop {
            if self.packet.is_none() {
                let packet = match self.format_context.read_packet(self.best_stream_index) {
                    Ok(packet) => packet,
                    Err(BackendError::EOF) => {
                        if !self.do_loop_video {
                            return;
                        }

                        let best_index = self.best_stream_index;

                        if let Err(error) = self.format_context.repeat_stream(best_index) {
                            panic!("failed to repeat video stream: {error}");
                        }

                        continue;
                    }
                    Err(error) => {
                        panic!("failed to read next video packet: {error}");
                    }
                };

                self.codec_context.send_packet(&packet).unwrap();

                _ = self.packet.insert(packet);
            }

            match self.codec_context.receive_frame(&mut self.frame) {
                Ok(()) => break,
                Err(BackendError::EAGAIN) => {
                    self.packet = None;
                    continue;
                }
                Err(error) => {
                    panic!("failed to receive frame from the decoder: {error}");
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RenderVideo {
    pub texture: wgpu::Texture,
    pub texture_y_plane: wgpu::TextureView,
    pub texture_uv_plane: wgpu::TextureView,
}

impl RenderVideo {
    pub fn get_memory_properties(adapter: &wgpu::Adapter) -> PhysicalDeviceMemoryProperties {
        let adapter = unsafe { adapter.as_hal::<api::Vulkan>().unwrap() };
        let raw_instance = adapter.shared_instance().raw_instance();

        let memory_properties = unsafe {
            raw_instance.get_physical_device_memory_properties(adapter.raw_physical_device())
        };

        let ext_format_info = vk::PhysicalDeviceExternalImageFormatInfo {
            s_type: vk::StructureType::PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO,
            p_next: ptr::null(),
            handle_type: vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD,
            _marker: std::marker::PhantomData,
        };

        let format_info = vk::PhysicalDeviceImageFormatInfo2 {
            s_type: vk::StructureType::PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2,
            p_next: (&raw const ext_format_info).cast(),
            format: vk::Format::G8_B8R8_2PLANE_420_UNORM,
            ty: vk::ImageType::TYPE_2D,
            tiling: vk::ImageTiling::LINEAR,
            usage: vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::empty(),
            _marker: std::marker::PhantomData,
        };

        let mut ext_properties = vk::ExternalImageFormatProperties {
            s_type: vk::StructureType::EXTERNAL_IMAGE_FORMAT_PROPERTIES,
            p_next: ptr::null_mut(),
            external_memory_properties: vk::ExternalMemoryProperties::default(),
            _marker: std::marker::PhantomData,
        };

        let mut format_properties = vk::ImageFormatProperties2 {
            s_type: vk::StructureType::IMAGE_FORMAT_PROPERTIES_2,
            p_next: (&raw mut ext_properties).cast(),
            image_format_properties: vk::ImageFormatProperties::default(),
            _marker: std::marker::PhantomData,
        };

        unsafe {
            raw_instance
                .get_physical_device_image_format_properties2(
                    adapter.raw_physical_device(),
                    &format_info,
                    &mut format_properties,
                )
                .unwrap()
        };

        assert!(
            ext_properties
                .external_memory_properties
                .external_memory_features
                .contains(vk::ExternalMemoryFeatureFlags::IMPORTABLE)
        );

        memory_properties
    }

    pub fn create_texture(gpu: &Wgpu, surface: VaSurfaceHandle) -> wgpu::Texture {
        let dma_desc = *surface.desc();
        let dma_buf_fd = surface.into_fd().into_raw_fd();

        let memory_properties = Self::get_memory_properties(&gpu.adapter);

        let device = unsafe { gpu.device.as_hal::<api::Vulkan>().unwrap() };
        let vk_device = device.raw_device();

        let vk_free_memory = vk_device.fp_v1_0().free_memory;
        let vk_destroy_image = vk_device.fp_v1_0().destroy_image;
        let vk_device_raw = vk_device.handle();

        let ext_info = vk::ExternalMemoryImageCreateInfo {
            s_type: vk::StructureType::EXTERNAL_MEMORY_IMAGE_CREATE_INFO,
            // TODO(hack3rmann): use `DMA_BUF_EXT` whenever it is possible
            // The reason is it has no restrictions on the device that was
            // used to decode a video
            handle_types: vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD,
            p_next: ptr::null(),
            _marker: std::marker::PhantomData,
        };

        let plane_layouts = [
            vk::SubresourceLayout {
                offset: dma_desc.layers[0].offset[0] as u64,
                size: 0,
                row_pitch: dma_desc.layers[0].pitch[0] as u64,
                array_pitch: 0,
                depth_pitch: 0,
            },
            vk::SubresourceLayout {
                offset: dma_desc.layers[1].offset[0] as u64,
                size: 0,
                row_pitch: dma_desc.layers[1].pitch[0] as u64,
                array_pitch: 0,
                depth_pitch: 0,
            },
        ];

        let formats = [vk::Format::R8_UNORM, vk::Format::R8G8_UNORM];

        let format_list_info = vk::ImageFormatListCreateInfo {
            s_type: vk::StructureType::IMAGE_FORMAT_LIST_CREATE_INFO,
            p_next: (&raw const ext_info).cast(),
            view_format_count: formats.len() as u32,
            p_view_formats: formats.as_ptr(),
            _marker: std::marker::PhantomData,
        };

        let drm_create_info = vk::ImageDrmFormatModifierExplicitCreateInfoEXT {
            s_type: vk::StructureType::IMAGE_DRM_FORMAT_MODIFIER_EXPLICIT_CREATE_INFO_EXT,
            p_next: (&raw const format_list_info).cast(),
            drm_format_modifier: dma_desc.objects[0].drm_format_modifier,
            drm_format_modifier_plane_count: dma_desc.num_layers,
            p_plane_layouts: plane_layouts.as_ptr(),
            _marker: std::marker::PhantomData,
        };

        let image_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            format: vk::Format::G8_B8R8_2PLANE_420_UNORM,
            usage: vk::ImageUsageFlags::SAMPLED,
            extent: vk::Extent3D {
                width: dma_desc.width,
                height: dma_desc.height,
                depth: 1,
            },
            p_next: (&raw const drm_create_info).cast(),
            image_type: vk::ImageType::TYPE_2D,
            flags: vk::ImageCreateFlags::MUTABLE_FORMAT,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            initial_layout: vk::ImageLayout::UNDEFINED,
            _marker: std::marker::PhantomData,
        };

        let vk_image = unsafe { vk_device.create_image(&image_info, None).unwrap() };
        let memory_requirements = unsafe { vk_device.get_image_memory_requirements(vk_image) };

        let memory_type_index = memory_properties
            .memory_types_as_slice()
            .iter()
            .enumerate()
            .find(|&(i, memory_type)| {
                memory_type
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                    && (memory_requirements.memory_type_bits & (1 << i as u32)) != 0
            })
            .map(|(i, _)| i as u32)
            .unwrap();

        let import_info = vk::ImportMemoryFdInfoKHR {
            s_type: vk::StructureType::IMPORT_MEMORY_FD_INFO_KHR,
            p_next: ptr::null(),
            handle_type: vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD,
            fd: dma_buf_fd,
            _marker: std::marker::PhantomData,
        };

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: (&raw const import_info).cast(),
            allocation_size: memory_requirements.size,
            memory_type_index,
            _marker: std::marker::PhantomData,
        };

        let device_memory = unsafe { vk_device.allocate_memory(&alloc_info, None).unwrap() };

        unsafe {
            vk_device
                .bind_image_memory(vk_image, device_memory, 0)
                .unwrap()
        };

        let texture_desc = wgpu::hal::TextureDescriptor {
            label: Some("video-texture"),
            size: wgpu::Extent3d {
                width: dma_desc.width,
                height: dma_desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::NV12,
            usage: wgpu::TextureUses::RESOURCE,
            memory_flags: wgpu::hal::MemoryFlags::PREFER_COHERENT,
            view_formats: vec![],
        };

        let destructor = Box::new(move || unsafe {
            // NOTE(hack3rmann): we have to manually destroy the image
            // because wgpu does not do this due creation of drop callback
            vk_destroy_image(vk_device_raw, vk_image, ptr::null());
            // NOTE(hack3rmann): we have to manually deallocate the memory
            // because wgpu does not do this due to call to `texture_from_raw`
            vk_free_memory(vk_device_raw, device_memory, ptr::null());
        });

        let texture_hal = unsafe {
            wgpu::hal::vulkan::Device::texture_from_raw(vk_image, &texture_desc, Some(destructor))
        };

        unsafe {
            gpu.device.create_texture_from_hal::<api::Vulkan>(
                texture_hal,
                &wgpu::TextureDescriptor {
                    label: Some("video-texture"),
                    size: wgpu::Extent3d {
                        width: dma_desc.width,
                        height: dma_desc.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::NV12,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
            )
        }
    }

    pub fn export_from(video: &Video, gpu: &Wgpu) -> Self {
        let Some(va_display) = video.codec_context.va_display() else {
            panic!("failed to retrieve libva display");
        };

        let surface_id = unsafe { video.frame.surface_id() };

        if let Err(error) = va_display.sync_surface(surface_id) {
            panic!("failed to sync libva surface: {error:?}");
        }

        let surface_handle = match va_display.export_surface_handle(surface_id) {
            Ok(handle) => handle,
            Err(error) => panic!("failed to export surface handle: {error:?}"),
        };

        let texture = Self::create_texture(gpu, surface_handle);

        let texture_y_plane = texture.create_view(&wgpu::TextureViewDescriptor {
            aspect: wgpu::TextureAspect::Plane0,
            ..Default::default()
        });

        let texture_uv_plane = texture.create_view(&wgpu::TextureViewDescriptor {
            aspect: wgpu::TextureAspect::Plane1,
            ..Default::default()
        });

        Self {
            texture,
            texture_y_plane,
            texture_uv_plane,
        }
    }
}

impl RenderAsset for RenderVideo {
    type Asset = Video;
    type Param = SRes<RenderGpu>;

    fn extract(video: &Self::Asset, gpu: &mut SystemParamItem<'_, '_, Self::Param>) -> Self {
        Self::export_from(video, gpu)
    }
}

#[derive(Resource)]
pub struct VideoPipeline {
    pub sampler: wgpu::Sampler,
    pub shader: VertexFragmentShader,
}

impl VideoPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("image-material"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let shader = VideoMaterial::create_shader(device);

        Self { sampler, shader }
    }
}

impl FromWorld for VideoPipeline {
    fn from_world(world: &mut World) -> Self {
        let gpu = world.resource::<RenderGpu>();
        Self::new(&gpu.device)
    }
}

pub struct VideoMaterial {
    pub video: AssetHandle<Video>,
}

impl Asset for VideoMaterial {}

impl Material for VideoMaterial {
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
                shader: include_str!("../../shaders/scene-video-fragment.glsl").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: Default::default(),
            },
        });

        VertexFragmentShader { vertex, fragment }
    }
}

impl AsBindGroup for VideoMaterial {
    type Param = (SRes<VideoPipeline>, SRes<RenderAssets<RenderVideo>>);

    const LABEL: Option<&'static str> = Some("video-material");

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Self::LABEL,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    fn create_bind_group(
        &self,
        (pipeline, assets): &mut SystemParamItem<'_, '_, Self::Param>,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let video = assets.get(self.video).unwrap();

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Self::LABEL,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&video.texture_y_plane),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&video.texture_uv_plane),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                },
            ],
        })
    }
}

pub fn extract_video_materials(
    materials: Extract<Res<Assets<VideoMaterial>>>,
    mut render_materials: ResMut<Assets<RenderMaterial>>,
    image_params: StaticSystemParam<<VideoMaterial as AsBindGroup>::Param>,
    mut handle_map: ResMut<MaterialAssetMap>,
    pipeline: Res<VideoPipeline>,
    gpu: Res<RenderGpu>,
) {
    let mut image_params = image_params.into_inner();

    for (id, material) in materials.iter() {
        let bind_group_layout = VideoMaterial::bind_group_layout(&gpu.device);
        let bind_group =
            material.create_bind_group(&mut image_params, &gpu.device, &bind_group_layout);

        let render_material = RenderMaterial {
            bind_group_layout,
            bind_group,
            shader: pipeline.shader.clone(),
        };

        if let Some(render_id) = handle_map.get(id)
            && let Some(stored_material) = render_materials.get_mut(render_id)
        {
            *stored_material = render_material;
        } else {
            let render_id = render_materials.add(render_material);
            handle_map.set(id, render_id);
        }
    }
}
