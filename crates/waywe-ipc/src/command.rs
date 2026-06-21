use crate::WallpaperType;
use bincode::{Decode, Encode};
use std::path::PathBuf;

#[derive(Encode, Decode, Debug, PartialEq, PartialOrd, Hash, Eq, Ord, Clone)]
pub enum DaemonCommand {
    Show {
        ty: WallpaperType,
        path: PathBuf,
        monitor: Option<String>,
    },
    Pause {
        monitor: Option<String>,
    },
}
