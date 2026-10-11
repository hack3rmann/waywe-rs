use crate::{
    platform::{
        CurrentSurfaceTexture, MonitorMap, SurfaceInfo, SurfaceTexture, WaywePlatform, WayweSurface,
    },
    shaders::{ShaderCache, ShaderDescriptor},
};
use ash::vk;
use glam::UVec2;
use std::sync::RwLock;
use wgpu::hal::{DeviceError, api};

pub struct Surface {
    pub surface: Box<dyn WayweSurface>,
    pub config: wgpu::SurfaceConfiguration,
    pub format: wgpu::TextureFormat,
}

pub struct Wgpu {
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surfaces: RwLock<MonitorMap<Surface>>,
    pub shader_cache: ShaderCache,
}

impl Wgpu {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::from_build_config(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::from_env_or_default(),
            // NOTE(hack3rmann): on Vulkan this handle is unused
            display: None,
        });

        let adapter_opts = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            // take any available surface
            compatible_surface: None,
            apply_limit_buckets: false,
        };

        // NOTE(hack3rmann): when the backend is Vulkan, `wgpu::Instance::request_adapter` is
        // effectively synchronous, so no need to spread `async`-ness desease
        let adapter_result = pollster::block_on(instance.request_adapter(&adapter_opts));

        let adapter = match adapter_result {
            Ok(adapter) => adapter,
            Err(error) => panic!("failed to request adapter: {error:?}"),
        };

        let features = wgpu::Features::TEXTURE_FORMAT_NV12
            | wgpu::Features::IMMEDIATES
            | wgpu::Features::BGRA8UNORM_STORAGE
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | wgpu::Features::PIPELINE_CACHE
            | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;

        let memory_hints = wgpu::MemoryHints::Performance;

        let (device, queue) = match create_device_and_queue(&adapter, features, memory_hints) {
            Ok(both) => both,
            Err(error) => panic!("failed to request device: {error}"),
        };

        Self {
            adapter,
            instance,
            device,
            queue,
            surfaces: RwLock::default(),
            shader_cache: ShaderCache::default(),
        }
    }

    pub fn resize_surface(&self, monitor_name: &str, size: UVec2) {
        let mut surfaces = self.surfaces.write().unwrap();

        let Some(info) = surfaces.get_mut(monitor_name) else {
            return;
        };

        info.config = info.surface.get_config(&self.adapter, size);
        info.surface.configure(&self.device, &info.config);
    }

    pub fn unregister_surface(&self, monitor_name: &str) {
        let mut surfaces = self.surfaces.write().unwrap();
        _ = surfaces.remove(monitor_name);
    }

    pub fn register_surface(&self, platform: &dyn WaywePlatform, info: &SurfaceInfo) {
        let surface = create_surface(&self.instance, &self.adapter, &self.device, platform, info);

        let mut surfaces = self.surfaces.write().unwrap();
        surfaces.insert(info.monitor_name.clone(), surface);
    }

    pub fn require_shader<S: ShaderDescriptor>(&self) {
        self.shader_cache.initialize::<S>(&self.device);
    }

    pub fn reconfigure_surface(&self, monitor_name: &str) {
        let surfaces = self.surfaces.read().unwrap();
        let Some(info) = surfaces.get(monitor_name) else {
            return;
        };
        info.surface.configure(&self.device, &info.config);
    }

    /// # Note
    ///
    /// Returns `None` if this frame should be skipped
    pub fn get_current_surface(
        &self,
        platform: &dyn WaywePlatform,
        info: &SurfaceInfo,
    ) -> SurfaceResult {
        const N_TRIES: usize = 4;

        let mut surfaces = self.surfaces.write().unwrap();
        let Some(surface_info) = surfaces.get_mut(&info.monitor_name) else {
            return SurfaceResult::Err;
        };

        for _ in 0..N_TRIES {
            match surface_info.surface.get_current_texture() {
                CurrentSurfaceTexture::Success(texture) => return SurfaceResult::Ok(texture),
                CurrentSurfaceTexture::Suboptimal(texture) => {
                    return SurfaceResult::Reconfigure(texture);
                }
                CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                    return SurfaceResult::Skip;
                }
                CurrentSurfaceTexture::Outdated => {
                    surface_info
                        .surface
                        .configure(&self.device, &surface_info.config);
                }
                CurrentSurfaceTexture::Lost => {
                    let new_info =
                        create_surface(&self.instance, &self.adapter, &self.device, platform, info);

                    *surface_info = new_info;
                }
                CurrentSurfaceTexture::Validation => return SurfaceResult::Err,
            }
        }

        SurfaceResult::Err
    }
}

impl Default for Wgpu {
    fn default() -> Self {
        Self::new()
    }
}

pub enum SurfaceResult {
    Ok(Box<dyn SurfaceTexture>),
    Reconfigure(Box<dyn SurfaceTexture>),
    Skip,
    Err,
}

fn create_device_and_queue(
    adapter: &wgpu::Adapter,
    features: wgpu::Features,
    memory_hints: wgpu::MemoryHints,
) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    let adapter_hal = unsafe { adapter.as_hal::<api::Vulkan>().unwrap() };
    let instance_raw = adapter_hal.shared_instance().raw_instance();

    let mut enabled_extensions = adapter_hal.required_device_extensions(features);
    enabled_extensions.extend_from_slice(&[
        c"VK_KHR_external_memory_fd",
        c"VK_EXT_image_drm_format_modifier",
    ]);

    let mut enabled_phd_features =
        adapter_hal.physical_device_features(&enabled_extensions, features);

    let family_index = 0;
    let family_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(family_index)
        .queue_priorities(&[1.0]);
    let family_infos = [family_info];

    let str_pointers = enabled_extensions
        .iter()
        .map(|&s| s.as_ptr())
        .collect::<Vec<_>>();

    let pre_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&family_infos)
        .enabled_extension_names(&str_pointers);
    let info = enabled_phd_features.add_to_device_create(pre_info);
    let raw_device =
        unsafe { instance_raw.create_device(adapter_hal.raw_physical_device(), &info, None) }
            .map_err(|err| match err {
                vk::Result::ERROR_TOO_MANY_OBJECTS => DeviceError::OutOfMemory,
                vk::Result::ERROR_INITIALIZATION_FAILED => DeviceError::Lost,
                vk::Result::ERROR_EXTENSION_NOT_PRESENT | vk::Result::ERROR_FEATURE_NOT_PRESENT => {
                    panic!("{err:?}");
                }
                _ => unimplemented!(),
            })
            .map_err(wgpu::wgc::device::DeviceError::from_hal)
            .map_err(wgpu::wgc::instance::RequestDeviceError::Device)?;

    let open_device = unsafe {
        adapter_hal.device_from_raw(
            raw_device,
            None,
            &enabled_extensions,
            features,
            &adapter.limits(),
            &memory_hints,
            family_info.queue_family_index,
            0,
        )
    }
    .unwrap();

    let desc = wgpu::DeviceDescriptor {
        required_features: features,
        label: Some("waywe-gpu-device"),
        required_limits: adapter.limits(),
        memory_hints,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    };

    unsafe { adapter.create_device_from_hal::<api::Vulkan>(open_device, &desc) }
}

fn create_surface(
    instance: &wgpu::Instance,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    platform: &dyn WaywePlatform,
    info: &SurfaceInfo,
) -> Surface {
    let surface = platform.create_surface(instance, &info.monitor_name);
    let format = surface.get_format(adapter);
    let config = surface.get_config(adapter, info.physical_size);

    surface.configure(device, &config);

    Surface {
        surface,
        format,
        config,
    }
}

impl SurfaceTexture for wgpu::SurfaceTexture {
    fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn present(self: Box<Self>, queue: &wgpu::Queue) {
        let texture = *self;
        queue.present(texture);
    }
}

impl WayweSurface for wgpu::Surface<'static> {
    fn get_current_texture(&self) -> CurrentSurfaceTexture {
        match wgpu::Surface::get_current_texture(self) {
            wgpu::CurrentSurfaceTexture::Success(texture) => {
                CurrentSurfaceTexture::Success(Box::new(texture))
            }
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                CurrentSurfaceTexture::Suboptimal(Box::new(texture))
            }
            wgpu::CurrentSurfaceTexture::Timeout => CurrentSurfaceTexture::Timeout,
            wgpu::CurrentSurfaceTexture::Occluded => CurrentSurfaceTexture::Occluded,
            wgpu::CurrentSurfaceTexture::Outdated => CurrentSurfaceTexture::Outdated,
            wgpu::CurrentSurfaceTexture::Lost => CurrentSurfaceTexture::Lost,
            wgpu::CurrentSurfaceTexture::Validation => CurrentSurfaceTexture::Validation,
        }
    }

    fn configure(&self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        wgpu::Surface::configure(self, device, config)
    }

    fn get_format(&self, adapter: &wgpu::Adapter) -> wgpu::TextureFormat {
        self.get_capabilities(adapter)
            .formats
            .first()
            .copied()
            .unwrap_or_else(|| panic!("no surface format supported"))
    }

    fn get_config(
        &self,
        adapter: &wgpu::Adapter,
        physical_size: UVec2,
    ) -> wgpu::SurfaceConfiguration {
        let Some(format) = self.get_capabilities(adapter).formats.first().copied() else {
            panic!("no surface format supported");
        };

        let config = self
            .get_default_config(adapter, physical_size.x, physical_size.y)
            .unwrap();

        // TODO(hack3rmann): configure surface with
        // `usage |= wgt::TextureUsages::STORAGE_BINDING`
        // to render to it using compute shaders
        wgpu::SurfaceConfiguration {
            // NOTE(hack3rmann): `COPY_SRC` used to allow transitions between wallpapers
            usage: config.usage
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: vec![format.remove_srgb_suffix()],
            ..config
        }
    }
}
