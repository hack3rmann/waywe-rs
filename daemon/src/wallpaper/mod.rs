pub mod image;
pub mod video;

use crate::{
    event_loop::{FrameError, FrameInfo},
    runtime::{Runtime, RuntimeFeatures},
};

pub trait Wallpaper: Send + Sync + 'static {
    fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError>;
}
static_assertions::assert_obj_safe!(Wallpaper);

pub trait RequiresFeatures {
    const REQUIRED_FEATURES: RuntimeFeatures;
}
