use super::{Dispatch, WlObject, WlObjectHandle, buffer::WlBuffer};
use crate::{
    interface::{Request, WlShmPoolCreateBufferRequest, registry::request::HasInterface},
    sys::{
        Interface, InterfaceObjectType,
        object_storage::WlObjectStorage,
        wire::{Message, MessageBuffer},
    },
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

impl HasInterface for WlShmPool {
    const INTERFACE: Interface = Interface {
        object_type: InterfaceObjectType::ShmPool,
        version: 2,
    };
}

impl Dispatch for WlShmPool {
    fn dispatch(&mut self, _: Message<'_>) {}
}
