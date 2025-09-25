use crate::task_pool::TaskPool;
use bitflags::bitflags;
use for_sure::prelude::*;
use gpu::Wgpu;
use runtime::{DaemonCommand, IpcSocket, ipc::Server};
use std::sync::Arc;
use timer::Timer;
use video::Video;
use wayland::Wayland;

pub mod gpu;
pub mod shaders;
pub mod timer;
pub mod video;
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

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Default, Hash)]
    pub struct RuntimeFeatures: u32 {
        const GPU = 0x1;
        const VIDEO = 0x2;
    }
}

pub struct Runtime {
    pub timer: Timer,
    pub video: Almost<Video>,
    pub wgpu: Almost<Arc<Wgpu>>,
    pub wayland: Arc<Wayland>,
    pub ipc: IpcSocket<Server, DaemonCommand>,
    pub control_flow: ControlFlow,
    pub task_pool: TaskPool,
}

impl Runtime {
    pub fn new(wayland: Wayland, control_flow: ControlFlow, task_pool: TaskPool) -> Self {
        Self {
            timer: Timer::default(),
            wayland: Arc::new(wayland),
            wgpu: Nil,
            video: Nil,
            ipc: match IpcSocket::server() {
                Ok(ipc) => ipc,
                Err(error) => panic!("failed to initialize ipc: {error:?}"),
            },
            control_flow,
            task_pool,
        }
    }

    pub fn init_video(&mut self) {
        if Almost::is_nil(&self.video) {
            self.video = Value(Video::default());
        }
    }

    pub async fn init_wgpu(&mut self) {
        if Almost::is_nil(&self.wgpu) {
            self.wgpu = Value(Arc::new(Wgpu::new(&self.wayland).await));
        }
    }

    pub async fn enable(&mut self, features: RuntimeFeatures) {
        if features.contains(RuntimeFeatures::VIDEO) {
            self.init_video();
        }

        if features.contains(RuntimeFeatures::GPU) {
            self.init_wgpu().await;
        }
    }
}
