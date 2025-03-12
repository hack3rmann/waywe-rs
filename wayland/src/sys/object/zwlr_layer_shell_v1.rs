use super::{output::WlOutput, surface::WlSurface, zwlr_layer_surface_v1::WlrLayerSurfaceV1, Dispatch, WlObject, WlObjectHandle};
use crate::{
    interface::{Request as _, ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellV1Layer},
    sys::{object_storage::WlObjectStorage, wire::MessageBuffer, HasObjectType, ObjectType},
};
use std::ffi::CStr;

#[derive(Debug, Default)]
pub struct WlrLayerShellV1;

impl WlrLayerShellV1 {
    pub fn get_layer_surface(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
        shell: WlObjectHandle<Self>,
        surface: WlObjectHandle<WlSurface>,
        output: Option<WlObjectHandle<WlOutput>>,
        layer: ZwlrLayerShellV1Layer,
        // TODO(hack3rmann): make new type for namespaces
        namespace: &CStr,
    ) -> Option<WlObjectHandle<WlrLayerSurfaceV1>> {
        let proxy = unsafe {
            ZwlrLayerShellGetLayerSurfaceRequest {
                surface: storage.object(surface).proxy(),
                output: output.map(|handle| storage.object(handle).proxy()),
                layer,
                namespace,
            }
            .send(storage.object(shell).proxy(), buf)?
        };

        Some(storage.insert(WlObject::new(proxy, WlrLayerSurfaceV1)))
    }
}

impl HasObjectType for WlrLayerShellV1 {
    const OBJECT_TYPE: ObjectType = ObjectType::WlrLayerShellV1;
}

impl Dispatch for WlrLayerShellV1 {}
