use super::wayland::{MonitorId, MonitorInfo, MonitorMap, SurfaceExtension, Wayland};
use crate::shaders::{ShaderCache, ShaderDescriptor};
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
    pub async fn new(wayland: &Wayland) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: if cfg!(debug_assertions) {
                wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION
            } else {
                wgpu::InstanceFlags::empty()
            },
            ..Default::default()
        });

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                // take any available surface
                compatible_surface: None,
            })
            .await
        {
            Ok(adapter) => adapter,
            Err(error) => panic!("failed to request adapter: {error:?}"),
        };

        let features = wgpu::Features::TEXTURE_FORMAT_NV12
            | wgpu::Features::PUSH_CONSTANTS
            | wgpu::Features::BGRA8UNORM_STORAGE
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        let memory_hints = wgpu::MemoryHints::Performance;

        let open_device = unsafe {
            let adapter = adapter.as_hal::<api::Vulkan>().unwrap();

            let mut enabled_extensions = adapter.required_device_extensions(features);
            enabled_extensions.extend_from_slice(&[
                c"VK_KHR_external_memory_fd",
                c"VK_EXT_image_drm_format_modifier",
            ]);

            let mut enabled_phd_features =
                adapter.physical_device_features(&enabled_extensions, features);

            let family_index = 0; //TODO
            let family_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(family_index)
                .queue_priorities(&[1.0]);
            let family_infos = [family_info];

            let str_pointers = enabled_extensions
                .iter()
                .map(|&s| {
                    // Safe because `enabled_extensions` entries have static lifetime.
                    s.as_ptr()
                })
                .collect::<Vec<_>>();

            let pre_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&family_infos)
                .enabled_extension_names(&str_pointers);
            let info = enabled_phd_features.add_to_device_create(pre_info);
            let raw_device = adapter
                .shared_instance()
                .raw_instance()
                .create_device(adapter.raw_physical_device(), &info, None)
                .map_err(map_err)
                .unwrap();

            fn map_err(err: vk::Result) -> DeviceError {
                match err {
                    vk::Result::ERROR_TOO_MANY_OBJECTS => DeviceError::OutOfMemory,
                    vk::Result::ERROR_INITIALIZATION_FAILED => DeviceError::Lost,
                    vk::Result::ERROR_EXTENSION_NOT_PRESENT
                    | vk::Result::ERROR_FEATURE_NOT_PRESENT => {
                        panic!("{err:?}");
                    }
                    _ => todo!(),
                }
            }

            adapter
                .device_from_raw(
                    raw_device,
                    None,
                    &enabled_extensions,
                    features,
                    &memory_hints,
                    family_info.queue_family_index,
                    0,
                )
                .unwrap()
        };

        let (device, queue) = match unsafe {
            adapter.create_device_from_hal::<api::Vulkan>(
                open_device,
                &wgpu::DeviceDescriptor {
                    required_features: features,
                    label: Some("waywe-gpu-device"),
                    required_limits: adapter.limits(),
                    memory_hints,
                    trace: wgpu::Trace::Off,
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                },
            )
        } {
            Ok(x) => x,
            Err(error) => panic!("failed to request device: {error}"),
        };

        let surfaces = wayland
            .client_state
            .monitors
            .read()
            .unwrap()
            .iter()
            .map(|(&id, info)| {
                (
                    id,
                    create_surface(&instance, &adapter, &device, wayland, info, id),
                )
            })
            .collect::<MonitorMap<_>>();

        Self {
            adapter,
            instance,
            device,
            queue,
            surfaces: RwLock::new(surfaces),
            shader_cache: ShaderCache::default(),
        }
    }

    pub fn resize_surface(&self, monitor_id: MonitorId, size: UVec2) {
        let surfaces = self.surfaces.read().unwrap();

        let Some(surface) = surfaces.get(&monitor_id) else {
            return;
        };

        let surface_config = surface
            .surface
            .get_default_config(&self.adapter, size.x, size.y)
            .unwrap();

        surface.surface.configure(&self.device, &surface_config);
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
                raw_display_handle: wayland.raw_display_handle(),
                raw_window_handle: handle,
            })
            .unwrap()
    };

    let screen_size = wayland.client_state.monitor_size(id).unwrap();

    let Some(format) = surface.get_capabilities(adapter).formats.first().copied() else {
        panic!("no surface format supported");
    };

    let config = surface
        .get_default_config(adapter, screen_size.x, screen_size.y)
        .unwrap();

    // TODO(hack3rmann): configure surface with
    // `usage |= wgt::TextureUsages::STORAGE_BINDING`
    // to render to it using compute shaders
    let config = wgpu::SurfaceConfiguration {
        // NOTE(hack3rmann): `COPY_SRC` used to allow transitions between wallpapers
        usage: config.usage
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: vec![format.remove_srgb_suffix()],
        ..config
    };

    surface.configure(device, &config);

    Surface {
        surface,
        format,
        config,
    }
}
