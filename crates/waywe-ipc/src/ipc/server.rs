use crate::ipc;
use bincode::{Decode, error::DecodeError};
use rustix::{
    io::Errno,
    net::{self, AddressFamily, RecvFlags, SocketAddrUnix, SocketFlags, SocketType},
};
use smallvec::{SmallVec, smallvec};
use std::{
    fs::{File, TryLockError},
    io,
    marker::PhantomData,
    mem,
    os::{
        fd::{AsFd, AsRawFd, OwnedFd},
        unix::prelude::{BorrowedFd, RawFd},
    },
    path::Path,
};
use thiserror::Error;
use tracing::{debug, warn};

pub struct IpcServer<T> {
    _lock_file: File,
    fd: OwnedFd,
    _p: PhantomData<T>,
}

impl<T> AsFd for IpcServer<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl<T> AsRawFd for IpcServer<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl<T> IpcServer<T> {
    pub fn try_recv(&self) -> Result<T, RecvError>
    where
        T: Decode<()>,
    {
        let fd = match net::accept_with(&self.fd, SocketFlags::empty()) {
            Ok(fd) => fd,
            Err(Errno::INTR | Errno::WOULDBLOCK) => return Err(RecvError::Empty),
            Err(other) => return Err(RecvError::Os(other)),
        };

        const MAX_LENGTH: u32 = 4096;
        let mut length = 0_u32;

        match net::recv(&fd, bytemuck::bytes_of_mut(&mut length), RecvFlags::WAITALL) {
            Ok(n_bytes) => assert_eq!(n_bytes, mem::size_of_val(&length)),
            Err(error) => return Err(RecvError::Os(error)),
        }

        assert!(length <= MAX_LENGTH, "too large message, unbelivable");

        let mut buf: SmallVec<[u8; ipc::BUFFER_SIZE]> = smallvec![0; length as usize];
        net::recv(&fd, &mut buf, RecvFlags::WAITALL)?;

        let (value, _n_bytes) = bincode::decode_from_slice(&buf, bincode::config::standard())?;

        Ok(value)
    }

    fn acquire_file_lock() -> Result<File, TryLockError> {
        let parent = Path::new(ipc::socket_file()).parent().unwrap();

        std::fs::create_dir_all(parent).map_err(TryLockError::Error)?;

        let lock_file_path = parent.join("waywe-daemon.lock");
        let lock_file = File::options()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_file_path)
            .map_err(TryLockError::Error)?;

        lock_file.try_lock()?;

        Ok(lock_file)
    }

    pub fn new() -> Result<Self, CreateServerError> {
        let lock_file = Self::acquire_file_lock()?;
        let path = Path::new(ipc::socket_file());

        if let Some(dir) = path.parent()
            && !dir.exists()
        {
            std::fs::create_dir_all(dir)?;
        }

        debug!(?path, "creating daemon socket");

        let addr = SocketAddrUnix::new(ipc::socket_file()).expect("addr is correct");

        let socket = net::socket_with(
            AddressFamily::UNIX,
            SocketType::STREAM,
            SocketFlags::CLOEXEC | SocketFlags::NONBLOCK,
            None,
        )?;

        loop {
            match net::bind_unix(&socket, &addr) {
                Ok(()) => break,
                Err(Errno::ADDRINUSE) => {
                    warn!(
                        path = ipc::socket_file(),
                        "socket address already in use, trying to remove",
                    );
                    // NOTE(hack3rmann): we're holding `waywe-daemon.lock` so no other daemon
                    // is using the socket right now
                    rustix::fs::unlink(ipc::socket_file())?
                }
                Err(other) => return Err(CreateServerError::Os(other)),
            }
        }

        net::listen(&socket, 128)?;

        Ok(Self {
            _lock_file: lock_file,
            fd: socket,
            _p: PhantomData,
        })
    }
}

impl<T> Drop for IpcServer<T> {
    fn drop(&mut self) {
        debug!(path = ipc::socket_file(), "removing daemon socket");
        _ = rustix::fs::unlink(ipc::socket_file());
    }
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error("socket is empty")]
    Empty,
    #[error(transparent)]
    Os(#[from] Errno),
    #[error(transparent)]
    Decode(#[from] DecodeError),
}

#[derive(Debug, Error)]
pub enum CreateServerError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Os(#[from] Errno),
    #[error("failed to acquire the daemon file lock")]
    AcquireFileLock,
}

impl From<TryLockError> for CreateServerError {
    fn from(value: TryLockError) -> Self {
        match value {
            TryLockError::Error(error) => Self::Io(error),
            TryLockError::WouldBlock => Self::AcquireFileLock,
        }
    }
}
