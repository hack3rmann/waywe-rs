use crate::platform::{SurfaceInfo, WaywePlatform};
use calloop::channel::Sender;
use glam::UVec2;
use gpu::Wgpu;
use std::sync::{Arc, Once};
use tasks::Tasks;
use timer::Timer;
use waywe_ipc::{command::DaemonResult, ipc::server::IpcResponse};

pub mod app;
pub mod effects;
pub mod event;
pub mod frame;
pub mod gpu;
pub mod platform;
pub mod shaders;
pub mod tasks;
pub mod timer;
pub mod wayland;

pub struct Runtime {
    pub timer: Timer,
    pub wgpu: Arc<Wgpu>,
    pub platform: Arc<dyn WaywePlatform>,
    pub tasks: Tasks,
    pub ipc: Sender<IpcResponse<DaemonResult>>,
}

impl Runtime {
    pub fn new(
        platform: Arc<dyn WaywePlatform>,
        tasks: Tasks,
        ipc: Sender<IpcResponse<DaemonResult>>,
    ) -> Self {
        static VIDEO_ONCE: Once = Once::new();
        VIDEO_ONCE.call_once(video::init);

        Self {
            ipc,
            timer: Timer::default(),
            wgpu: Arc::default(),
            platform,
            tasks,
        }
    }

    pub fn wallpaper_config(&self, info: &SurfaceInfo) -> Option<WallpaperConfig> {
        let surface_format = {
            let surfaces = self.wgpu.surfaces.read().unwrap();
            surfaces.get(&info.monitor_name)?.format
        };

        Some(WallpaperConfig {
            surface_size: info.phisical_size,
            surface_format,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WallpaperConfig {
    pub surface_size: UVec2,
    pub surface_format: wgpu::TextureFormat,
}

impl WallpaperConfig {
    pub const fn aspect_ratio(self) -> f32 {
        self.surface_size.y as f32 / self.surface_size.x as f32
    }
}
