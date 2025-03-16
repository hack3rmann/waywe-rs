use super::{Dispatch, FromProxy, WlObjectHandle};
use crate::{
    interface::{Event, LayerSurfaceAckConfigureRequest, LayerSurfaceConfigureEvent},
    object::{HasObjectType, WlObjectType},
    sys::{
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{WlMessage, SmallVecMessageBuffer},
    },
};
use std::pin::Pin;

#[derive(Debug, Default)]
pub struct WlrLayerSurfaceV1 {
    pub handle: WlObjectHandle<Self>,
}

impl HasObjectType for WlrLayerSurfaceV1 {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WlrLayerSurfaceV1;
}

impl FromProxy for WlrLayerSurfaceV1 {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

impl Dispatch for WlrLayerSurfaceV1 {
    fn dispatch(&mut self, storage: Pin<&mut WlObjectStorage>, message: WlMessage<'_>) {
        let Some(LayerSurfaceConfigureEvent { serial, .. }) =
            LayerSurfaceConfigureEvent::from_message(message)
        else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<4>::new();

        self.handle.request(
            &mut buf,
            storage.as_ref().get_ref(),
            LayerSurfaceAckConfigureRequest { serial },
        );
    }
}
