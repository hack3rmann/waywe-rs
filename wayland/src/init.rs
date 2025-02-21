use super::GetSocketPathError;
use super::c_api::ExternalWaylandContext;
use super::c_api::ExternalWaylandError;
use super::c_api::initialize_wayland;
use super::connect_wayland_socket;
use std::os::fd::BorrowedFd;
use std::os::fd::IntoRawFd as _;
use std::os::fd::RawFd;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct WaylandContext {
    pub(crate) sock: RawFd,
    pub(crate) external_context: ExternalWaylandContext,
}

impl WaylandContext {
    pub fn new() -> Result<Self, WaylandInitError> {
        let sock = unsafe { connect_wayland_socket()?.into_raw_fd() };
        let (external_context, _object_info) = unsafe { initialize_wayland(sock)? };

        Ok(Self {
            sock,
            external_context,
        })
    }

    pub fn wayland_sock(&self) -> BorrowedFd<'_> {
        // Safety:
        // - external wayland-client uses socket no more
        // - lifetime of this borrow is attached to the `WaylandContext`
        unsafe { BorrowedFd::borrow_raw(self.sock) }
    }

    pub fn external_context(&self) -> ExternalWaylandContext {
        self.external_context
    }
}

impl Drop for WaylandContext {
    fn drop(&mut self) {
        unsafe { self.external_context.close_connection() };
    }
}

#[derive(Debug, Error)]
pub enum WaylandInitError {
    #[error(transparent)]
    GetSocketPath(#[from] GetSocketPathError),
    #[error(transparent)]
    ExternalError(#[from] ExternalWaylandError),
}
