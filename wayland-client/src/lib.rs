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
            FromProxy,
            dispatch::{Dispatch, NoState, State},
            event_queue::{CreateQueueError, WlEventQueue},
            registry::WlRegistry,
            {NonZstError, WlObject, WlObjectHandle},
        },
        object_storage::{NoEntryError, ObjectDataAcquireError, WlObjectStorage},
        protocol,
        proxy::WlProxy,
        wire::{
            MessageBuffer, OpCode, SmallVecMessageBuffer, StackMessageBuffer, VecMessageBuffer,
            WlMessage,
        },
    },
    wayland_sys as ffi,
};
