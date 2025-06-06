use super::wayland::Wayland;

pub struct Wgpu {
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
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

        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: wayland.raw_display_handle(),
                    raw_window_handle: wayland.raw_window_handle(),
                })
                .unwrap()
        };

        let Some(adapter) = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
        else {
            panic!("failed to request adapter");
        };

        let Ok((device, queue)) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::PUSH_CONSTANTS,
                    label: None,
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
        else {
            panic!("failed to request device");
        };

        let screen_size = wayland.client_state.monitor_size();

        surface.configure(
            &device,
            &surface
                .get_default_config(&adapter, screen_size.x, screen_size.y)
                .unwrap(),
        );

        let Some(surface_format) = surface.get_capabilities(&adapter).formats.first().copied()
        else {
            panic!("no surface format supported");
        };

        Self {
            adapter,
            instance,
            device,
            queue,
            surface,
            surface_format,
        }
    }
}
