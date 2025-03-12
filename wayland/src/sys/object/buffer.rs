use super::Dispatch;
use crate::{
    interface::registry::request::HasInterface,
    sys::{Interface, InterfaceObjectType, wire::Message},
};

#[derive(Debug)]
pub struct WlBuffer;

impl HasInterface for WlBuffer {
    const INTERFACE: Interface = Interface {
        object_type: InterfaceObjectType::Buffer,
        version: 1,
    };
}

impl Dispatch for WlBuffer {
    fn dispatch(&mut self, _: Message<'_>) {}
}
