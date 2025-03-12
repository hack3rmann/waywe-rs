use super::{Dispatch, WlObject, WlObjectHandle, surface::WlSurface};
use crate::{
    interface::{Request, WlCompositorCreateSurface},
    sys::{object_storage::WlObjectStorage, wire::MessageBuffer, HasObjectType, ObjectType},
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

impl HasObjectType for WlCompositor {
    const OBJECT_TYPE: ObjectType = ObjectType::Compositor;
}

impl Dispatch for WlCompositor {}
