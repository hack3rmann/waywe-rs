use ash::vk;
use waywe_runtime::shaders::{ShaderCache, ShaderDescriptor};
use wgpu::hal::{DeviceError, api};

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub shader_cache: ShaderCache,
}

impl Gpu {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: if cfg!(debug_assertions) {
                wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION
            } else {
                wgpu::InstanceFlags::empty()
            },
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::from_env_or_default(),
            display: None,
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

        let limits = adapter.limits();

        let features = wgpu::Features::TEXTURE_FORMAT_NV12
            | wgpu::Features::IMMEDIATES
            | wgpu::Features::BGRA8UNORM_STORAGE
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | wgpu::Features::PIPELINE_CACHE;
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

            let family_index = 0;
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
                    &limits,
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

        Self {
            adapter,
            instance,
            device,
            queue,
            shader_cache: ShaderCache::default(),
        }
    }

    pub fn require_shader<S: ShaderDescriptor>(&self) {
        self.shader_cache.initialize::<S>(&self.device);
    }
}
