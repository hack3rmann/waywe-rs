use runtime::{DaemonCommand, IpcSocket, ipc::Server};
use rustix::io::Errno;

pub struct Ipc {
    pub socket: IpcSocket<Server, DaemonCommand>,
}

impl Ipc {
    pub fn new() -> Result<Self, Errno> {
        Ok(Self {
            socket: IpcSocket::server()?,
        })
    }
}
