use super::{Dispatch, WlObject, WlObjectHandle, shm_pool::WlShmPool};
use crate::{
    interface::{Request, WlShmCreatePoolRequest, registry::request::HasInterface},
    sys::{
        Interface, ObjectType,
        object_storage::WlObjectStorage,
        wire::{Message, MessageBuffer},
    },
};
use std::os::fd::BorrowedFd;

#[derive(Debug)]
pub struct WlShm;

impl WlShm {
    pub fn create_pool(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
        shm: WlObjectHandle<Self>,
        fd: BorrowedFd<'_>,
        size: usize,
    ) -> WlObjectHandle<WlShmPool> {
        let proxy = unsafe {
            WlShmCreatePoolRequest {
                fd,
                size: size as i32,
            }
            .send(storage.object(shm).proxy(), buf)
            .unwrap()
        };

        storage.insert(WlObject::new(proxy, WlShmPool))
    }
}

impl HasInterface for WlShm {
    const INTERFACE: Interface = Interface {
        object_type: ObjectType::Shm,
        version: 2,
    };
}

impl Dispatch for WlShm {
    fn dispatch(&mut self, _: Message<'_>) {}
}
