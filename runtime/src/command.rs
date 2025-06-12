use bincode::{Decode, Encode};
use std::{ffi::CString, path::PathBuf};

#[derive(Encode, Decode, Debug, PartialEq, PartialOrd, Hash, Eq, Ord, Clone)]
pub enum DaemonCommand {
    SetVideo { path: CString },
    SetImage { path: PathBuf },
}
