use super::Dispatch;
use crate::{
    interface::registry::request::HasInterface,
    sys::{Interface, InterfaceObjectType, wire::Message},
};

#[derive(Default)]
pub struct WlCompositor;

impl HasInterface for WlCompositor {
    const INTERFACE: Interface = Interface {
        object_type: InterfaceObjectType::Compositor,
        version: 6,
    };
}

impl Dispatch for WlCompositor {
    fn dispatch(&mut self, _message: Message<'_>) {}
}
