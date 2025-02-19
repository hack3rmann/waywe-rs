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
};

pub mod display;
pub mod registry;

pub type NewId = ObjectId;

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
    fn header_desc() -> MessageHeaderDesc;
    fn from_message(message: &'s Message) -> Self;

    fn recv(stream: &mut dyn Read, buf: &'s mut MessageBuffer) -> Result<Self, io::Error> {
        wire::read_message_into(stream, buf)?;
        Ok(Self::from_message(buf.get_message()))
    }
}
