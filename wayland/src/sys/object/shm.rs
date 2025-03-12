use super::{Dispatch, WlObject, WlObjectHandle, shm_pool::WlShmPool};
use crate::{
    interface::{Request, WlShmCreatePoolRequest},
    sys::{HasObjectType, ObjectType, object_storage::WlObjectStorage, wire::MessageBuffer},
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

impl HasObjectType for WlShm {
    const OBJECT_TYPE: ObjectType = ObjectType::Shm;
}

impl Dispatch for WlShm {}
