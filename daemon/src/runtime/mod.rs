use crate::almost::Almost;
use gpu::Wgpu;
use ipc::Ipc;
use timer::Timer;
use video::Video;
use wayland::Wayland;

pub mod gpu;
pub mod ipc;
pub mod timer;
pub mod video;
pub mod wayland;

pub struct Runtime {
    pub timer: Timer,
    pub video: Almost<Video>,
    pub wgpu: Almost<Wgpu>,
    pub wayland: Wayland,
    pub ipc: Ipc,
}

impl Runtime {
    pub fn new(wayland: Wayland) -> Self {
        Self {
            timer: Timer::default(),
            wayland,
            wgpu: Almost::uninit(),
            video: Almost::uninit(),
            ipc: match Ipc::new() {
                Ok(ipc) => ipc,
                Err(error) => panic!("failed to initialize ipc: {error:?}"),
            },
        }
    }

    pub fn init_video(&mut self) {
        if Almost::is_uninit(&self.video) {
            Almost::init(&mut self.video, Video::default());
        }
    }

    pub async fn init_wgpu(&mut self) {
        if Almost::is_uninit(&self.wgpu) {
            Almost::init(&mut self.wgpu, Wgpu::new(&self.wayland).await);
        }
    }
}
