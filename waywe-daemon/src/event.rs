use crate::{box_ext::BoxExt, runtime::Runtime};
use bytemuck::{Contiguous, NoUninit};
use fxhash::FxHashMap;
use reusable_box::{ReusableBox, ReusedBoxFuture};
use rustix::fs::OFlags;
use std::{
    any::{Any, TypeId},
    io::{self, ErrorKind, PipeReader, PipeWriter, Read as _, Write as _},
    os::fd::{AsFd, BorrowedFd},
    sync::mpsc::{self, Receiver, RecvError, Sender, TryRecvError},
};
use thiserror::Error;

pub trait Handle<E: IntoEvent> {
    fn handle(&mut self, runtime: &mut Runtime, event: E) -> impl Future<Output = ()> + Send;
}

type Handler<A> = for<'a> unsafe fn(
    &'a mut A,
    &'a mut Runtime,
    Event,
    &'a mut ReusableBox,
) -> ReusedBoxFuture<'a, ()>;

pub struct EventHandler<A> {
    pub handlers: FxHashMap<TypeId, Handler<A>>,
    pub future: ReusableBox,
}

/// # Safety
///
/// - event should contain data
/// - event type should be exactly `E`
unsafe fn handle_event<'a, A, E>(
    app: &'a mut A,
    runtime: &'a mut Runtime,
    event: Event,
    future: &'a mut ReusableBox,
) -> ReusedBoxFuture<'a, ()>
where
    E: IntoEvent,
    A: Handle<E>,
{
    let event = unsafe { event.downcast_unchecked::<E>() };
    future.store_future(<A as Handle<E>>::handle(app, runtime, event))
}

impl<A: 'static> EventHandler<A> {
    pub fn add_event<E>(&mut self) -> &mut Self
    where
        E: IntoEvent,
        A: Handle<E>,
    {
        let id = TypeId::of::<E>();

        self.handlers.insert(id, handle_event::<A, E>);
        self
    }

    pub async fn execute_all(&mut self, app: &mut A, runtime: &mut Runtime, event: Event) {
        let Some(id) = event.underlying_type() else {
            return;
        };

        let Some(handle) = self.handlers.get(&id) else {
            return;
        };

        // Safety:
        // - event contains data
        // - type matches exactly
        unsafe { handle(app, runtime, event, &mut self.future).await };
    }
}

impl<A> Default for EventHandler<A> {
    fn default() -> Self {
        Self {
            handlers: FxHashMap::default(),
            future: ReusableBox::new(),
        }
    }
}

pub struct Event(Option<Box<dyn Any + Send + 'static>>);

impl Event {
    pub fn underlying_type(&self) -> Option<TypeId> {
        self.0
            .as_ref()
            .map(Box::as_ref)
            .map(<dyn Any + Send>::type_id)
    }

    /// # Safety
    ///
    /// - event should contain a value
    /// - underlying type should be exactly `E`
    pub unsafe fn downcast_unchecked<E: IntoEvent>(self) -> E {
        let any = unsafe { self.0.unwrap_unchecked() };
        let boxed = unsafe { any.downcast::<E>().unwrap_unchecked() };
        BoxExt::into_inner(boxed)
    }

    pub async fn handle<T: IntoEvent>(&mut self, f: impl AsyncFnOnce(T)) {
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
