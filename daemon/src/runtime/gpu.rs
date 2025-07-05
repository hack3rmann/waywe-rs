use super::wayland::Wayland;
use ash::vk;
use glam::UVec2;
use smallvec::SmallVec;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{RwLock, RwLockReadGuard},
};
use wgpu::hal::{DeviceError, api};

#[derive(Default)]
pub struct ShaderCache {
    shaders: RwLock<HashMap<&'static str, wgpu::ShaderModule>>,
}

impl ShaderCache {
    pub fn contains(&self, id: &str) -> bool {
        self.shaders.read().unwrap().contains_key(id)
    }

    pub fn insert_with(
        &self,
        id: &'static str,
        create_shader: impl FnOnce() -> wgpu::ShaderModule,
    ) {
        if self.contains(id) {
            return;
        }

        let mut map = self.shaders.write().unwrap();
        map.insert(id, create_shader());
    }

    pub fn get(&self, id: &'static str) -> Option<RwLockShaderReadGuard<'_>> {
        let shaders = self.shaders.read().unwrap();

        if !shaders.contains_key(id) {
            return None;
        }

        Some(RwLockShaderReadGuard { shaders, id })
    }
}

pub struct RwLockShaderReadGuard<'s> {
    shaders: RwLockReadGuard<'s, HashMap<&'static str, wgpu::ShaderModule>>,
    id: &'static str,
}

impl Deref for RwLockShaderReadGuard<'_> {
    type Target = wgpu::ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.shaders[self.id]
    }
}

pub struct Wgpu {
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surfaces: SmallVec<[wgpu::Surface<'static>; 1]>,
    pub surface_formats: SmallVec<[wgpu::TextureFormat; 1]>,
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

        let surfaces = wayland
            .raw_window_handles()
            .map(|raw_window_handle| unsafe {
                instance
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle: wayland.raw_display_handle(),
                        raw_window_handle,
                    })
                    .unwrap()
            })
            .collect::<SmallVec<[wgpu::Surface; 1]>>();

        let Some(adapter) = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(surfaces.first().unwrap()),
            })
            .await
        else {
            panic!("failed to request adapter");
        };

        let features = wgpu::Features::TEXTURE_FORMAT_NV12 | wgpu::Features::PUSH_CONSTANTS;
        let memory_hints = wgpu::MemoryHints::Performance;

        let open_device = unsafe {
            adapter.as_hal::<api::Vulkan, _, _>(|adapter| {
                let Some(adapter) = adapter else {
                    unreachable!()
                };

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
            })
        };

        let Ok((device, queue)) = (unsafe {
            adapter.create_device_from_hal::<api::Vulkan>(
                open_device,
                &wgpu::DeviceDescriptor {
                    required_features: features,
                    label: Some("waywe-gpu-device"),
                    required_limits: adapter.limits(),
                    memory_hints,
                },
                None,
            )
        }) else {
            panic!("failed to request device")
        };

        let surface_formats = surfaces
            .iter()
            .enumerate()
            .map(|(monitor_index, surface)| {
                let capabilities = surface.get_capabilities(&adapter);
                let screen_size = wayland.client_state.monitor_size(monitor_index).unwrap();

                let Some(surface_format) =
                    surface.get_capabilities(&adapter).formats.first().copied()
                else {
                    panic!("no surface format supported");
                };

                // TODO(hack3rmann): configure surface with
                // `usage |= wgt::TextureUsages::STORAGE_BINDING`
                // to render to it using compute shaders
                let surface_config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: surface_format,
                    width: screen_size.x,
                    height: screen_size.y,
                    desired_maximum_frame_latency: 2,
                    present_mode: *capabilities
                        .present_modes
                        .first()
                        .expect("should has at least one format"),
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                };

                surface.configure(&device, &surface_config);

                surface_format
            })
            .collect();

        Self {
            adapter,
            instance,
            device,
            queue,
            surfaces,
            surface_formats,
            shader_cache: ShaderCache::default(),
        }
    }

    pub fn resize_surface(&self, size: UVec2) {
        // FIXME(hack3rmann): multiple monitors
        self.surfaces.first().unwrap().configure(
            &self.device,
            &self
                .surfaces
                .first()
                .unwrap()
                .get_default_config(&self.adapter, size.x, size.y)
                .unwrap(),
        );
    }

    pub fn use_shader(&self, id: &'static str, desc: wgpu::ShaderModuleDescriptor) {
        self.shader_cache
            .insert_with(id, || self.device.create_shader_module(desc));
    }
}
