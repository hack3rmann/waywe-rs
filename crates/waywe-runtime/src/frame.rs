use std::time::Duration;
use thiserror::Error;
use video::RatioI32;

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInfo {
    pub target_frame_time: Option<Duration>,
}

impl FrameInfo {
    pub const MAX_FPS: Duration = RatioI32::new(1, 60).unwrap().to_duration_seconds();

    pub const fn new_60_fps() -> Self {
        Self {
            target_frame_time: Some(Self::MAX_FPS),
        }
    }

    pub fn min_or_60_fps(self, other: Self) -> Self {
        match (self.target_frame_time, other.target_frame_time) {
            (Some(time1), Some(time2)) => Self {
                target_frame_time: Some(time1.min(time2).min(Self::MAX_FPS)),
            },
            (Some(time), None) | (None, Some(time)) => Self {
                target_frame_time: Some(time.min(Self::MAX_FPS)),
            },
            (None, None) => Self {
                target_frame_time: None,
            },
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum FrameError {
    #[error("event loop stop requested")]
    StopRequested,
    #[error("frame skipped due to error")]
    Skip,
    #[error("no work to do")]
    NoWorkToDo,
}
