pub mod api;
pub mod conversions;
pub mod ffi;

use crate::conversions::{
    map_ffi_label, map_texture_usages_to_vk, map_vk_dimension, map_vk_extent, map_vk_format,
    map_vk_image_usage, map_vk_image_usage_to_texture_usages, map_wgpu_dimension, map_wgpu_extent,
    map_wgpu_format, map_wgpu_label,
};
use abi_stable::std_types::{ROption, RStr};
use ash::{khr::external_memory_fd::Device as FdDevice, vk};
use std::{
    marker::PhantomData,
    os::fd::{FromRawFd, IntoRawFd, OwnedFd},
    ptr,
};
use wgpu::{hal, wgc::api::Vulkan};

unsafe fn find_memory_type_index(
    instance: &ash::Instance,
    phys_device: vk::PhysicalDevice,
    type_bits: u32,
    props: vk::MemoryPropertyFlags,
) -> Option<u32> {
    let mem_props = unsafe { instance.get_physical_device_memory_properties(phys_device) };
    (0..mem_props.memory_type_count).find(|&i| {
        type_bits & (1 << i) != 0
            && mem_props.memory_types[i as usize]
                .property_flags
                .contains(props)
    })
}

fn texture_export_fd(device: &wgpu::Device, texture: &wgpu::Texture) -> OwnedFd {
    let texture_hal = unsafe { texture.as_hal::<Vulkan>().unwrap() };
    let memory = unsafe { texture_hal.external_memory().unwrap() };

    let device_hal = unsafe { device.as_hal::<Vulkan>().unwrap() };
    let device_raw = device_hal.raw_device();
    let instance_raw = device_hal.shared_instance().raw_instance();

    let ext = FdDevice::new(instance_raw, device_raw);

    let info = vk::MemoryGetFdInfoKHR {
        s_type: vk::StructureType::MEMORY_GET_FD_INFO_KHR,
        p_next: ptr::null(),
        handle_type: vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
        memory,
        _marker: PhantomData,
    };

    // NOTE(hack3rmann): from Vulkan spec:
    //
    // ```doc
    // The file descriptor returned by vkGetMemoryFdKHR is owned by the application and
    // should be closed using the close system call once the application no longer needs it.
    // ```
    let raw_fd = unsafe { ext.get_memory_fd(&info).unwrap() };
    unsafe { OwnedFd::from_raw_fd(raw_fd) }
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct FfiTextureDescriptor<'s> {
    pub extent: vk::Extent3D,
    pub format: vk::Format,
    pub wgpu_format: wgpu::TextureFormat,
    pub mip_level_count: u32,
    pub label: ROption<RStr<'s>>,
    pub dimension: vk::ImageType,
    pub usage: vk::ImageUsageFlags,
}

impl<'s> FfiTextureDescriptor<'s> {
    pub fn wgpu_extent(&self) -> wgpu::Extent3d {
        map_vk_extent(self.extent)
    }

    pub fn wgpu_format(&self) -> wgpu::TextureFormat {
        self.wgpu_format
    }

    pub fn wgpu_label(&self) -> Option<&'s str> {
        map_ffi_label(self.label)
    }

    pub fn wgpu_dimension(&self) -> wgpu::TextureDimension {
        map_vk_dimension(self.dimension)
    }

    pub fn wgpu_uses(&self) -> wgpu::TextureUses {
        map_vk_image_usage(self.usage)
    }

    pub fn wgpu_usages(&self) -> wgpu::TextureUsages {
        map_vk_image_usage_to_texture_usages(self.usage)
    }
}

impl<'s> From<wgpu::TextureDescriptor<'s>> for FfiTextureDescriptor<'s> {
    fn from(value: wgpu::TextureDescriptor<'s>) -> Self {
        Self {
            extent: map_wgpu_extent(value.size),
            format: map_wgpu_format(value.format).unwrap(),
            wgpu_format: value.format,
            mip_level_count: value.mip_level_count,
            label: map_wgpu_label(value.label),
            dimension: map_wgpu_dimension(value.dimension),
            usage: map_texture_usages_to_vk(value.usage, value.format),
        }
    }
}

fn import_fd_as_texture(
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    fd: OwnedFd,
    desc: FfiTextureDescriptor,
) -> wgpu::Texture {
    let device_hal = unsafe { device.as_hal::<Vulkan>().unwrap() };
    let adapter_hal = unsafe { adapter.as_hal::<Vulkan>().unwrap() };

    let vk_device = device_hal.raw_device();
    let vk_instance = device_hal.shared_instance().raw_instance();
    let phd = adapter_hal.raw_physical_device();

    let image_info_ext = vk::ExternalMemoryImageCreateInfo {
        s_type: vk::StructureType::EXTERNAL_MEMORY_IMAGE_CREATE_INFO,
        p_next: ptr::null(),
        handle_types: vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
        _marker: PhantomData,
    };

    let image_info = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        p_next: (&raw const image_info_ext).cast(),
        flags: vk::ImageCreateFlags::empty(),
        image_type: desc.dimension,
        format: desc.format,
        extent: desc.extent,
        mip_levels: desc.mip_level_count,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage: desc.usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
        initial_layout: vk::ImageLayout::UNDEFINED,
        _marker: PhantomData,
    };

    let vk_image = unsafe { vk_device.create_image(&image_info, None).unwrap() };
    let requirements = unsafe { vk_device.get_image_memory_requirements(vk_image) };

    let dedicated_info = vk::MemoryDedicatedAllocateInfo {
        s_type: vk::StructureType::MEMORY_DEDICATED_ALLOCATE_INFO,
        p_next: ptr::null(),
        image: vk_image,
        buffer: vk::Buffer::null(),
        _marker: PhantomData,
    };

    let import_info = vk::ImportMemoryFdInfoKHR {
        s_type: vk::StructureType::IMPORT_MEMORY_FD_INFO_KHR,
        p_next: (&raw const dedicated_info).cast(),
        handle_type: vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
        fd: fd.into_raw_fd(),
        _marker: PhantomData,
    };

    let mem_type_index = unsafe {
        find_memory_type_index(
            vk_instance,
            phd,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .unwrap()
    };

    let alloc_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: (&raw const import_info).cast(),
        allocation_size: requirements.size,
        memory_type_index: mem_type_index,
        _marker: PhantomData,
    };

    let memory = unsafe { vk_device.allocate_memory(&alloc_info, None).unwrap() };
    unsafe { vk_device.bind_image_memory(vk_image, memory, 0).unwrap() };

    let hal_desc = hal::TextureDescriptor {
        label: desc.wgpu_label(),
        size: desc.wgpu_extent(),
        mip_level_count: desc.mip_level_count,
        sample_count: 1,
        dimension: desc.wgpu_dimension(),
        format: desc.wgpu_format(),
        usage: desc.wgpu_uses(),
        memory_flags: hal::MemoryFlags::empty(),
        view_formats: vec![],
    };

    let device_hal = unsafe { device.as_hal::<Vulkan>().unwrap() };
    let texture_hal = unsafe { device_hal.texture_from_raw(vk_image, &hal_desc, None) };

    let wgpu_desc = wgpu::TextureDescriptor {
        label: desc.wgpu_label(),
        size: desc.wgpu_extent(),
        mip_level_count: desc.mip_level_count,
        sample_count: 1,
        dimension: desc.wgpu_dimension(),
        format: desc.wgpu_format(),
        usage: desc.wgpu_usages(),
        view_formats: &[],
    };

    unsafe { device.create_texture_from_hal::<hal::api::Vulkan>(texture_hal, &wgpu_desc) }
}

pub trait Sealed {}
impl Sealed for wgpu::Device {}

pub trait DeviceExt: Sealed {
    fn export_fd(&self, texture: &wgpu::Texture) -> OwnedFd;
}

impl DeviceExt for wgpu::Device {
    fn export_fd(&self, texture: &wgpu::Texture) -> OwnedFd {
        texture_export_fd(self, texture)
    }
}
