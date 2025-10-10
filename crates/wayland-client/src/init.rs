//! Connecting to wayland on-init

use rustix::net::SocketAddrAny;
use std::{
    env,
    ffi::OsString,
    io,
    os::{
        fd::{FromRawFd, OwnedFd},
        unix::net::UnixStream,
    },
    path::PathBuf,
};
use thiserror::Error;

/// # Safety
///
/// Wayland socket's file desc should not be owned anywhere else in this program.
pub unsafe fn connect_wayland_socket() -> Result<OwnedFd, ConnectWaylandSocketError> {
    if let Ok(sock) = env::var("WAYLAND_SOCKET") {
        let file_desc_number = sock
            .parse::<i32>()
            .map_err(|_| ConnectWaylandSocketError::InvallidWaylandSocketEnvVar(sock))?;

        // Safety: see safety invariant above
        let file_desc = unsafe { OwnedFd::from_raw_fd(file_desc_number) };

        let socket_address = rustix::net::getsockname(&file_desc)
            .map_err(ConnectWaylandSocketError::GetSockNameFailed)?;

        if !matches!(socket_address, SocketAddrAny::Unix(..)) {
            return Err(ConnectWaylandSocketError::SocketAddrIsNotUnix(
                socket_address,
            ));
        }

        return Ok(file_desc);
    }

    let xdg_runtime_dir: PathBuf = env::var_os("XDG_RUNTIME_DIR")
        .unwrap_or_else(|| {
            tracing::warn!("XDG_RUNTIME_DIR env variable not set");

            let real_user_id = rustix::process::getuid();
            OsString::from(format!("/run/user/{}", real_user_id.as_raw()))
        })
        .into();

    let display_name =
        env::var_os("WAYLAND_DISPLAY").unwrap_or_else(|| OsString::from("wayland-0"));

    let mut socket_path = xdg_runtime_dir;
    socket_path.push(&display_name);

    UnixStream::connect(&socket_path)
        .map(OwnedFd::from)
        .map_err(|error| ConnectWaylandSocketError::FailedToConnectToPath {
            error,
            path: socket_path,
        })
}

/// Failed to connect to wayland sock
#[derive(Debug, Error)]
pub enum ConnectWaylandSocketError {
    /// `$WAYLAND_SOCKET` env variable is not `i32` integer
    #[error("invalid $WAYLAND_SOCKET env variable '{0}'")]
    InvallidWaylandSocketEnvVar(String),

    /// Invalid file desc in `$WAYLAND_SOCKET`
    #[error(transparent)]
    GetSockNameFailed(#[from] rustix::io::Errno),

    /// Socket address passed into `$WAYLAND_SOCKET` var is not UNIX
    #[error("socket address '{0:?}' is not unix")]
    SocketAddrIsNotUnix(SocketAddrAny),

    /// Connect failed
    #[error("failed to connect to wayland socket from '{path}': {error}")]
    FailedToConnectToPath {
        /// OS error
        error: io::Error,
        /// Path tried to connect to
        path: PathBuf,
    },
}
