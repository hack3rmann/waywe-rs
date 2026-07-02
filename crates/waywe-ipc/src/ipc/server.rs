use crate::ipc;
use bincode::{Decode, Encode, error::DecodeError};
use calloop::{
    EventSource, Interest, Mode, Poll, PostAction, Readiness, Token, TokenFactory, channel::Channel,
};
use rustix::{
    io::Errno,
    net::{self, AddressFamily, RecvFlags, SendFlags, SocketAddrUnix, SocketFlags, SocketType},
};
use slab::Slab;
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
    sync::mpsc::TryRecvError,
};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
struct ClientIdGenerator {
    last: u32,
}

impl ClientIdGenerator {
    pub const fn next(&mut self) -> u32 {
        let result = self.last;
        self.last = self.last.wrapping_add(1);
        result
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub struct ClientId {
    id: u32,
    index: u32,
}

struct Client {
    pub fd: OwnedFd,
    pub id: u32,
}

pub struct IpcServer<T, R> {
    clients: Slab<Client>,
    fd: OwnedFd,
    responses: Channel<IpcResponse<R>>,
    response_buf: Vec<u8>,
    new_clients: SmallVec<[usize; 2]>,
    removed_clients: SmallVec<[usize; 2]>,
    id_generator: ClientIdGenerator,
    client_tokens: Vec<(Token, usize)>,
    token: Option<Token>,
    _lock_file: File,
    _p: PhantomData<T>,
}

impl<T, R> AsFd for IpcServer<T, R> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl<T, R> AsRawFd for IpcServer<T, R> {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl<T, R> IpcServer<T, R> {
    pub fn accept_all(&mut self) -> Result<(), RecvError> {
        loop {
            let fd = match net::accept_with(&self.fd, SocketFlags::empty()) {
                Ok(fd) => fd,
                Err(Errno::WOULDBLOCK) => return Err(RecvError::Empty),
                Err(other) => return Err(RecvError::Os(other)),
            };

            let id = self.id_generator.next();
            let index = self.clients.insert(Client { fd, id });

            self.new_clients.push(index);
        }
    }

    pub fn try_recv(&mut self, client_index: usize) -> Result<T, RecvError>
    where
        T: Decode<()>,
    {
        let client = &self.clients[client_index];

        const MAX_LENGTH: u32 = 4096;
        let mut length = 0_u32;

        match net::recv(
            &client.fd,
            bytemuck::bytes_of_mut(&mut length),
            RecvFlags::DONTWAIT,
        ) {
            Ok((_, 0)) => {
                self.removed_clients.push(client_index);
                return Err(RecvError::Disconnected);
            }
            Ok((_, n_bytes)) => assert_eq!(n_bytes, mem::size_of_val(&length)),
            Err(Errno::WOULDBLOCK) => return Err(RecvError::Empty),
            Err(error) => return Err(RecvError::Os(error)),
        }

        assert!(length <= MAX_LENGTH, "too large message, unbeleivable");

        let mut buf: SmallVec<[u8; ipc::BUFFER_SIZE]> = smallvec![0; length as usize];
        net::recv(&client.fd, &mut buf[..], RecvFlags::WAITALL)?;

        let (value, _n_bytes) = bincode::decode_from_slice(&buf, bincode::config::standard())?;

        Ok(value)
    }

    pub fn send(&mut self, response: IpcResponse<R>)
    where
        R: Encode,
    {
        let Some(client) = self.clients.get(response.destination_id.index as usize) else {
            return;
        };
        if client.id != response.destination_id.id {
            return;
        };

        self.response_buf.clear();
        self.response_buf
            .extend_from_slice(bytemuck::bytes_of(&0_u32));

        let n_bytes = bincode::encode_into_std_write(
            response.body,
            &mut self.response_buf,
            bincode::config::standard(),
        )
        .unwrap();

        *bytemuck::from_bytes_mut::<u32>(&mut self.response_buf[..4]) = n_bytes as u32;

        net::send(&client.fd, &self.response_buf, SendFlags::DONTWAIT).unwrap();
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

    pub fn new(responses: Channel<IpcResponse<R>>) -> Result<Self, CreateServerError> {
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
            match net::bind(&socket, &addr) {
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
            responses,
            response_buf: vec![],
            new_clients: SmallVec::new_const(),
            removed_clients: SmallVec::new_const(),
            clients: Slab::new(),
            id_generator: ClientIdGenerator::default(),
            _lock_file: lock_file,
            fd: socket,
            token: None,
            client_tokens: vec![],
            _p: PhantomData,
        })
    }
}

impl<T, R> Drop for IpcServer<T, R> {
    fn drop(&mut self) {
        debug!(path = ipc::socket_file(), "removing daemon socket");
        _ = rustix::fs::unlink(ipc::socket_file());
    }
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error("socket is empty")]
    Empty,
    #[error("client has disconnected")]
    Disconnected,
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

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub struct IpcEvent<T> {
    pub event: T,
    pub sender_id: ClientId,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub struct IpcResponse<T> {
    pub body: T,
    pub destination_id: ClientId,
}

impl<T: Decode<()>, R: Encode> EventSource for IpcServer<T, R> {
    type Event = IpcEvent<T>;
    type Metadata = ();
    type Ret = ();
    type Error = RecvError;

    fn process_events<F>(
        &mut self,
        _: Readiness,
        token: Token,
        mut callback: F,
    ) -> Result<PostAction, Self::Error>
    where
        F: FnMut(Self::Event, &mut Self::Metadata) -> Self::Ret,
    {
        if Some(token) == self.token {
            match self.accept_all() {
                Ok(()) | Err(RecvError::Empty) => {}
                Err(error) => return Err(error),
            }

            return Ok(PostAction::Reregister);
        }

        let Some(client_index) = self
            .client_tokens
            .iter()
            .find_map(|&(client_token, index)| (client_token == token).then_some(index))
        else {
            loop {
                match self.responses.try_recv() {
                    Ok(response) => {
                        self.send(response);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return Ok(PostAction::Reregister),
                }
            }

            return Ok(PostAction::Continue);
        };

        loop {
            match self.try_recv(client_index) {
                Ok(event) => {
                    let sender_id = ClientId {
                        id: self.clients[client_index].id,
                        index: u32::try_from(client_index).unwrap(),
                    };
                    let event = IpcEvent { event, sender_id };

                    callback(event, &mut ());
                }
                Err(RecvError::Empty) => break,
                Err(RecvError::Disconnected) => return Ok(PostAction::Reregister),
                Err(error) => return Err(error),
            }
        }

        Ok(PostAction::Continue)
    }

    fn register(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        let token = token_factory.token();
        self.token = Some(token);

        unsafe { poll.register(&self.fd, Interest::READ, Mode::Level, token)? };

        self.responses.register(poll, token_factory)?;

        self.client_tokens.clear();

        for (id, client) in &self.clients {
            let token = token_factory.token();

            unsafe { poll.register(&client.fd, Interest::READ, Mode::Level, token)? };
            self.client_tokens.push((token, id));
        }

        Ok(())
    }

    fn reregister(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        let token = token_factory.token();
        self.token = Some(token);

        poll.reregister(&self.fd, Interest::READ, Mode::Level, token)?;

        self.responses.reregister(poll, token_factory)?;

        for &id in &self.removed_clients {
            // NOTE(hack3rmann): client must be unregistered before its fd is closed
            poll.unregister(&self.clients[id].fd)?;
        }

        for id in self.removed_clients.drain(..) {
            // Safety: client Fd dies after unregister (see note above)
            self.clients.remove(id);

            let index = self
                .client_tokens
                .iter()
                .position(|(_token, this_id)| *this_id == id)
                .unwrap();
            self.client_tokens.swap_remove(index);
        }

        for (stored_token, id) in &mut self.client_tokens {
            let token = token_factory.token();

            poll.reregister(&self.clients[*id].fd, Interest::READ, Mode::Level, token)?;
            *stored_token = token;
        }

        for id in self.new_clients.drain(..) {
            let token = token_factory.token();

            // Safety: client Fd dies after unregister (see above)
            unsafe { poll.register(&self.clients[id].fd, Interest::READ, Mode::Level, token)? };
            self.client_tokens.push((token, id));
        }

        Ok(())
    }

    fn unregister(&mut self, poll: &mut Poll) -> calloop::Result<()> {
        poll.unregister(&self.fd)?;
        self.token = None;

        self.responses.unregister(poll)?;

        for (_id, client) in &self.clients {
            poll.unregister(&client.fd)?;
        }

        self.clients.clear();
        self.client_tokens.clear();

        Ok(())
    }
}
