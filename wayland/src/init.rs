use super::GetSocketPathError;
use super::c_api::ExternalWaylandContext;
use super::connect_wayland_socket;
use crate::c_api::ExternalWaylandError;
use crate::c_api::initialize_wayland;
use std::os::fd::BorrowedFd;
use std::os::fd::IntoRawFd as _;
use std::os::fd::RawFd;
use thiserror::Error;

pub struct WaylandContext {
    pub(crate) sock: RawFd,
    pub(crate) extern_context: ExternalWaylandContext,
}

impl WaylandContext {
    pub fn new() -> Result<Self, WaylandInitError> {
        let sock = unsafe { connect_wayland_socket()?.into_raw_fd() };
        let (extern_context, _object_info) = unsafe { initialize_wayland(sock)? };

        Ok(Self {
            sock,
            extern_context,
        })
    }

    pub fn wayland_sock(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.sock) }
    }

    pub fn extern_context(&self) -> &ExternalWaylandContext {
        &self.extern_context
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
    #[error(transparent)]
    ExternalError(#[from] ExternalWaylandError),
}
