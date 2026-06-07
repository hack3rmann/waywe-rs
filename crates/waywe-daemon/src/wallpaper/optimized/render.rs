use crate::wallpaper::Wallpaper;
use ash::vk;
use libloading::Library;
use std::{mem::MaybeUninit, path::Path};
use waywe_runtime::{WallpaperConfig, frame::FrameInfo, gpu::Wgpu};
use waywe_subrenderer::{
    FfiTextureDescriptor,
    api::{
        CREATE_OPAQUE_RENDERER_NAME, CreateOpaqueRendererFn, OpaqueRenderer, OpaqueRendererDesc,
        RenderSurfaceFd, Renderer,
    },
    conversions::{map_texture_usages_to_vk, map_wgpu_format},
    texture_export_fd,
};
use wgpu::wgc::api::Vulkan;

pub struct RenderWallpaper {
    // NOTE(hack3rmann): `renderer` must be dropped before `_lib`
    renderer: OpaqueRenderer,
    _lib: Library,
    surface: wgpu::Texture,
    config: WallpaperConfig,
}

unsafe impl Send for RenderWallpaper {}
unsafe impl Sync for RenderWallpaper {}

impl RenderWallpaper {
    pub fn load(path: impl AsRef<Path>, gpu: &Wgpu, config: WallpaperConfig) -> Self {
        let lib = unsafe { Library::new(path.as_ref()) }.unwrap();

        let create_opaque_renderer = unsafe {
            lib.get::<CreateOpaqueRendererFn>(CREATE_OPAQUE_RENDERER_NAME)
                .unwrap()
        };

        let mut renderer = MaybeUninit::uninit();

        let panic = create_opaque_renderer(&OpaqueRendererDesc { config }, &mut renderer);
        panic.propagate_if_any();

        let mut renderer = unsafe { renderer.assume_init() };

        let surface_desc = Self::surface_desc(config);
        let surface = Self::create_texture(&gpu.device, config);

        renderer.set_surface(RenderSurfaceFd {
            fd: unsafe { texture_export_fd(&gpu.device, &surface) },
            desc: FfiTextureDescriptor::from(surface_desc),
        });

        Self {
            renderer,
            _lib: lib,
            surface,
            config,
        }
    }

    fn surface_desc(config: WallpaperConfig) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some("waywe-subrenderer-surface"),
            size: wgpu::Extent3d {
                width: config.surface_size.x,
                height: config.surface_size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.surface_format,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }
    }

    fn create_texture(device: &wgpu::Device, config: WallpaperConfig) -> wgpu::Texture {
        let wgpu_desc = Self::surface_desc(config);
        let vk_format =
            map_wgpu_format(wgpu_desc.format).expect("unsupported wallpaper surface format");
        let vk_usage = map_texture_usages_to_vk(wgpu_desc.usage, wgpu_desc.format);

        let hal_device = unsafe { device.as_hal::<Vulkan>() }.expect("not a Vulkan device");
        let raw_device = hal_device.raw_device();
        let raw_phys = hal_device.raw_physical_device();
        let instance = hal_device.shared_instance().raw_instance();

        // OPAQUE_FD + OPTIMAL same-process sharing between the daemon device and the
        // scene plugin device on the same physical GPU. The import side must recreate
        // an identical VkImage (see import_fd_as_texture).
        let mut ext_mem_info = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk_format)
            .extent(vk::Extent3D {
                width: wgpu_desc.size.width,
                height: wgpu_desc.size.height,
                depth: 1,
            })
            .mip_levels(wgpu_desc.mip_level_count)
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
            .expect("no suitable memory type");

        let dedicated_info = vk::MemoryDedicatedAllocateInfo {
            s_type: vk::StructureType::MEMORY_DEDICATED_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            image: vk_image,
            buffer: vk::Buffer::null(),
            _marker: std::marker::PhantomData,
        };

        let export_info = vk::ExportMemoryAllocateInfo {
            s_type: vk::StructureType::EXPORT_MEMORY_ALLOCATE_INFO,
            p_next: (&raw const dedicated_info).cast(),
            handle_types: vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD,
            _marker: std::marker::PhantomData,
        };

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: (&raw const export_info).cast(),
            allocation_size: mem_req.size,
            memory_type_index: mem_type_index,
            _marker: std::marker::PhantomData,
        };

        let memory = unsafe { raw_device.allocate_memory(&alloc_info, None) }.unwrap();
        unsafe { raw_device.bind_image_memory(vk_image, memory, 0) }.unwrap();

        let hal_desc = wgpu::hal::TextureDescriptor {
            label: wgpu_desc.label,
            size: wgpu_desc.size,
            mip_level_count: wgpu_desc.mip_level_count,
            sample_count: 1,
            dimension: wgpu_desc.dimension,
            format: wgpu_desc.format,
            usage: wgpu::TextureUses::COLOR_TARGET | wgpu::TextureUses::COPY_SRC,
            memory_flags: wgpu::hal::MemoryFlags::empty(),
            view_formats: vec![],
        };

        let hal_texture = unsafe {
            hal_device.texture_from_raw(
                vk_image,
                &hal_desc,
                None,
                wgpu::hal::vulkan::TextureMemory::Dedicated(memory),
            )
        };

        unsafe { device.create_texture_from_hal::<Vulkan>(hal_texture, &wgpu_desc) }
    }
}

impl Wallpaper for RenderWallpaper {
    fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig) {
        if config == self.config {
            return;
        }

        let desc = Self::surface_desc(config);
        self.surface = Self::create_texture(&gpu.device, config);

        self.renderer.set_surface(RenderSurfaceFd {
            fd: unsafe { texture_export_fd(&gpu.device, &self.surface) },
            desc: FfiTextureDescriptor::from(desc),
        });
    }

    fn frame(
        &mut self,
        _: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo {
        let info = self.renderer.render();

        encoder.copy_texture_to_texture(
            self.surface.as_image_copy(),
            surface.texture().as_image_copy(),
            surface.texture().size(),
        );

        info
    }
}
