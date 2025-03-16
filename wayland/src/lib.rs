pub mod init;
pub mod interface;
pub mod object;
pub mod sys;

pub use {
    object::{HasObjectType, WlObjectId, WlObjectType},
    sys::{
        object::{WlObject, WlObjectHandle},
        wire::{
            MessageBuffer, OpCode, SmallVecMessageBuffer, StackMessageBuffer, VecMessageBuffer,
        },
    },
};
