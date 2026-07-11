use crate::ipc;
use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
use rustix::{
    io::{self, Errno},
    net::{self, AddressFamily, SocketAddrUnix, SocketFlags, SocketType},
};
use smallvec::{SmallVec, smallvec};
use std::{
    marker::PhantomData,
    mem,
    os::{
        fd::{AsFd, AsRawFd, OwnedFd},
        unix::prelude::{BorrowedFd, RawFd},
    },
};
use thiserror::Error;

pub struct IpcClient<T, R> {
    fd: OwnedFd,
    _p: PhantomData<(T, R)>,
}

impl<T, R> AsFd for IpcClient<T, R> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl<T, R> AsRawFd for IpcClient<T, R> {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl<T, R> IpcClient<T, R> {
    pub fn send(&self, value: T) -> Result<(), ClientError>
    where
        T: Encode,
    {
        let mut buf = SmallVec::<[u8; ipc::BUFFER_SIZE]>::new_const();
        buf.extend_from_slice(&[0; mem::size_of::<u32>()]);

        let n_bytes = bincode::encode_into_std_write(value, &mut buf, bincode::config::standard())?;

        let size = bytemuck::from_bytes_mut::<u32>(&mut buf[..mem::size_of::<u32>()]);
        *size = n_bytes as u32;

        io::write(self, &buf)?;

        Ok(())
    }

    pub fn recv(&self) -> Result<R, ClientError>
    where
        R: Decode<()>,
    {
        let mut n_bytes = 0_u32;

        let n_bytes_read = io::read(self, bytemuck::bytes_of_mut(&mut n_bytes))?;
        assert_eq!(n_bytes_read, mem::size_of_val(&n_bytes));

        let mut buf: SmallVec<[u8; ipc::BUFFER_SIZE]> = smallvec![0; n_bytes as usize];

        let n_bytes_read = match fd_read_all(self, &mut buf[..]) {
            Ok(n_bytes) | Err((n_bytes, Errno::CONNRESET)) => n_bytes,
            Err((_, error)) => return Err(ClientError::Os(error)),
        };
        assert_eq!(n_bytes_read, n_bytes as usize);

        let (response, n_bytes_decoded) =
            bincode::decode_from_slice(&buf, bincode::config::standard())?;
        assert_eq!(n_bytes_decoded, n_bytes_read);

        Ok(response)
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

        Ok(Self {
            fd: socket,
            _p: PhantomData,
        })
    }
}

fn fd_read_all(fd: impl AsFd, buf: &mut [u8]) -> Result<usize, (usize, Errno)> {
    let fd = fd.as_fd();
    let mut total_read = 0;

    while total_read < buf.len() {
        match io::read(fd, &mut buf[total_read..]) {
            Ok(0) => break,
            Ok(n_read) => total_read += n_read,
            Err(error) => return Err((total_read, error)),
        }
    }

    Ok(total_read)
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    Os(#[from] Errno),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Decode(#[from] DecodeError),
}
