use crate::init::{WaylandSocketCreateInfo, create_wayland_socket};
use rustix::{fs, io::Errno};
use std::{
    ffi::{OsStr, OsString},
    os::fd::{FromRawFd, IntoRawFd as _, OwnedFd},
    path::PathBuf,
    ptr::NonNull,
};
use thiserror::Error;
use tracing::error;
use wayland_sys::{wl_display, wl_display_add_socket_fd, wl_display_create, wl_display_destroy};

pub struct WlDisplay {
    socket_path: PathBuf,
    name: OsString,
    raw: NonNull<wl_display>,
}

impl WlDisplay {
    pub fn create() -> Result<Self, WlDisplayCreateError> {
        let Some(raw) = NonNull::new(wl_display_create()) else {
            panic!("failed to create wayland display");
        };

        let WaylandSocketCreateInfo {
            socket,
            display_name: name,
            socket_path,
        } = create_wayland_socket()?;

        let socket = socket.into_raw_fd();

        if 0 != unsafe { wl_display_add_socket_fd(raw.as_ptr(), socket) } {
            // NOTE(hack3rmann): libwayland does not close the socket on error
            _ = unsafe { OwnedFd::from_raw_fd(socket) };
            return Err(WlDisplayCreateError::AddSocketFd);
        }

        Ok(Self {
            raw,
            socket_path,
            name,
        })
    }

    pub fn as_raw(&self) -> NonNull<wl_display> {
        self.raw
    }

    pub fn name(&self) -> &OsStr {
        &self.name
    }
}

impl Drop for WlDisplay {
    fn drop(&mut self) {
        unsafe { wl_display_destroy(self.raw.as_ptr()) };

        // NOTE(hack3rmann): no need to cause panic in destructor
        if let Err(error) = fs::unlink(&self.socket_path) {
            error!(?error, "failed to unlink wayland socket file");
        }
    }
}

#[derive(Debug, Error)]
pub enum WlDisplayCreateError {
    #[error(transparent)]
    Os(#[from] Errno),
    #[error("failed to add socket fd to display")]
    AddSocketFd,
}
