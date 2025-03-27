// FIXME(hack3rmann): warn missing safety
#![allow(clippy::missing_safety_doc)]

pub mod init;
pub mod interface;
pub mod object;
pub mod sys;

pub use {
    interface::WlObjectType,
    object::{HasObjectType, WlObjectId},
    sys::{
        display::{DisplayConnectError, DisplayConnectToFdError, WlDisplay},
        object::{
            dispatch::Dispatch,
            registry::WlRegistry,
            {WlObject, WlObjectHandle},
        },
        object_storage::{NoEntryError, ObjectDataAcquireError, WlObjectStorage},
        protocol,
        proxy::WlProxy,
        wire::{
            MessageBuffer, OpCode, SmallVecMessageBuffer, StackMessageBuffer, VecMessageBuffer,
        },
    },
};
