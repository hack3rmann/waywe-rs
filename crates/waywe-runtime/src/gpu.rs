use super::wayland::{MonitorId, MonitorInfo, MonitorMap, SurfaceExtension};
use crate::{
    shaders::{ShaderCache, ShaderDescriptor},
    wayland::Wayland,
};
use ash::vk;
use glam::UVec2;
use std::sync::RwLock;
use wgpu::hal::{DeviceError, api};

pub struct Surface {
    pub surface: wgpu::Surface<'static>,
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
            | wgpu::Features::PIPELINE_CACHE;

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

    pub fn resize_surface(&self, monitor_id: MonitorId, size: UVec2) {
        let mut surfaces = self.surfaces.write().unwrap();

        let Some(info) = surfaces.get_mut(&monitor_id) else {
            return;
        };

        info.config = get_surface_config(&info.surface, &self.adapter, size);
        info.surface.configure(&self.device, &info.config);
    }

    pub fn unregister_surface(&self, monitor_id: MonitorId) {
        let mut surfaces = self.surfaces.write().unwrap();
        _ = surfaces.remove(&monitor_id);
    }

    pub fn register_surface(&self, wayland: &Wayland, monitor_id: MonitorId) {
        let monitors = wayland.client_state.monitors.read().unwrap();
        let info = &monitors[&monitor_id];

        let surface = create_surface(
            &self.instance,
            &self.adapter,
            &self.device,
            wayland,
            info,
            monitor_id,
        );

        let mut surfaces = self.surfaces.write().unwrap();
        surfaces.insert(monitor_id, surface);
    }

    pub fn require_shader<S: ShaderDescriptor>(&self) {
        self.shader_cache.initialize::<S>(&self.device);
    }

    pub fn reconfigure_surface(&self, monitor_id: MonitorId) {
        let surfaces = self.surfaces.read().unwrap();
        let Some(info) = surfaces.get(&monitor_id) else {
            return;
        };
        info.surface.configure(&self.device, &info.config);
    }

    /// # Note
    ///
    /// Returns `None` if this frame should be skipped
    pub fn get_current_surface(&self, wayland: &Wayland, monitor_id: MonitorId) -> SurfaceResult {
        const N_TRIES: usize = 4;

        let mut surfaces = self.surfaces.write().unwrap();
        let Some(info) = surfaces.get_mut(&monitor_id) else {
            return SurfaceResult::Err;
        };

        for _ in 0..N_TRIES {
            let surface_result = info.surface.get_current_texture();

            match surface_result {
                wgpu::CurrentSurfaceTexture::Success(texture) => return SurfaceResult::Ok(texture),
                wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                    return SurfaceResult::Reconfigure(texture);
                }
                wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                    return SurfaceResult::Skip;
                }
                wgpu::CurrentSurfaceTexture::Outdated => {
                    info.surface.configure(&self.device, &info.config);
                }
                wgpu::CurrentSurfaceTexture::Lost => {
                    let monitors = wayland.client_state.monitors.read().unwrap();
                    let monitor_info = &monitors[&monitor_id];

                    let new_info = create_surface(
                        &self.instance,
                        &self.adapter,
                        &self.device,
                        wayland,
                        monitor_info,
                        monitor_id,
                    );

                    *info = new_info;
                }
                wgpu::CurrentSurfaceTexture::Validation => return SurfaceResult::Err,
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

#[derive(Clone, Debug, PartialEq)]
pub enum SurfaceResult {
    Ok(wgpu::SurfaceTexture),
    Reconfigure(wgpu::SurfaceTexture),
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
    wayland: &Wayland,
    info: &MonitorInfo,
    id: MonitorId,
) -> Surface {
    let handle = {
        let queue = wayland.main_queue.read().unwrap();
        queue
            .as_ref()
            .storage()
            .object(info.surface)
            .raw_window_handle()
    };

    let surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(wayland.raw_display_handle()),
                raw_window_handle: handle,
            })
            .unwrap()
    };

    let screen_size = wayland.client_state.monitor_size(id).unwrap();

    let Some(format) = surface.get_capabilities(adapter).formats.first().copied() else {
        panic!("no surface format supported");
    };

    let config = get_surface_config(&surface, adapter, screen_size);

    surface.configure(device, &config);

    Surface {
        surface,
        format,
        config,
    }
}

fn get_surface_config(
    surface: &wgpu::Surface,
    adapter: &wgpu::Adapter,
    screen_size: UVec2,
) -> wgpu::SurfaceConfiguration {
    let Some(format) = surface.get_capabilities(adapter).formats.first().copied() else {
        panic!("no surface format supported");
    };

    let config = surface
        .get_default_config(adapter, screen_size.x, screen_size.y)
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
