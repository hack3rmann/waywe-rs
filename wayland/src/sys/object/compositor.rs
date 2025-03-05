use super::{Dispatch, WlObject, WlObjectHandle, surface::WlSurface};
use crate::{
    interface::{Request, WlCompositorCreateSurface, registry::request::HasInterface},
    sys::{
        Interface, InterfaceObjectType,
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{Message, MessageBuffer},
    },
};
use std::ptr::NonNull;

#[derive(Default)]
pub struct WlCompositor;

impl WlCompositor {
    pub fn create_surface(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
        compositor_handle: WlObjectHandle<Self>,
    ) -> WlObjectHandle<WlSurface> {
        let compositor = storage.object(compositor_handle);
        let raw_proxy = unsafe { WlCompositorCreateSurface.send_raw(&compositor.proxy, buf) };

        let proxy = unsafe { WlProxy::from_raw(NonNull::new(raw_proxy).unwrap()) };

        storage.insert(WlObject::new(proxy, WlSurface))
    }
}

impl HasInterface for WlCompositor {
    const INTERFACE: Interface = Interface {
        object_type: InterfaceObjectType::Compositor,
        version: 6,
    };
}

impl Dispatch for WlCompositor {
    fn dispatch(&mut self, _message: Message<'_>) {}
}
