use super::{Dispatch, WlObject, WlObjectHandle, buffer::WlBuffer};
use crate::{
    interface::{Request, WlShmPoolCreateBufferRequest},
    sys::{HasObjectType, ObjectType, object_storage::WlObjectStorage, wire::MessageBuffer},
};

#[derive(Debug)]
pub struct WlShmPool;

impl WlShmPool {
    pub fn create_buffer(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage<'_>,
        shm_pool: WlObjectHandle<Self>,
        desc: WlShmPoolCreateBufferRequest,
    ) -> WlObjectHandle<WlBuffer> {
        let proxy = unsafe { desc.send(storage.object(shm_pool).proxy(), buf).unwrap() };

        storage.insert(WlObject::new(proxy, WlBuffer))
    }
}

impl HasObjectType for WlShmPool {
    const OBJECT_TYPE: ObjectType = ObjectType::ShmPool;
}

impl Dispatch for WlShmPool {}
