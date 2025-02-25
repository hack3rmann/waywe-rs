pub mod callback;
pub mod compositor;
pub mod display;
pub mod registry;
pub mod shm;
pub mod shm_pool;
pub mod surface;
pub mod zwlr_layer_shell_v1;
pub mod zwlr_layer_surface_v1;

use crate::{
    object::{ObjectIdMap, ObjectIdProvider},
    wire::MessageHeader,
};

use super::{
    object::ObjectId,
    wire::{self, Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
};
use std::{
    io::{self, Read},
    os::{fd::AsFd, unix::net::UnixStream},
};
use thiserror::Error;

pub use {
    callback::event::Done as WlCallbackDoneEvent,
    compositor::request::{
        CreateRegion as WlCompositorCreateRegion, CreateSurface as WlCompositorCreateSurface,
    },
    display::{
        event::{DeleteId as WlDisplayDeleteIdEvent, Error as WlDisplayErrorEvent},
        request::{GetRegistry as WlDisplayGetRegistryRequest, Sync as WlDisplaySyncRequest},
        wl_enum::Error as WlDisplayErrorEnum,
    },
    registry::{
        event::{Global as WlRegistryGlobalEvent, GlobalRemove as WlRegistryGlobalRemoveEvent},
        request::Bind as WlRegistryBindRequest,
    },
    shm::{request::CreatePool as WlShmCreatePoolRequest, wl_enum::Format as WlShmFormat},
    shm_pool::request::CreateBuffer as WlShmPoolCreateBufferRequest,
    surface::{
        event::{Enter as WlSurfaceEnterEvent, Leave as WlSurfaceLeaveEvent},
        request::{
            Attach as WlSurfaceAttachRequest, Commit as WlSurfaceCommitRequest,
            Damage as WlSurfaceDamageRequest, Destroy as WlSurfaceDestroyRequest,
            Frame as WlSurfaceFrameRequest, SetInputRegion as WlSurfaceSetInputRegionRequest,
            SetOpaqueRegion as SetOpaqueRegionRequest,
        },
        wl_enum::Error as WlSurfaceError,
    },
};

/// An [`ObjectId`] bundled with an interface name and a version
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct NewId<'s> {
    pub id: ObjectId,
    pub interface: &'s str,
    pub version: u32,
}

/// Represents requests on Wayland's interfaces
pub trait Request: Copy {
    /// The object id and the opcode for this request
    fn header_desc(self) -> MessageHeaderDesc;

    /// Builds the message on the top of given message buffer
    fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError>;

    /// Sends built message to the stream
    fn send(self, stream: impl AsFd, buf: &mut MessageBuffer) -> Result<(), MessageBuildError> {
        self.build_message(buf)?.send(stream)?;
        Ok(())
    }
}

/// Represents events on Wayland's interfaces
pub trait Event<'s>: Copy {
    /// The object id and the opcode for this event
    fn header_desc(self) -> MessageHeaderDesc;

    /// Tries to read the given message as an event of implementor type
    fn from_message(message: Message<'s>) -> Option<Self>;

    /// Receives read message from the stream
    fn recv(stream: &mut dyn Read, buf: &'s mut MessageBuffer) -> Result<Self, RecvEventError> {
        wire::read_message_into(stream, buf)?;
        let message = buf.get_message();

        Self::from_message(message).ok_or_else(|| RecvEventError::Parse(message.header()))
    }
}

#[derive(Debug, Error)]
pub enum RecvEventError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("failed to parse message with header {0:?}")]
    Parse(MessageHeader),
}

pub fn send_request(
    request: impl Request,
    stream: impl AsFd,
    buf: &mut MessageBuffer,
) -> Result<(), MessageBuildError> {
    request.send(stream, buf)
}

pub fn recv_event<'b>(
    id_map: &ObjectIdMap,
    stream: &mut dyn Read,
    buf: &'b mut MessageBuffer,
) -> Result<AnyEvent<'b>, RecvAnyEventError> {
    wire::read_message_into(stream, buf)?;
    AnyEvent::new_global(id_map, buf.get_message()).ok_or(RecvAnyEventError::Parse)
}

#[derive(Debug, Error)]
pub enum RecvAnyEventError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("unknown event")]
    Parse,
}

/// Bundles all implemented events together
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub enum AnyEvent<'s> {
    WlDisplayDeleteId(WlDisplayDeleteIdEvent),
    WlDisplayError(WlDisplayErrorEvent<'s>),
    WlRegistryGlobal(WlRegistryGlobalEvent<'s>),
    WlRegistryGlobalRemove(WlRegistryGlobalRemoveEvent),
    WlCallbackDone(WlCallbackDoneEvent),
}

impl<'s> AnyEvent<'s> {
    pub fn new_global(id_map: &ObjectIdMap, message: Message<'s>) -> Option<Self> {
        let header = message.header();
        let object_id = ObjectId::try_from(header.object_id).ok()?;
        let object_name = id_map.get_name(object_id)?;

        Some(match (object_name, header.opcode) {
            (ObjectId::WL_REGISTRY, 0) => {
                Self::WlRegistryGlobal(WlRegistryGlobalEvent::from_message(message)?)
            }
            (ObjectId::WL_REGISTRY, 1) => {
                Self::WlRegistryGlobalRemove(WlRegistryGlobalRemoveEvent::from_message(message)?)
            }
            (ObjectId::WL_DISPLAY, 0) => {
                Self::WlDisplayError(WlDisplayErrorEvent::from_message(message)?)
            }
            (ObjectId::WL_DISPLAY, 1) => {
                Self::WlDisplayDeleteId(WlDisplayDeleteIdEvent::from_message(message)?)
            }
            _ => return None,
        })
    }
}

pub fn wayland_sync_with(
    stream: &mut UnixStream,
    buf: &mut MessageBuffer,
    id_map: &ObjectIdMap,
    id_provider: &mut ObjectIdProvider,
    wait: impl FnOnce(&mut UnixStream, &mut MessageBuffer, ObjectId) -> Result<(), io::Error>,
) -> Result<(), WaylandSyncNowError> {
    let callback_id = id_provider.next_id();

    send_request(
        WlDisplaySyncRequest {
            object_id: id_map.get_id(ObjectId::WL_DISPLAY).unwrap(),
            callback: callback_id,
        },
        &mut *stream,
        buf,
    )?;

    wait(stream, buf, callback_id)?;

    assert_eq!(
        WlCallbackDoneEvent::recv(stream, buf)?.object_id,
        callback_id,
    );

    Ok(())
}

pub fn wayland_sync_now(
    stream: &mut UnixStream,
    buf: &mut MessageBuffer,
    id_map: &ObjectIdMap,
    id_provider: &mut ObjectIdProvider,
) -> Result<(), WaylandSyncNowError> {
    wayland_sync_with(stream, buf, id_map, id_provider, |_, _, _| Ok(()))
}

#[derive(Debug, Error)]
pub enum WaylandSyncNowError {
    #[error(transparent)]
    SendFailed(#[from] MessageBuildError),
    #[error(transparent)]
    RecvFailed(#[from] RecvEventError),
    #[error(transparent)]
    Io(#[from] io::Error),
}
