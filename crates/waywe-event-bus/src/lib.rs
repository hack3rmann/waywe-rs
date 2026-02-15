use crate::event::{EventHeader, ExternalEvent};
use bincode::error::DecodeError;
use box_into_inner::IntoInner as _;
use bytemuck::Zeroable;
use rustix::{
    fs::OFlags,
    io::Errno,
    net::{self, AddressFamily, RecvFlags, SocketAddrUnix, SocketFlags, SocketType},
};
use smallvec::{SmallVec, smallvec};
use std::{
    any::{Any, TypeId},
    io::{self, ErrorKind, PipeReader, PipeWriter, Read as _},
    mem,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
    path::Path,
    sync::mpsc::{self, Receiver, Sender},
};
use thiserror::Error;
use tracing::{debug, error};

pub mod event;
mod path;

pub struct ExternalEventBus {
    server_sock: OwnedFd,
    server_connections: SmallVec<[OwnedFd; 4]>,
}

impl ExternalEventBus {
    pub fn new() -> Result<Self, Errno> {
        let path = Path::new(path::get_socket_path());

        if let Some(dir) = path.parent()
            && !dir.exists()
        {
            std::fs::create_dir_all(dir).unwrap();
        }

        debug!(?path, "creating daemon socket");

        let addr = SocketAddrUnix::new(path::get_socket_path()).expect("addr is correct");

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
                        path = path::get_socket_path(),
                        "socket address already in use, trying to remove",
                    );
                    rustix::fs::unlink(path::get_socket_path())?
                }
                Err(other) => return Err(other),
            }
        }

        net::listen(&socket, 0)?;

        Ok(Self {
            server_sock: socket,
            server_connections: SmallVec::new_const(),
        })
    }

    pub fn accept_all(&mut self) -> Result<(), Errno> {
        loop {
            let fd = match net::accept_with(&self.server_sock, SocketFlags::empty()) {
                Ok(fd) => fd,
                Err(Errno::INTR) => continue,
                Err(Errno::WOULDBLOCK) => return Ok(()),
                Err(other) => return Err(other),
            };

            self.server_connections.push(fd);
        }
    }

    pub fn try_recv(&self) -> Result<ExternalEvent, Errno> {
        for fd in &self.server_connections {
            let mut header = EventHeader::zeroed();

            match net::recv(fd, bytemuck::bytes_of_mut(&mut header), RecvFlags::DONTWAIT) {
                Ok(n_bytes) => assert_eq!(
                    n_bytes,
                    mem::size_of_val(&header),
                    "the message's header isn't fully received, this is a bug",
                ),
                Err(Errno::WOULDBLOCK) => continue,
                Err(error) => return Err(error),
            }

            let mut data = smallvec![0; header.n_data_bytes as usize];
            let n_bytes = net::recv(fd, &mut data, RecvFlags::WAITALL)?;

            assert_eq!(
                n_bytes, header.n_data_bytes as usize,
                "should receive the message with the exact size specifiend in the header, this is a bug"
            );

            return Ok(ExternalEvent { header, data });
        }

        Err(Errno::AGAIN)
    }
}

pub trait TryReplicate: Any + Send {
    fn try_replicate(&self) -> Option<Box<dyn TryReplicate>> {
        None
    }
}

impl dyn TryReplicate {
    pub fn downcast<T: Any>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if TypeId::of::<T>() == self.as_ref().type_id() {
            Ok((self as Box<dyn Any>).downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }
}

impl<T: Clone + Any + Send> TryReplicate for T {
    fn try_replicate(&self) -> Option<Box<dyn TryReplicate>> {
        Some(Box::new(self.clone()))
    }
}

pub struct InternalEvent(Option<Box<dyn TryReplicate>>);

impl InternalEvent {
    pub fn underlying_type(&self) -> Option<TypeId> {
        self.0.as_ref().map(Box::as_ref).map(|e| e.type_id())
    }

    /// # Safety
    ///
    /// - event should contain a value
    /// - underlying type should be exactly `E`
    pub unsafe fn downcast_unchecked<E: IntoInternalEvent>(self) -> E {
        let any = unsafe { self.0.unwrap_unchecked() };
        let boxed = unsafe { any.downcast::<E>().unwrap_unchecked() };
        boxed.into_inner()
    }

    pub async fn handle<T: IntoInternalEvent>(&mut self, f: impl AsyncFnOnce(T)) {
        // Try to replicate the event. Take the event if could not replicate
        let replicated = self.0.as_deref().and_then(TryReplicate::try_replicate);

        let Some(any_value) = replicated.or_else(|| self.0.take()) else {
            return;
        };

        let boxed_value = match any_value.downcast::<T>() {
            Ok(value) => value,
            Err(other) => {
                self.0 = Some(other);
                return;
            }
        };

        let value = boxed_value.into_inner();
        f(value).await;
    }
}

pub trait IntoInternalEvent: TryReplicate {
    fn into_event(self) -> InternalEvent
    where
        Self: Sized;
}

impl<T: TryReplicate> IntoInternalEvent for T {
    fn into_event(self) -> InternalEvent {
        InternalEvent(Some(Box::new(self)))
    }
}

pub struct InternalEventReceiver {
    reader: PipeReader,
    receiver: Receiver<InternalEvent>,
}

impl InternalEventReceiver {
    pub fn recv(&mut self) -> Result<InternalEvent, RecvError> {
        let mut buffer = [0_u8];

        self.reader.read_exact(&mut buffer)?;
        assert_eq!(buffer[0], 0, "event type should be 0, this is a bug");

        Ok(self.receiver.recv()?)
    }

    pub fn try_recv(&mut self) -> Result<InternalEvent, RecvError> {
        let mut buffer = [0_u8];

        match self.reader.read(&mut buffer) {
            Ok(1) => {}
            Ok(..) => return Err(RecvError::WouldBlock),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                return Err(RecvError::WouldBlock);
            }
            Err(error) => return Err(error.into()),
        }

        assert_eq!(buffer[0], 0, "event type should be 0, this is a bug");

        Ok(self.receiver.try_recv()?)
    }
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error(transparent)]
    Os(#[from] Errno),
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Recv(#[from] mpsc::RecvError),
    #[error(transparent)]
    TryRecv(#[from] mpsc::TryRecvError),
    #[error("would block the execution path")]
    WouldBlock,
}

pub struct InternalEventSender {
    writer: PipeWriter,
    sender: Sender<InternalEvent>,
}

impl InternalEventSender {
    pub fn try_clone(&self) -> Result<Self, io::Error> {
        Ok(Self {
            writer: self.writer.try_clone()?,
            sender: self.sender.clone(),
        })
    }
}

pub struct InternalEventBus {
    receiver: InternalEventReceiver,
    sender: InternalEventSender,
}

impl InternalEventBus {
    pub fn new() -> Result<Self, io::Error> {
        let (reader, writer) = io::pipe()?;

        let read_flags = rustix::fs::fcntl_getfl(&reader).unwrap();
        rustix::fs::fcntl_setfl(&reader, read_flags | OFlags::NONBLOCK).unwrap();

        let write_flags = rustix::fs::fcntl_getfl(&writer).unwrap();
        rustix::fs::fcntl_setfl(&writer, write_flags | OFlags::NONBLOCK).unwrap();

        let (sender, receiver) = mpsc::channel();

        Ok(Self {
            receiver: InternalEventReceiver { receiver, reader },
            sender: InternalEventSender { sender, writer },
        })
    }

    pub fn make_sender(&self) -> InternalEventSender {
        self.sender.try_clone().unwrap()
    }
}

impl AsFd for InternalEventBus {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.receiver.reader.as_fd()
    }
}
