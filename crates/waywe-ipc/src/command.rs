use crate::WallpaperType;
use bincode::{Decode, Encode};
use std::path::PathBuf;

#[derive(Encode, Decode, Default, Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum PauseMode {
    #[default]
    Toggle,
    On,
    Off,
}

impl PauseMode {
    pub const fn from_on_off(on: bool, off: bool) -> Self {
        let mut res = Self::Toggle;

        if on {
            res = Self::On;
        }

        if off {
            res = Self::Off
        }

        res
    }
}

#[derive(Encode, Decode, Debug, PartialEq, PartialOrd, Hash, Eq, Ord, Clone)]
pub enum DaemonCommand {
    Show {
        ty: WallpaperType,
        path: PathBuf,
        monitor: Option<String>,
    },
    Pause {
        monitor: Option<String>,
        mode: PauseMode,
    },
}
