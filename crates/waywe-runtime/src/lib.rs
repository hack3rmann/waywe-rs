use crate::wayland::MonitorId;
use glam::UVec2;
use gpu::Wgpu;
use std::sync::{Arc, Once};
use task_pool::TaskPool;
use thiserror::Error;
use timer::Timer;
use wayland::Wayland;
use waywe_ipc::{DaemonCommand, IpcServer, ipc::server::CreateServerError};

pub mod app;
pub mod effects;
pub mod event;
pub mod frame;
pub mod gpu;
pub mod shaders;
pub mod task_pool;
pub mod timer;
pub mod wayland;

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum ControlFlow {
    // TODO(hack3rmann): add optional timeout here
    #[default]
    Idle,
    Busy,
    ShouldStop,
}

impl ControlFlow {
    pub fn idle(&mut self) {
        *self = Self::Idle;
    }

    pub fn busy(&mut self) {
        *self = Self::Busy;
    }

    pub fn stop(&mut self) {
        *self = Self::ShouldStop;
    }
}

#[derive(Debug, Error)]
pub enum CreateRuntimeError {
    #[error(transparent)]
    Ipc(#[from] CreateServerError),
}

pub struct Runtime {
    pub timer: Timer,
    pub wgpu: Arc<Wgpu>,
    pub wayland: Arc<Wayland>,
    pub ipc: IpcServer<DaemonCommand>,
    pub control_flow: ControlFlow,
    pub task_pool: TaskPool,
}

impl Runtime {
    pub fn new(
        wayland: Wayland,
        control_flow: ControlFlow,
        task_pool: TaskPool,
    ) -> Result<Self, CreateRuntimeError> {
        static VIDEO_ONCE: Once = Once::new();
        VIDEO_ONCE.call_once(video::init);

        Ok(Self {
            timer: Timer::default(),
            wgpu: Arc::default(),
            wayland: Arc::new(wayland),
            ipc: IpcServer::new()?,
            control_flow,
            task_pool,
        })
    }

    pub fn wallpaper_config(&self, monitor_id: MonitorId) -> WallpaperConfig {
        let surface_size = {
            let monitors = self.wayland.client_state.monitors.read().unwrap();
            monitors[&monitor_id].size
        };
        let surface_format = {
            let surfaces = self.wgpu.surfaces.read().unwrap();
            surfaces[&monitor_id].format
        };

        WallpaperConfig {
            surface_size,
            surface_format,
        }
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
