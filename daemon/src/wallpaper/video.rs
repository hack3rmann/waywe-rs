use super::Wallpaper;
use crate::{
    app::VideoAppEvent,
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures, gpu::Wgpu},
    video_pipeline::VideoPipeline,
};
use ash::vk;
use glam::UVec2;
use std::{ffi::CStr, os::fd::IntoRawFd, ptr, time::Duration};
use thiserror::Error;
use tracing::error;
use video::{
    AudioVideoFormat, BackendError, Codec, CodecContext, FormatContext, Frame, FrameDuration,
    MediaType, Packet, RatioI32, VideoPixelFormat,
};
use wgpu::hal::api;

pub struct VideoWallpaper {
    pub do_loop_video: bool,
    pub pipeline: VideoPipeline,
    pub format_context: FormatContext,
    pub time_base: RatioI32,
    pub best_stream_index: usize,
    pub codec_context: CodecContext,
    pub frame_time_fallback: Duration,
    pub packet: Option<Packet>,
    pub frame: Frame,
}

impl VideoWallpaper {
    pub fn new(
        gpu: &Wgpu,
        monitor_size: UVec2,
        path: &CStr,
        monitor_index: usize,
    ) -> Result<Self, VideoWallpaperCreationError> {
        let format_context = FormatContext::from_input(path)?;
        let best_stream = format_context.find_best_stream(MediaType::Video)?;

        let time_base = best_stream.time_base();
        let best_stream_index = best_stream.index();
        let codec_parameters = best_stream.codec_parameters();
        let frame_rate = codec_parameters.frame_rate().unwrap();

        if !matches!(
            codec_parameters.format(),
            Some(video::AudioVideoFormat::Video(VideoPixelFormat::Yuv420p))
        ) {
            return Err(VideoWallpaperCreationError::UnsupportedFormat(
                codec_parameters.format(),
            ));
        }

        let Some(decoder) = Codec::find_decoder_for_id(codec_parameters.codec_id()) else {
            return Err(VideoWallpaperCreationError::VideoBackend(
                BackendError::DECODER_NOT_FOUND,
            ));
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
            pipeline: VideoPipeline::new(gpu, monitor_size, monitor_index),
            format_context,
            time_base,
            best_stream_index,
            codec_context,
            frame_time_fallback,
            do_loop_video: true,
            packet: None,
            frame: Frame::new(),
        })
    }
}

impl Wallpaper for VideoWallpaper {
    fn required_features() -> RuntimeFeatures
    where
        Self: Sized,
    {
        RuntimeFeatures::GPU | RuntimeFeatures::VIDEO
    }

    fn frame(
        &mut self,
        runtime: &Runtime<VideoAppEvent>,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError> {
        loop {
            if self.packet.is_none() {
                let packet = match self.format_context.read_packet(self.best_stream_index) {
                    Ok(packet) => packet,
                    Err(BackendError::EOF) => {
                        if !self.do_loop_video {
                            return Err(FrameError::StopRequested);
                        }

                        let best_index = self.best_stream_index;

                        if let Err(error) = self.format_context.repeat_stream(best_index) {
                            error!(?error, "failed to reapead video stream");
                            return Err(FrameError::Skip);
                        }

                        continue;
                    }
                    Err(error) => {
                        error!(?error, "failed to read next video packet");
                        return Err(FrameError::Skip);
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
                    error!(?error, "failed to receive frame from the decoder");
                    return Err(FrameError::Skip);
                }
            }
        }

        let Some(va_display) = self.codec_context.va_display() else {
            panic!("failed to retrieve libva display");
        };

        let surface_id = unsafe { self.frame.surface_id() };

        if let Err(error) = va_display.sync_surface(surface_id) {
            panic!("failed to sync libva surface: {error:?}");
        }

        let surface_handle = match va_display.export_surface_handle(surface_id) {
            Ok(handle) => handle,
            Err(error) => panic!("failed to export surface handle: {error:?}"),
        };

        let dma_desc = *surface_handle.desc();
        let dma_buf_fd = surface_handle.into_fd().into_raw_fd();

        let memory_properties = unsafe {
            runtime.wgpu.adapter.as_hal::<api::Vulkan, _, _>(|adapter| {
                let Some(adapter) = adapter else {
                    unreachable!()
                };

                let raw_instance = adapter.shared_instance().raw_instance();

                let memory_properties = raw_instance
                    .get_physical_device_memory_properties(adapter.raw_physical_device());

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

                raw_instance
                    .get_physical_device_image_format_properties2(
                        adapter.raw_physical_device(),
                        &format_info,
                        &mut format_properties,
                    )
                    .unwrap();

                assert!(
                    ext_properties
                        .external_memory_properties
                        .external_memory_features
                        .contains(vk::ExternalMemoryFeatureFlags::IMPORTABLE)
                );

                memory_properties
            })
        };

        let texture_hal = unsafe {
            runtime
                .wgpu
                .device
                .as_hal::<api::Vulkan, _, _>(move |device| {
                    let device = device.unwrap();
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
                        s_type:
                            vk::StructureType::IMAGE_DRM_FORMAT_MODIFIER_EXPLICIT_CREATE_INFO_EXT,
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

                    let vk_image = vk_device.create_image(&image_info, None).unwrap();
                    let memory_requirements = vk_device.get_image_memory_requirements(vk_image);

                    let memory_type_index = memory_properties.memory_types
                        [..memory_properties.memory_type_count as usize]
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

                    let device_memory = vk_device.allocate_memory(&alloc_info, None).unwrap();
                    vk_device
                        .bind_image_memory(vk_image, device_memory, 0)
                        .unwrap();

                    wgpu::hal::vulkan::Device::texture_from_raw(
                        vk_image,
                        &wgpu::hal::TextureDescriptor {
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
                            usage: wgpu::hal::TextureUses::RESOURCE,
                            memory_flags: wgpu::hal::MemoryFlags::PREFER_COHERENT,
                            view_formats: vec![],
                        },
                        Some(Box::new(move || {
                            // NOTE(hack3rmann): we have to manually destroy the image
                            // because wgpu does not do this due creation of drop callback
                            vk_destroy_image(vk_device_raw, vk_image, ptr::null());
                            // NOTE(hack3rmann): we have to manually deallocate the memory
                            // because wgpu does not do this due to call to `texture_from_raw`
                            vk_free_memory(vk_device_raw, device_memory, ptr::null());
                        })),
                    )
                })
        };

        let texture = unsafe {
            runtime.wgpu.device.create_texture_from_hal::<api::Vulkan>(
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
        };

        let texture_y_plane = texture.create_view(&wgpu::TextureViewDescriptor {
            aspect: wgpu::TextureAspect::Plane0,
            ..Default::default()
        });

        let texture_uv_plane = texture.create_view(&wgpu::TextureViewDescriptor {
            aspect: wgpu::TextureAspect::Plane1,
            ..Default::default()
        });

        let bind_group = runtime
            .wgpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("video-bind-group"),
                layout: &self.pipeline.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_y_plane),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_uv_plane),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.pipeline.sampler),
                    },
                ],
            });

        let target_frame_time = self
            .frame
            .duration_in(self.time_base)
            .map(FrameDuration::to_duration)
            .unwrap_or(self.frame_time_fallback);

        self.pipeline.render(encoder, surface_view, &bind_group);

        Ok(FrameInfo {
            target_frame_time: Some(target_frame_time),
        })
    }
}

#[derive(Debug, Error)]
pub enum VideoWallpaperCreationError {
    #[error(transparent)]
    VideoBackend(#[from] BackendError),
    #[error("unsupported format {0:?}")]
    UnsupportedFormat(Option<AudioVideoFormat>),
}
