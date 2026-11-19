use ash::vk;
use std::{marker::PhantomData, sync::Arc};
use waywe_rendering_api::{
    FfiTextureDescriptor, conversions::map_wgpu_format, import_fd_as_texture, texture_export_fd,
};
use waywe_runtime::{
    gpu::Wgpu,
    testing::{
        DEFAULT_FORMAT, DEFAULT_SIZE, Rect, assert_avg_color, clear_texture, readback_rgba8,
    },
};
use wgpu::wgc::api::Vulkan;

fn create_exportable_texture(device: &wgpu::Device) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: DEFAULT_SIZE.x,
        height: DEFAULT_SIZE.y,
        depth_or_array_layers: 1,
    };
    let usage = wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::COPY_DST
        | wgpu::TextureUsages::TEXTURE_BINDING;
    let vk_format = map_wgpu_format(DEFAULT_FORMAT).expect("format");
    let vk_usage =
        waywe_rendering_api::conversions::map_texture_usages_to_vk(usage, DEFAULT_FORMAT);

    let hal_device = unsafe { device.as_hal::<Vulkan>() }.expect("vulkan");
    let raw_device = hal_device.raw_device();
    let raw_phys = hal_device.raw_physical_device();
    let instance = hal_device.shared_instance().raw_instance();

    let mut ext_mem_info = vk::ExternalMemoryImageCreateInfo::default()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);

    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(vk_format)
        .extent(vk::Extent3D {
            width: size.width,
            height: size.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk_usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .push_next(&mut ext_mem_info);

    let vk_image = unsafe { raw_device.create_image(&image_info, None) }.unwrap();
    let mem_req = unsafe { raw_device.get_image_memory_requirements(vk_image) };
    let mem_props = unsafe { instance.get_physical_device_memory_properties(raw_phys) };
    let mem_type_index = mem_props
        .memory_types_as_slice()
        .iter()
        .enumerate()
        .find(|(i, ty)| {
            let bit = 1u32 << i;
            mem_req.memory_type_bits & bit != 0
                && ty
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
        })
        .map(|(i, _)| i as u32)
        .expect("memory type");

    let dedicated_info = vk::MemoryDedicatedAllocateInfo {
        s_type: vk::StructureType::MEMORY_DEDICATED_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        image: vk_image,
        buffer: vk::Buffer::null(),
        _marker: PhantomData,
    };
    let export_info = vk::ExportMemoryAllocateInfo {
        s_type: vk::StructureType::EXPORT_MEMORY_ALLOCATE_INFO,
        p_next: (&raw const dedicated_info).cast(),
        handle_types: vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD,
        _marker: PhantomData,
    };
    let alloc_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: (&raw const export_info).cast(),
        allocation_size: mem_req.size,
        memory_type_index: mem_type_index,
        _marker: PhantomData,
    };

    let memory = unsafe { raw_device.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { raw_device.bind_image_memory(vk_image, memory, 0) }.unwrap();

    let hal_desc = wgpu::hal::TextureDescriptor {
        label: Some("fd-exportable"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEFAULT_FORMAT,
        usage: wgpu::TextureUses::COLOR_TARGET
            | wgpu::TextureUses::COPY_SRC
            | wgpu::TextureUses::COPY_DST
            | wgpu::TextureUses::RESOURCE,
        memory_flags: wgpu::hal::MemoryFlags::empty(),
        view_formats: vec![],
    };

    let texture_hal = unsafe {
        hal_device.texture_from_raw(
            vk_image,
            &hal_desc,
            None,
            wgpu::hal::vulkan::TextureMemory::Dedicated(memory),
        )
    };

    let wgpu_desc = wgpu::TextureDescriptor {
        label: Some("fd-exportable"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEFAULT_FORMAT,
        usage,
        view_formats: &[],
    };

    unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Vulkan>(
            texture_hal,
            &wgpu_desc,
            wgpu::TextureUses::UNINITIALIZED,
        )
    }
}

#[test]
fn export_import_fd_preserves_clear_color() {
    let gpu = Arc::new(Wgpu::new());
    let src = create_exportable_texture(&gpu.device);

    clear_texture(
        &gpu,
        &src,
        wgpu::Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    );

    let desc = FfiTextureDescriptor::from(wgpu::TextureDescriptor {
        label: Some("fd-import"),
        size: src.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEFAULT_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let fd = unsafe { texture_export_fd(&gpu.device, &src) };
    let imported = unsafe { import_fd_as_texture(&gpu.device, &gpu.adapter, fd, desc) };

    let image = readback_rgba8(&gpu, &imported);
    assert_avg_color(
        &image,
        Rect::full(DEFAULT_SIZE),
        [0, 255, 0, 255],
        8,
        "fd_roundtrip_green",
    );
}
