use crate::box_ext::BoxExt;
use bytemuck::{Contiguous, NoUninit};
use rustix::fs::OFlags;
use std::{
    any::Any,
    io::{self, ErrorKind, PipeReader, PipeWriter, Read as _, Write as _},
    os::fd::{AsFd, BorrowedFd},
    sync::mpsc::{self, Receiver, RecvError, Sender, TryRecvError},
};
use thiserror::Error;

pub struct Event(Option<Box<dyn Any + Send + 'static>>);

impl Event {
    pub async fn handle<T: Any + Send + 'static>(&mut self, f: impl AsyncFnOnce(T)) {
        let Some(any_value) = self.0.take() else {
            return;
        };

        let boxed_value = match any_value.downcast::<T>() {
            Ok(value) => value,
            Err(other) => {
                self.0 = Some(other);
                return;
            }
        };

        let value = BoxExt::into_inner(boxed_value);
        f(value).await;
    }
}

pub trait IntoEvent: Any + Send + 'static {
    fn into_event(self) -> Event
    where
        Self: Sized;
}

impl<T: Any + Send + 'static> IntoEvent for T {
    fn into_event(self) -> Event {
        Event(Some(Box::new(self)))
    }
}

#[repr(u8)]
#[derive(
    Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit, Contiguous,
)]
pub enum EventType {
    #[default]
    Any = 0,
}

#[derive(Debug)]
pub struct EventReceiver {
    reader: PipeReader,
    writer: PipeWriter,
    receiver: Receiver<Event>,
    sender: Sender<Event>,
}

impl EventReceiver {
    pub fn new() -> Result<Self, io::Error> {
        let (reader, writer) = io::pipe()?;

        let read_flags = rustix::fs::fcntl_getfl(&reader).unwrap();
        rustix::fs::fcntl_setfl(&reader, read_flags | OFlags::NONBLOCK).unwrap();

        let write_flags = rustix::fs::fcntl_getfl(&writer).unwrap();
        rustix::fs::fcntl_setfl(&writer, write_flags | OFlags::NONBLOCK).unwrap();

        let (sender, receiver) = mpsc::channel();

        Ok(Self {
            reader,
            writer,
            sender,
            receiver,
        })
    }

    pub fn make_emitter(&self) -> Result<EventEmitter, io::Error> {
        Ok(EventEmitter {
            writer: self.writer.try_clone()?,
            sender: self.sender.clone(),
        })
    }

    pub fn pipe_fd(&self) -> BorrowedFd<'_> {
        self.reader.as_fd()
    }

    pub fn recv(&mut self) -> Result<Event, AbsorbError> {
        self.reader.read_exact(&mut [0_u8])?;
        Ok(self.receiver.recv()?)
    }

    pub fn try_recv(&mut self) -> Result<Event, AbsorbError> {
        match self.reader.read(&mut [0_u8]) {
            Ok(1) => {}
            Ok(..) => return Err(AbsorbError::WouldBlock),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                return Err(AbsorbError::WouldBlock);
            }
            Err(error) => return Err(error.into()),
        }

        Ok(self.receiver.try_recv()?)
    }
}

pub struct EventEmitter {
    writer: PipeWriter,
    sender: Sender<Event>,
}

impl EventEmitter {
    pub fn emit(&mut self, event: impl IntoEvent) -> Result<(), EmitError> {
        self.writer.write_all(bytemuck::bytes_of(&EventType::Any))?;
        self.sender
            .send(event.into_event())
            .map_err(|_| EmitError::Disconnected)?;
        Ok(())
    }
}

impl Clone for EventEmitter {
    fn clone(&self) -> Self {
        Self {
            writer: self.writer.try_clone().unwrap(),
            sender: self.sender.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum EmitError {
    #[error(transparent)]
    Pipe(#[from] io::Error),
    #[error("receiver is disconnected")]
    Disconnected,
}

#[derive(Debug, Error)]
pub enum AbsorbError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    TryRecv(#[from] TryRecvError),
    #[error(transparent)]
    Recv(#[from] RecvError),
    #[error("would block")]
    WouldBlock,
}
