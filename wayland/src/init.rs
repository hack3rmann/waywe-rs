use crate::{
    interface::{
        self, Event as _, NewId, RecvAnyEventError, WlCallbackDoneEvent,
        WlDisplayDeleteIdEvent, WlDisplaySyncRequest, WlRegistryBindRequest,
    },
    object::{ObjectId, ObjectIdProvider},
    wire::{MessageBuffer, MessageBuildError},
};

use super::c_api::{ExternalWaylandContext, ExternalWaylandError, initialize_wayland};
use rustix::net::SocketAddrAny;
use std::{
    env,
    ffi::OsString,
    io,
    os::{
        fd::{AsRawFd, BorrowedFd, FromRawFd, IntoRawFd as _, OwnedFd, RawFd},
        unix::net::UnixStream,
    },
    path::PathBuf,
};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct WaylandContext {
    pub(crate) sock: RawFd,
    pub(crate) external_context: ExternalWaylandContext,
}

impl WaylandContext {
    /// # Safety
    ///
    /// Wayland socket's file desc should not be owned anywhere else in this program.
    pub unsafe fn new() -> Result<Self, WaylandInitError> {
        let sock = unsafe { connect_wayland_socket()?.into_raw_fd() };
        let (external_context, object_info) = unsafe { initialize_wayland(sock)? };

        // Safety: external wayland-client impl
        // uses this sock no more therefore we can own it
        let mut sock = unsafe { UnixStream::from_raw_fd(sock) };
        let mut buf = MessageBuffer::new();

        let mut id_map = object_info.mapped_names;

        let max_id = ObjectId::new(id_map.list.into_iter().max().unwrap());
        let mut id_provider = ObjectIdProvider::new(max_id);

        let wl_shm_id = id_provider.next_id();
        let wl_shm_interface = "wl_shm";
        let wl_shm = object_info.globals[wl_shm_interface];
        id_map.set_id(ObjectId::WL_SHM, wl_shm_id);

        interface::send_request(
            WlRegistryBindRequest {
                object_id: id_map.get_id(ObjectId::WL_REGISTRY).unwrap(),
                name: wl_shm.name,
                new_id: NewId {
                    id: wl_shm_id,
                    interface: wl_shm_interface,
                    version: wl_shm.version,
                },
            },
            &mut sock,
            &mut buf,
        )?;

        let sync_object_id = id_provider.next_id();

        interface::send_request(
            WlDisplaySyncRequest {
                object_id: id_map.get_id(ObjectId::WL_DISPLAY).unwrap(),
                callback: sync_object_id,
            },
            &mut sock,
            &mut buf,
        )?;

        while WlCallbackDoneEvent::recv(&mut sock, &mut buf)?.object_id != sync_object_id {}

        assert_eq!(
            WlDisplayDeleteIdEvent::recv(&mut sock, &mut buf)?.removed_id,
            sync_object_id
        );

        Ok(Self {
            sock: sock.as_raw_fd(),
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
    #[error(transparent)]
    MessageBuildError(#[from] MessageBuildError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    MessageRecv(#[from] RecvAnyEventError),
}

/// # Safety
///
/// Wayland socket's file desc should not be owned anywhere else in this program.
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
