pub mod callback;
pub mod compositor;
pub mod display;
pub mod registry;

use super::{
    object::ObjectId,
    wire::{self, Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
};
use std::io::{self, Read, Write};

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
    fn header_desc(&self) -> MessageHeaderDesc;

    /// Builds the message on the top of given message buffer
    fn build_message(self, buf: &mut MessageBuffer) -> Result<&Message, MessageBuildError>;

    /// Sends built message to the stream
    fn send(
        self,
        stream: &mut dyn Write,
        buf: &mut MessageBuffer,
    ) -> Result<(), MessageBuildError> {
        self.build_message(buf)?.send(stream)?;
        Ok(())
    }
}

/// Represents events on Wayland's interfaces
pub trait Event<'s>: Copy {
    /// The object id and the opcode for this event
    fn header_desc() -> Option<MessageHeaderDesc>;

    /// Tries to read the given message as an event of implementor type
    fn from_message(message: &'s Message) -> Option<Self>;

    /// Receives read message from the stream
    fn recv(stream: &mut dyn Read, buf: &'s mut MessageBuffer) -> Result<Self, io::Error> {
        wire::read_message_into(stream, buf)?;
        // TODO: handle error
        Ok(Self::from_message(buf.get_message()).unwrap())
    }
}

pub fn send_request(
    request: impl Request,
    stream: &mut dyn Write,
    buf: &mut MessageBuffer,
) -> Result<(), MessageBuildError> {
    request.send(stream, buf)
}

pub fn recv_event<'b, E: Event<'b>>(
    stream: &mut dyn Read,
    buf: &'b mut MessageBuffer,
) -> Result<E, io::Error> {
    E::recv(stream, buf)
}

/// Bundles all implemented events together
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub enum AnyEvent<'s> {
    WlDisplayDeleteId(WlDisplayDeleteIdEvent),
    WlDisplayError(WlDisplayErrorEvent<'s>),
    WlRegistryGlobal(WlRegistryGlobalEvent<'s>),
    WlRegistryGlobalRemove(WlRegistryGlobalRemoveEvent),
    WlCallbackDone(WlCallbackDoneEvent),
    Other(&'s Message),
}

impl<'s> Event<'s> for AnyEvent<'s> {
    fn header_desc() -> Option<MessageHeaderDesc> {
        None
    }

    fn from_message(message: &'s Message) -> Option<Self> {
        Some(Self::from(message))
    }
}

impl<'s> From<&'s Message> for AnyEvent<'s> {
    /// Reads a given message into [`AnyEvent`]
    fn from(message: &'s Message) -> Self {
        let header = message.header();

        match (ObjectId::new(header.object_id), header.opcode) {
            (ObjectId::WL_REGISTRY, 0) => {
                Self::WlRegistryGlobal(WlRegistryGlobalEvent::from_message(message).unwrap())
            }
            (ObjectId::WL_REGISTRY, 1) => Self::WlRegistryGlobalRemove(
                WlRegistryGlobalRemoveEvent::from_message(message).unwrap(),
            ),
            (ObjectId::WL_DISPLAY, 0) => {
                Self::WlDisplayError(WlDisplayErrorEvent::from_message(message).unwrap())
            }
            (ObjectId::WL_DISPLAY, 1) => {
                Self::WlDisplayDeleteId(WlDisplayDeleteIdEvent::from_message(message).unwrap())
            }
            (ObjectId::WL_CALLBACK, 0) => {
                Self::WlCallbackDone(WlCallbackDoneEvent::from_message(message).unwrap())
            }
            _ => Self::Other(message),
        }
    }
}
