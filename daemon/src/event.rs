use bytemuck::{Contiguous, NoUninit};
use rustix::fs::OFlags;
use std::{
    io::{self, ErrorKind, PipeReader, PipeWriter, Read as _, Write as _},
    os::fd::{AsFd, BorrowedFd},
    sync::mpsc::{self, Receiver, RecvError, Sender, TryRecvError},
};
use thiserror::Error;

#[repr(u8)]
#[derive(
    Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, NoUninit, Contiguous,
)]
pub enum EventType {
    #[default]
    Custom = 0,
}

pub trait CustomEvent: Send + 'static {}
impl<T: Send + 'static> CustomEvent for T {}

#[derive(Debug)]
pub struct EventReceiver<T> {
    reader: PipeReader,
    writer: PipeWriter,
    receiver: Receiver<T>,
    sender: Sender<T>,
}

impl<T: CustomEvent> EventReceiver<T> {
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

    pub fn make_emitter(&self) -> Result<EventEmitter<T>, io::Error> {
        Ok(EventEmitter {
            writer: self.writer.try_clone()?,
            sender: self.sender.clone(),
        })
    }

    pub fn pipe_fd(&self) -> BorrowedFd<'_> {
        self.reader.as_fd()
    }

    pub fn recv(&mut self) -> Result<T, AbsorbError> {
        self.reader.read_exact(&mut [0_u8])?;
        Ok(self.receiver.recv()?)
    }

    pub fn try_recv(&mut self) -> Result<T, AbsorbError> {
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

pub struct EventEmitter<T> {
    writer: PipeWriter,
    sender: Sender<T>,
}

impl<T> EventEmitter<T> {
    pub fn emit(&mut self, event: T) -> Result<(), EmitError> {
        self.writer
            .write_all(bytemuck::bytes_of(&EventType::Custom))?;
        self.sender
            .send(event)
            .map_err(|_| EmitError::Disconnected)?;
        Ok(())
    }
}

impl<T> Clone for EventEmitter<T> {
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
