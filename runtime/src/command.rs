use bincode::{Decode, Encode};
use std::path::PathBuf;

#[derive(Encode, Decode, Debug, PartialEq, PartialOrd, Hash, Eq, Ord, Clone)]
pub enum DaemonCommand {
    SetVideo {
        path: PathBuf,
        monitor: Option<String>,
    },
    SetImage {
        path: PathBuf,
        monitor: Option<String>,
    },
    SetScene {
        monitor: Option<String>,
    },
    Pause {
        monitor: Option<String>,
    },
}
