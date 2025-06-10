use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
use rustix::{
    io::{self, Errno},
    net::{self, AddressFamily, SocketAddrUnix, SocketFlags, SocketType, sockopt::Timeout},
};
use smallvec::{SmallVec, smallvec};
use std::{
    env, io::ErrorKind, marker::PhantomData, mem, os::fd::OwnedFd, path::Path, sync::OnceLock,
    time::Duration,
};
use thiserror::Error;
use tracing::warn;

pub const BUFFER_SIZE: usize = 256;

pub struct Client;
pub struct Server;

pub struct IpcSocket<Side, T> {
    fd: OwnedFd,
    _p: PhantomData<(Side, T)>,
}

impl<Side, T> IpcSocket<Side, T> {
    pub(crate) fn socket_file() -> String {
        let runtime = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
            let uid = rustix::process::getuid();
            format!("/run/user/{}", uid.as_raw())
        });

        let display = if let Ok(wayland_socket) = std::env::var("WAYLAND_DISPLAY") {
            let mut i = 0;
            // if WAYLAND_DISPLAY is a full path, use only its final component
            for (j, ch) in wayland_socket.bytes().enumerate().rev() {
                if ch == b'/' {
                    i = j + 1;
                    break;
                }
            }
            wayland_socket[i..].to_owned()
        } else {
            warn!("WAYLAND_DISPLAY variable not set. Defaulting to wayland-0");
            "wayland-0.sock".to_owned()
        };

        format!("{runtime}/waywe-{display}.sock")
    }

    #[must_use]
    pub fn path() -> &'static str {
        static PATH: OnceLock<String> = OnceLock::new();
        PATH.get_or_init(Self::socket_file)
    }

    pub fn send(&self, value: T) -> Result<(), SendError>
    where
        T: Encode,
    {
        let mut buf = SmallVec::<[u8; BUFFER_SIZE]>::new_const();
        buf.extend_from_slice(&[0; mem::size_of::<u32>()]);

        let n_bytes = bincode::encode_into_std_write(value, &mut buf, bincode::config::standard())?;

        let size = bytemuck::from_bytes_mut::<u32>(&mut buf[..mem::size_of::<u32>()]);
        *size = n_bytes as u32;

        io::write(&self.fd, &buf)?;

        Ok(())
    }

    pub fn try_recv(&self) -> Result<T, RecvError>
    where
        T: Decode<()>,
    {
        let fd = match net::accept(&self.fd) {
            Ok(fd) => fd,
            Err(Errno::INTR | Errno::WOULDBLOCK) => return Err(RecvError::Empty),
            Err(other) => return Err(RecvError::Os(other)),
        };

        let mut length = 0_u32;
        match io::read(&fd, bytemuck::bytes_of_mut(&mut length)) {
            Ok(n_bytes) => assert_eq!(n_bytes, mem::size_of_val(&length)),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock) => {
                return Err(RecvError::Empty);
            }
            Err(error) => return Err(RecvError::Os(error)),
        }

        let mut buf: SmallVec<[u8; BUFFER_SIZE]> = smallvec![0; length as usize];
        io::read(&fd, &mut buf)?;

        let (value, _n_bytes) = bincode::decode_from_slice(&buf, bincode::config::standard())?;

        Ok(value)
    }
}

impl<T> IpcSocket<Client, T> {
    pub fn connect() -> Result<Self, Errno> {
        let socket = net::socket_with(
            AddressFamily::UNIX,
            SocketType::STREAM,
            SocketFlags::CLOEXEC,
            None,
        )?;

        let addr = SocketAddrUnix::new(Self::path()).expect("addr is correct");

        net::connect_unix(&socket, &addr)?;

        const TIMEOUT: Duration = Duration::from_secs(5);
        net::sockopt::set_socket_timeout(&socket, Timeout::Recv, Some(TIMEOUT))?;

        Ok(Self {
            fd: socket,
            _p: PhantomData,
        })
    }
}

impl<T> IpcSocket<Server, T> {
    pub fn server() -> Result<Self, Errno> {
        _ = rustix::fs::unlink(Self::path());
        let path = Path::new(Self::path());

        if let Some(dir) = path.parent() {
            if !dir.exists() {
                std::fs::create_dir_all(dir).unwrap();
            }
        }

        let addr = SocketAddrUnix::new(Self::path()).expect("addr is correct");

        let socket = net::socket_with(
            net::AddressFamily::UNIX,
            net::SocketType::STREAM,
            net::SocketFlags::CLOEXEC | net::SocketFlags::NONBLOCK,
            None,
        )?;

        net::bind_unix(&socket, &addr)?;
        net::listen(&socket, 0)?;

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

#[derive(Debug, Error)]
pub enum RecvError {
    #[error("socket is empty")]
    Empty,
    #[error(transparent)]
    Os(#[from] Errno),
    #[error(transparent)]
    Decode(#[from] DecodeError),
}
