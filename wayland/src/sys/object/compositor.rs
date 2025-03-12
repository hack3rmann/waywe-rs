use super::{Dispatch, WlObject, WlObjectHandle, surface::WlSurface};
use crate::{
    interface::{Request, WlCompositorCreateSurface, registry::request::HasInterface},
    sys::{
        Interface, InterfaceObjectType,
        object_storage::WlObjectStorage,
        wire::{Message, MessageBuffer},
    },
};

#[derive(Default)]
pub struct WlCompositor;

impl WlCompositor {
    pub fn create_surface(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
        compositor_handle: WlObjectHandle<Self>,
    ) -> WlObjectHandle<WlSurface> {
        let compositor = storage.object(compositor_handle);
        let proxy = unsafe {
            WlCompositorCreateSurface
                .send(&compositor.proxy, buf)
                .unwrap()
        };

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
