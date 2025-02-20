use rustix::net::SocketAddrAny;
use std::{
    env,
    ffi::OsString,
    io,
    os::{
        fd::{FromRawFd as _, OwnedFd},
        unix::net::UnixStream,
    },
    path::PathBuf,
};
use thiserror::Error;

pub mod init;
pub mod interface;
pub mod object;
pub mod wire;

/// # Safety
///
/// Wayland socket's `OwnedFd` should not be owned anywhere else in this program.
pub unsafe fn connect_wayland_socket() -> Result<OwnedFd, GetSocketPathError> {
    if let Ok(sock) = env::var("WAYLAND_SOCKET") {
        let file_desc_number = sock
            .parse::<i32>()
            .map_err(|_| GetSocketPathError::InvallidWaylandSocketEnvVar(sock))?;

        // Safety: see safety invariant above
        let file_desc = unsafe { OwnedFd::from_raw_fd(file_desc_number) };

        let socket_address =
            rustix::net::getsockname(&file_desc).map_err(GetSocketPathError::GetSockNameFailed)?;

        if !matches!(socket_address, SocketAddrAny::Unix(..)) {
            return Err(GetSocketPathError::SocketAddrIsNotUnix(socket_address));
        }

        return Ok(file_desc);
    }

    let xdg_runtime_dir: PathBuf = env::var_os("XDG_RUNTIME_DIR")
        .unwrap_or_else(|| {
            // TODO: print warn user about this
            let real_user_id = rustix::process::getuid();
            OsString::from(format!("/run/user/{}", real_user_id.as_raw()))
        })
        .into();

    let display_name =
        env::var_os("WAYLAND_DISPLAY").unwrap_or_else(|| OsString::from("wayland-0"));

    let mut socket_path = xdg_runtime_dir;
    socket_path.push(&display_name);

    UnixStream::connect(&socket_path)
        .map(Into::<OwnedFd>::into)
        .map_err(|error| GetSocketPathError::FailedToConnectToPath {
            error,
            path: socket_path,
        })
}

#[derive(Debug, Error)]
pub enum GetSocketPathError {
    #[error("invalid $WAYLAND_SOCKET env variable '{0}'")]
    InvallidWaylandSocketEnvVar(String),

    #[error(transparent)]
    GetSockNameFailed(#[from] rustix::io::Errno),

    #[error("socket address '{0:?}' is not unix")]
    SocketAddrIsNotUnix(SocketAddrAny),

    #[error("failed to connect to wayland socket from '{path}': {error}")]
    FailedToConnectToPath { error: io::Error, path: PathBuf },
}
