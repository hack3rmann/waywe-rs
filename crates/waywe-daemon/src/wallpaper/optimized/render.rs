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
        // 1. Get the HAL device
        let hal_device = unsafe { device.as_hal::<Vulkan>() }.expect("not a Vulkan device");

        let raw_device = hal_device.raw_device(); // &ash::Device  
        let raw_phys = hal_device.raw_physical_device(); // vk::PhysicalDevice  
        let instance = hal_device.shared_instance().raw_instance();
        // NOTE: .raw may be pub(crate);
        // check wgpu_hal::vulkan::InstanceShared

        // 2. Create VkImage with DMA-buf export + DRM modifier support
        let modifiers = [0]; // DRM_FOURCC_LINEAR

        let mut ext_mem_info = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

        let mut modifier_list =
            vk::ImageDrmFormatModifierListCreateInfoEXT::default().drm_format_modifiers(&modifiers);

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::B8G8R8A8_SRGB) // map from config.surface_format manually
            .extent(vk::Extent3D {
                width: config.surface_size.x,
                height: config.surface_size.y,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut ext_mem_info)
            .push_next(&mut modifier_list);

        let vk_image = unsafe { raw_device.create_image(&image_info, None) }.unwrap();

        // 3. Get memory requirements
        let mem_req = unsafe { raw_device.get_image_memory_requirements(vk_image) };

        // 4. Find a DEVICE_LOCAL memory type
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

        // 5. Allocate with ExportMemoryAllocateInfo
        let mut export_info = vk::ExportMemoryAllocateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

        let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().image(vk_image);

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_req.size)
            .memory_type_index(mem_type_index)
            .push_next(&mut export_info)
            .push_next(&mut dedicated_info);

        let memory = unsafe { raw_device.allocate_memory(&alloc_info, None) }.unwrap();

        // 6. Bind
        unsafe { raw_device.bind_image_memory(vk_image, memory, 0) }.unwrap();

        // 8. Wrap into a wgpu-hal Texture
        //    Build the HAL descriptor matching your image_info exactly.
        let hal_desc = wgpu::hal::TextureDescriptor {
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
            usage: wgpu::TextureUses::COLOR_TARGET | wgpu::TextureUses::COPY_SRC,
            memory_flags: wgpu::hal::MemoryFlags::empty(),
            view_formats: vec![],
        };

        let hal_texture = unsafe {
            hal_device.texture_from_raw(
                vk_image,
                &hal_desc,
                None, // wgpu-hal takes ownership; it will destroy vk_image + memory on drop
                wgpu::hal::vulkan::TextureMemory::Dedicated(memory),
            )
        };

        // 9. Wrap into a wgpu Texture
        unsafe {
            device.create_texture_from_hal::<Vulkan>(
                hal_texture,
                &wgpu::TextureDescriptor {
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
                },
            )
        }
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
