use super::c_api::{ExternalWaylandContext, ExternalWaylandError, initialize_wayland};
use crate::{
    interface::{
        self, Event as _, NewId, RecvAnyEventError, RecvEventError, WlCallbackDoneEvent,
        WlDisplayDeleteIdEvent, WlDisplaySyncRequest, WlRegistryBindRequest,
        WlShmCreatePoolRequest, WlShmFormat, WlShmPoolCreateBufferRequest, WlSurfaceAttachRequest,
        WlSurfaceCommitRequest, WlSurfaceDamageRequest,
    },
    object::{ObjectId, ObjectIdProvider},
    wire::{MessageBuffer, MessageBuildError},
};
use core::slice;
use rustix::{
    fs::Mode,
    mm::{MapFlags, ProtFlags},
    net::SocketAddrAny,
    shm::OFlags,
};
use std::{
    env,
    ffi::OsString,
    io, mem,
    os::{
        fd::{AsRawFd, BorrowedFd, FromRawFd, IntoRawFd as _, OwnedFd, RawFd},
        unix::net::UnixStream,
    },
    path::PathBuf,
    ptr,
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

        let max_id = id_map.iter().map(|(&_name, &id)| id).max().unwrap();
        let mut id_provider = ObjectIdProvider::new(max_id);

        let wl_shm_id = id_provider.next_id();
        let wl_shm_interface = "wl_shm";
        let wl_shm = object_info.globals[wl_shm_interface];
        id_map.map(ObjectId::WL_SHM, wl_shm_id);

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

        fn open_shm() -> Result<(OwnedFd, String), rustix::io::Errno> {
            for i in 0.. {
                let wl_shm_path = format!("/wl_shm#{i}");

                match rustix::shm::open(
                    &wl_shm_path,
                    OFlags::EXCL | OFlags::RDWR | OFlags::CREATE | OFlags::TRUNC,
                    Mode::RUSR | Mode::WUSR,
                ) {
                    Ok(fd) => return Ok((fd, wl_shm_path)),
                    Err(rustix::io::Errno::EXIST) => continue,
                    Err(error) => return Err(error),
                };
            }

            unreachable!();
        }

        let (shm_file_desc, wl_shm_path) = open_shm()?;

        const BUFFER_WIDTH: usize = 2520;
        const BUFFER_HEIGHT: usize = 1680;
        const COLOR_SIZE: usize = mem::size_of::<u32>();
        const BUFFER_SIZE: usize = BUFFER_WIDTH * BUFFER_HEIGHT * COLOR_SIZE;

        rustix::fs::ftruncate(&shm_file_desc, BUFFER_SIZE as u64)?;

        let shm_ptr = unsafe {
            rustix::mm::mmap(
                ptr::null_mut(),
                BUFFER_SIZE,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::SHARED,
                &shm_file_desc,
                0,
            )?
        };

        assert!(!shm_ptr.is_null());

        unsafe { shm_ptr.write_bytes(0, BUFFER_SIZE) };

        let _shm =
            unsafe { slice::from_raw_parts_mut(shm_ptr.cast::<u32>(), BUFFER_SIZE / COLOR_SIZE) };

        let wl_shm_pool_id = id_provider.next_id();
        id_map.map(ObjectId::WL_SHM_POOL, wl_shm_pool_id);

        interface::send_request(
            WlShmCreatePoolRequest {
                object_id: id_map.get_id(ObjectId::WL_SHM).unwrap(),
                id: wl_shm_pool_id,
                fd: shm_file_desc.as_raw_fd(),
                size: BUFFER_SIZE as i32,
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

        assert_eq!(
            WlCallbackDoneEvent::recv(&mut sock, &mut buf)?.object_id,
            sync_object_id
        );

        assert_eq!(
            WlDisplayDeleteIdEvent::recv(&mut sock, &mut buf)?.removed_id,
            sync_object_id
        );

        rustix::shm::unlink(wl_shm_path)?;

        let wl_buffer_id = id_provider.next_id();

        interface::send_request(
            WlShmPoolCreateBufferRequest {
                object_id: wl_shm_pool_id,
                id: wl_buffer_id,
                offset: 0,
                width: BUFFER_WIDTH as i32,
                height: BUFFER_HEIGHT as i32,
                stride: (BUFFER_WIDTH * COLOR_SIZE) as i32,
                format: WlShmFormat::Xrgb8888,
            },
            &sock,
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

        assert_eq!(
            WlCallbackDoneEvent::recv(&mut sock, &mut buf)?.object_id,
            sync_object_id
        );

        assert_eq!(
            WlDisplayDeleteIdEvent::recv(&mut sock, &mut buf)?.removed_id,
            sync_object_id
        );

        interface::send_request(
            WlSurfaceAttachRequest {
                object_id: id_map.get_id(ObjectId::WL_SURFACE).unwrap(),
                buffer: wl_buffer_id,
                x: 0,
                y: 0,
            },
            &sock,
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

        assert_eq!(
            WlCallbackDoneEvent::recv(&mut sock, &mut buf)?.object_id,
            sync_object_id
        );

        assert_eq!(
            WlDisplayDeleteIdEvent::recv(&mut sock, &mut buf)?.removed_id,
            sync_object_id
        );

        interface::send_request(
            WlSurfaceDamageRequest {
                object_id: id_map.get_id(ObjectId::WL_SURFACE).unwrap(),
                x: 0,
                y: 0,
                width: BUFFER_WIDTH as i32,
                height: BUFFER_HEIGHT as i32,
            },
            &sock,
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

        assert_eq!(
            WlCallbackDoneEvent::recv(&mut sock, &mut buf)?.object_id,
            sync_object_id
        );

        assert_eq!(
            WlDisplayDeleteIdEvent::recv(&mut sock, &mut buf)?.removed_id,
            sync_object_id
        );

        interface::send_request(
            WlSurfaceCommitRequest {
                object_id: id_map.get_id(ObjectId::WL_SURFACE).unwrap(),
            },
            &mut sock,
            &mut buf,
        )?;

        Ok(Self {
            sock: sock.into_raw_fd(),
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
    RustixIo(#[from] rustix::io::Errno),
    #[error(transparent)]
    MessageRecvAny(#[from] RecvAnyEventError),
    #[error(transparent)]
    MessageRecv(#[from] RecvEventError),
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
