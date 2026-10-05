pub mod client;
pub mod server;

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use waywe_runtime_dir::runtime_dir;

pub const BUFFER_SIZE: usize = 256;

fn socket_file_from_env() -> PathBuf {
    runtime_dir().join("waywe.sock")
}

pub fn socket_file() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(socket_file_from_env)
}
