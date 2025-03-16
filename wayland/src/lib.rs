pub mod init;
pub mod interface;
pub mod object;
pub mod sys;

pub use {
    object::{HasObjectType, WlObjectId, WlObjectType},
    sys::{
        object::{WlObject, WlObjectHandle},
        object::dispatch::Dispatch,
        object_storage::{NoEntryError, ObjectDataAcquireError, WlObjectStorage},
        wire::{
            MessageBuffer, OpCode, SmallVecMessageBuffer, StackMessageBuffer, VecMessageBuffer,
        },
        proxy::WlProxy,
        protocol,
    },
};
