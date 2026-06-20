use crate::wayland::{MonitorId, Wayland};
use glam::UVec2;
use gpu::Wgpu;
use std::sync::{Arc, Once};
use task_pool::TaskPool;
use timer::Timer;

pub mod app;
pub mod effects;
pub mod event;
pub mod frame;
pub mod gpu;
pub mod shaders;
pub mod task_pool;
pub mod timer;
pub mod wayland;

pub struct Runtime {
    pub timer: Timer,
    pub wgpu: Arc<Wgpu>,
    pub wayland: Wayland,
    pub task_pool: TaskPool,
}

impl Runtime {
    pub fn new(wayland: Wayland, task_pool: TaskPool) -> Self {
        static VIDEO_ONCE: Once = Once::new();
        VIDEO_ONCE.call_once(video::init);

        Self {
            timer: Timer::default(),
            wgpu: Arc::default(),
            wayland,
            task_pool,
        }
    }

    pub fn wallpaper_config(&self, monitor_id: MonitorId) -> Option<WallpaperConfig> {
        let surface_size = {
            let monitors = self.wayland.client_state.monitors.read().unwrap();
            monitors.get(&monitor_id)?.size
        };
        let surface_format = {
            let surfaces = self.wgpu.surfaces.read().unwrap();
            surfaces.get(&monitor_id)?.format
        };

        Some(WallpaperConfig {
            surface_size,
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
