use crate::ipc;
use bincode::{Encode, error::EncodeError};
use rustix::{
    io::{self, Errno},
    net::{self, AddressFamily, SocketAddrUnix, SocketFlags, SocketType, sockopt::Timeout},
};
use smallvec::SmallVec;
use std::{
    marker::PhantomData,
    mem,
    os::{
        fd::{AsFd, AsRawFd, OwnedFd},
        unix::prelude::{BorrowedFd, RawFd},
    },
    time::Duration,
};
use thiserror::Error;

pub struct IpcClient<T> {
    fd: OwnedFd,
    _p: PhantomData<T>,
}

impl<T> AsFd for IpcClient<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl<T> AsRawFd for IpcClient<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl<T> IpcClient<T> {
    pub fn send(&self, value: T) -> Result<(), SendError>
    where
        T: Encode,
    {
        let mut buf = SmallVec::<[u8; ipc::BUFFER_SIZE]>::new_const();
        buf.extend_from_slice(&[0; mem::size_of::<u32>()]);

        let n_bytes = bincode::encode_into_std_write(value, &mut buf, bincode::config::standard())?;

        let size = bytemuck::from_bytes_mut::<u32>(&mut buf[..mem::size_of::<u32>()]);
        *size = n_bytes as u32;

        io::write(&self.fd, &buf)?;

        Ok(())
    }

    pub fn connect() -> Result<Self, Errno> {
        let socket = net::socket_with(
            AddressFamily::UNIX,
            SocketType::STREAM,
            SocketFlags::CLOEXEC,
            None,
        )?;

        let addr = SocketAddrUnix::new(ipc::socket_file()).expect("addr is correct");

        net::connect(&socket, &addr)?;

        const TIMEOUT: Duration = Duration::from_secs(5);
        net::sockopt::set_socket_timeout(&socket, Timeout::Recv, Some(TIMEOUT))?;

        Ok(Self {
            fd: socket,
            _p: PhantomData,
        })
    }
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error(transparent)]
    Os(#[from] Errno),
    #[error(transparent)]
    Encode(#[from] EncodeError),
}
