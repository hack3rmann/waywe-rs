use super::{
    object::ObjectId,
    wire::{self, Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
};
use std::io::{self, Read, Write};

pub use {
    display::{
        event::{DeleteId as WlDisplayDeleteIdEvent, Error as WlDisplayErrorEvent},
        request::{GetRegistry as WlDisplayGetRegistryRequest, Sync as WlDisplaySyncRequest},
        wl_enum::Error as WlDisplayErrorEnum,
    },
    registry::{
        event::{Global as WlRegistryGlobalEvent, GlobalRemove as WlRegistryGlobalRemoveEvent},
        request::Bind as WlRegistryBindRequest,
    },
    callback::event::Done as WlCallbackDoneEvent,
};

pub mod callback;
pub mod display;
pub mod registry;

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct NewId<'s> {
    pub id: ObjectId,
    pub interface: &'s str,
    pub version: u32,
}

pub trait Request: Copy {
    fn header_desc() -> MessageHeaderDesc;
    fn build_message(self, buf: &mut MessageBuffer) -> Result<&Message, MessageBuildError>;

    fn send(
        self,
        stream: &mut dyn Write,
        buf: &mut MessageBuffer,
    ) -> Result<(), MessageBuildError> {
        self.build_message(buf)?.send(stream)?;
        Ok(())
    }
}

pub trait Event<'s>: Copy {
    fn header_desc() -> Option<MessageHeaderDesc>;
    fn from_message(message: &'s Message) -> Option<Self>;

    fn recv(stream: &mut dyn Read, buf: &'s mut MessageBuffer) -> Result<Self, io::Error> {
        wire::read_message_into(stream, buf)?;
        // TODO: handle error
        Ok(Self::from_message(buf.get_message()).unwrap())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnyEvent<'s> {
    WlDisplayDeleteId(WlDisplayDeleteIdEvent),
    WlDisplayError(WlDisplayErrorEvent<'s>),
    WlRegistryGlobal(WlRegistryGlobalEvent<'s>),
    WlRegistryGlobalRemove(WlRegistryGlobalRemoveEvent),
    WlCallbackDone(WlCallbackDoneEvent),
    Other(&'s Message),
}

impl<'s> From<&'s Message> for AnyEvent<'s> {
    fn from(message: &'s Message) -> Self {
        let header = message.header();

        match (ObjectId::new(header.object_id), header.opcode) {
            (ObjectId::WL_REGISTRY, 0) => {
                Self::WlRegistryGlobal(WlRegistryGlobalEvent::from_message(message).unwrap())
            }
            (ObjectId::WL_REGISTRY, 1) => {
                Self::WlRegistryGlobalRemove(WlRegistryGlobalRemoveEvent::from_message(message).unwrap())
            }
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
