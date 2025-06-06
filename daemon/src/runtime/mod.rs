use crate::almost::Almost;
use gpu::Wgpu;
use timer::Timer;
use video::Video;
use wayland::Wayland;

pub mod gpu;
pub mod timer;
pub mod video;
pub mod wayland;

pub struct Runtime {
    pub timer: Timer,
    pub video: Almost<Video>,
    pub wgpu: Almost<Wgpu>,
    pub wayland: Wayland,
}

impl Runtime {
    pub fn new(wayland: Wayland) -> Self {
        Self {
            timer: Timer::default(),
            wayland,
            wgpu: Almost::uninit(),
            video: Almost::uninit(),
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
