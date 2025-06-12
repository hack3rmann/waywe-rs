pub mod image;
pub mod transition;
pub mod video;

use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures},
};
use std::any::Any;

pub trait Wallpaper: Any + Send + Sync {
    fn frame(
        &mut self,
        runtime: &Runtime,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
    ) -> Result<FrameInfo, FrameError>;
}
static_assertions::assert_obj_safe!(Wallpaper);

pub type DynWallpaper = Box<dyn Wallpaper>;

pub trait RequiresFeatures {
    const REQUIRED_FEATURES: RuntimeFeatures;
}
