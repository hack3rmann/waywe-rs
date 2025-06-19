use bincode::{Decode, Encode};
use std::path::PathBuf;

#[derive(Encode, Decode, Debug, PartialEq, PartialOrd, Hash, Eq, Ord, Clone)]
pub enum DaemonCommand {
    SetVideo { path: PathBuf },
    SetImage { path: PathBuf },
}
