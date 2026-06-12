use bincode::{Decode, Encode};
use rustix::{
    fs::{self, Mode},
    io::Errno,
};
use std::{
    fs::File,
    os::{fd::AsFd, unix::prelude::BorrowedFd},
    path::{Path, PathBuf},
};
use thiserror::Error;
use uuid::Uuid;

pub const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

#[derive(Debug, Clone, Encode, Decode, Error)]
pub enum DaemonSetupError {
    #[error("waywe-daemon panicked:\n{message}")]
    Panicked { message: String },
    #[error("failed to acquire daemon file lock")]
    DaemonFileLock,
}

pub type DaemonSetupResult = Result<(), DaemonSetupError>;

pub struct SetupPipe {
    pub path: PathBuf,
}

impl SetupPipe {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let base_dir = base_dir.as_ref();

        std::fs::create_dir_all(base_dir).unwrap();

        let mut path = PathBuf::new();

        loop {
            let uuid = Uuid::now_v7();

            path.clear();
            path.push(base_dir);
            path.push(format!("fifo-{}", uuid.hyphenated()));

            match fs::mkfifoat(AtFdCwd, &path, Mode::RUSR | Mode::WUSR) {
                Ok(()) => return Self { path },
                Err(Errno::EXIST) => continue,
                Err(error) => panic!("failed to create fifo: {error}"),
            }
        }
    }

    pub fn read(&self) -> File {
        File::options().read(true).open(&self.path).unwrap()
    }
}

impl Drop for SetupPipe {
    fn drop(&mut self) {
        _ = std::fs::remove_file(&self.path);
    }
}

struct AtFdCwd;

impl AsFd for AtFdCwd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(libc::AT_FDCWD) }
    }
}
