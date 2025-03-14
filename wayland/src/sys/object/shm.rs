use super::{Dispatch, WlObject, WlObjectHandle, shm_pool::WlShmPool};
use crate::{
    interface::{Request, WlShmCreatePoolRequest},
    sys::{HasObjectType, ObjectType, object_storage::WlObjectStorage, wire::MessageBuffer},
};
use std::os::fd::AsFd;

#[derive(Debug, Default)]
pub struct WlShm;

impl WlShm {
    pub fn create_pool(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
        shm: WlObjectHandle<Self>,
        fd: impl AsFd,
        size: usize,
    ) -> WlObjectHandle<WlShmPool> {
        let proxy = unsafe {
            WlShmCreatePoolRequest {
                fd: fd.as_fd(),
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
