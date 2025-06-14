use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
use rustix::{
    event::epoll::{self, EventData, EventFlags, EventVec},
    io::{self, Errno},
    net::{
        self, AddressFamily, RecvFlags, SocketAddrUnix, SocketFlags, SocketType, sockopt::Timeout,
    },
};
use smallvec::{SmallVec, smallvec};
use std::{
    env,
    marker::PhantomData,
    mem,
    os::fd::{AsRawFd, OwnedFd},
    path::Path,
    sync::OnceLock,
    time::{Duration, Instant},
};
use thiserror::Error;
use tracing::{debug, error, warn};

pub const BUFFER_SIZE: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Side {
    #[default]
    Client,
    Server,
}

pub trait SocketSide {
    const SIDE: Side;
}

pub struct Client;

impl SocketSide for Client {
    const SIDE: Side = Side::Client;
}

pub struct Server;

impl SocketSide for Server {
    const SIDE: Side = Side::Server;
}

pub struct IpcSocket<Side: SocketSide, T> {
    fd: OwnedFd,
    _p: PhantomData<(Side, T)>,
}

impl<S: SocketSide, T> IpcSocket<S, T> {
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

    pub fn recv(
        &self,
        mode: RecvMode,
        timeout: Option<Duration>,
    ) -> Result<SmallVec<[T; 1]>, RecvError>
    where
        T: Decode<()>,
    {
        match mode {
            RecvMode::Blocking => self.blocking_recv(timeout),
            RecvMode::NonBlocking => self.nonblocking_recv().map(|value| smallvec![value]),
        }
    }

    pub fn blocking_recv(&self, timeout: Option<Duration>) -> Result<SmallVec<[T; 1]>, RecvError>
    where
        T: Decode<()>,
    {
        let epoll_fd = epoll::create(epoll::CreateFlags::CLOEXEC)?;

        epoll::add(
            &epoll_fd,
            &self.fd,
            EventData::new_u64(self.fd.as_raw_fd() as u64),
            EventFlags::IN,
        )?;

        let wait_time = timeout
            .and_then(|d| i32::try_from(d.as_millis()).ok())
            .unwrap_or(-1);

        let mut events = EventVec::with_capacity(1);

        let start = Instant::now();

        // TODO(hack3rmann): sleep on both wayland and our sockets
        match epoll::wait(&epoll_fd, &mut events, wait_time) {
            Ok(()) => {}
            Err(Errno::INTR) => return Err(RecvError::Empty),
            Err(error) => return Err(RecvError::Os(error)),
        }

        if let Some(duration) = timeout {
            if start.elapsed() >= duration {
                return Err(RecvError::Timeout);
            }
        }

        events
            .iter()
            .filter(|event| {
                // NOTE(hack3rmann): read from `event.flags` is unaligned,
                // therefore we must make a copy into a local variable
                let flags = event.flags;
                flags.contains(EventFlags::IN)
            })
            .map(|_| self.nonblocking_recv())
            // collecting into result lazily
            .collect()
    }

    pub fn nonblocking_recv(&self) -> Result<T, RecvError>
    where
        T: Decode<()>,
    {
        let fd = match net::accept_with(&self.fd, SocketFlags::empty()) {
            Ok(fd) => fd,
            Err(Errno::INTR | Errno::WOULDBLOCK) => return Err(RecvError::Empty),
            Err(other) => return Err(RecvError::Os(other)),
        };

        let mut length = 0_u32;

        match net::recv(
            &fd,
            bytemuck::bytes_of_mut(&mut length),
            RecvFlags::DONTWAIT,
        ) {
            Ok(n_bytes) => assert_eq!(n_bytes, mem::size_of_val(&length)),
            Err(Errno::WOULDBLOCK) => return Err(RecvError::Empty),
            Err(error) => return Err(RecvError::Os(error)),
        }

        let mut buf: SmallVec<[u8; BUFFER_SIZE]> = smallvec![0; length as usize];
        net::recv(&fd, &mut buf, RecvFlags::WAITALL)?;

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
        let path = Path::new(Self::path());

        if let Some(dir) = path.parent() {
            if !dir.exists() {
                std::fs::create_dir_all(dir).unwrap();
            }
        }

        debug!(?path, "creating daemon socket");

        let addr = SocketAddrUnix::new(Self::path()).expect("addr is correct");

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
                    error!(
                        path = Self::path(),
                        "socket address already in use, trying to remove",
                    );
                    rustix::fs::unlink(Self::path())?
                }
                Err(other) => return Err(other),
            }
        }

        net::listen(&socket, 0)?;

        Ok(Self {
            fd: socket,
            _p: PhantomData,
        })
    }
}

impl<S: SocketSide, T> Drop for IpcSocket<S, T> {
    fn drop(&mut self) {
        if S::SIDE == Side::Server {
            debug!(path = Self::path(), "removing daemon socket");
            _ = rustix::fs::unlink(Self::path());
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum RecvMode {
    #[default]
    NonBlocking,
    Blocking,
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
    #[error("timeout has reached")]
    Timeout,
    #[error(transparent)]
    Os(#[from] Errno),
    #[error(transparent)]
    Decode(#[from] DecodeError),
}
