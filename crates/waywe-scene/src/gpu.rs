use waywe_runtime::shaders::{ShaderCache, ShaderDescriptor};

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub shader_cache: ShaderCache,
}

impl Gpu {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::from_build_config(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
        });

        let Ok(adapter) = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // TODO(hack3rmann): let the user configure power preference
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
        else {
            panic!("failed to request adapter");
        };

        let Ok((device, queue)) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("waywe-scene-device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
        else {
            panic!("failed to request device");
        };

        Self {
            shader_cache: ShaderCache::default(),
            device,
            queue,
            instance,
            adapter,
        }
    }

    pub fn require_shader<S: ShaderDescriptor>(&self) {
        self.shader_cache.initialize::<S>(&self.device);
    }
}
