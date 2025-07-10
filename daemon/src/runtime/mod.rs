use crate::{almost::Almost, event_loop::Event, task_pool::TaskPool};
use bitflags::bitflags;
use gpu::Wgpu;
use ipc::Ipc;
use std::sync::Arc;
use timer::Timer;
use video::Video;
use wayland::Wayland;

pub mod gpu;
pub mod ipc;
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
    pub wayland: Wayland,
    pub ipc: Ipc,
    pub control_flow: ControlFlow,
    pub task_pool: TaskPool<Event>,
}

impl Runtime {
    pub fn new(wayland: Wayland, control_flow: ControlFlow, task_pool: TaskPool<Event>) -> Self {
        Self {
            timer: Timer::default(),
            wayland,
            wgpu: Almost::uninit(),
            video: Almost::uninit(),
            ipc: match Ipc::new() {
                Ok(ipc) => ipc,
                Err(error) => panic!("failed to initialize ipc: {error:?}"),
            },
            control_flow,
            task_pool,
        }
    }

    pub fn init_video(&mut self) {
        if Almost::is_uninit(&self.video) {
            Almost::init(&mut self.video, Video::default());
        }
    }

    pub async fn init_wgpu(&mut self) {
        if Almost::is_uninit(&self.wgpu) {
            Almost::init(&mut self.wgpu, Arc::new(Wgpu::new(&self.wayland).await));
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
