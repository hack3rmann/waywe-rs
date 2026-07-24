use crate::WallpaperType;
use bincode::{Decode, Encode};
use display_error_chain::ErrorChainExt;
use miette::Diagnostic;
use std::{collections::HashMap, error::Error, path::PathBuf, time::Duration};
use thiserror::Error;

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
    Preview {
        ty: WallpaperType,
        path: PathBuf,
        width: u32,
        height: u32,
        time: Duration,
    },
    Pause {
        monitor: Option<String>,
        mode: PauseMode,
    },
    Current {
        monitor: Option<String>,
    },
    ConfigReload {
        path: Option<PathBuf>,
    },
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum DaemonResponse {
    WallpaperSet,
    PauseDone,
    Preview {
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    Current(HashMap<String, PathBuf>),
    ConfigReloaded,
}

#[derive(Encode, Decode, Error, Diagnostic, Debug, PartialEq, PartialOrd, Hash, Eq, Ord, Clone)]
pub enum DaemonError {
    #[error("{0}")]
    #[diagnostic(code(waywe::daemon::error))]
    Generic(String),
    #[error("image dimensions {width}x{height} are too big (max is {max_width}x{max_height})")]
    #[diagnostic(
        code(waywe::daemon::image_dimensions_too_big),
        help("reduce the required image dimensions")
    )]
    ImageDimensionsTooBig {
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },
}

impl DaemonError {
    pub fn from_generic(error: impl Error) -> Self {
        Self::Generic(error.chain().to_string())
    }
}

pub type DaemonResult = Result<DaemonResponse, DaemonError>;
