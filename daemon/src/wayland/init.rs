use super::c_api::ExternWaylandContext;
use super::connect_wayland_socket;
use super::GetSocketPathError;
use std::os::fd::BorrowedFd;
use std::os::fd::IntoRawFd as _;
use std::os::fd::RawFd;
use thiserror::Error;

pub struct WaylandContext {
    pub(crate) sock: RawFd,
    pub(crate) extern_context: ExternWaylandContext,
}

impl WaylandContext {
    pub fn new() -> Result<Self, WaylandInitError> {
        let sock = unsafe { connect_wayland_socket()?.into_raw_fd() };
        let extern_context = unsafe { ExternWaylandContext::from_raw_fd(sock) };

        Ok(Self {
            sock,
            extern_context,
        })
    }

    pub fn wayland_sock(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.sock) }
    }
}

impl Drop for WaylandContext {
    fn drop(&mut self) {
        unsafe { self.extern_context.close_connection() }.unwrap()
    }
}

#[derive(Debug, Error)]
pub enum WaylandInitError {
    #[error(transparent)]
    GetSocketPath(#[from] GetSocketPathError),
}
