use super::{WlObject, WlObjectHandle};
use crate::{
    SmallVecMessageBuffer, WlObjectStorage,
    interface::{Event, LayerSurfaceAckConfigureRequest, LayerSurfaceConfigureEvent},
    object::{HasObjectType, WlObjectType},
    sys::{
        object::{Dispatch, FromProxy},
        proxy::WlProxy,
        wire::WlMessage,
    },
};
use raw_window_handle::{
    HandleError, HasWindowHandle, RawWindowHandle, WaylandWindowHandle, WindowHandle,
};
use std::pin::Pin;

macro_rules! define_empty_dispatchers {
    ( $( $name:ident ),* $(,)? ) => {
        $(
            ::paste::paste! {
                #[derive(Debug, Default)]
                pub struct [< Wl $name >];

                impl $crate::object::HasObjectType for [< Wl $name >] {
                    const OBJECT_TYPE: $crate::object::WlObjectType = $crate::object::WlObjectType:: $name;
                }

                impl $crate::sys::object::FromProxy for [< Wl $name >] {
                    fn from_proxy(_: &$crate::sys::proxy::WlProxy) -> Self { Self }
                }

                impl $crate::sys::object::Dispatch for [< Wl $name >] {}
            }
        )*
    };
}

define_empty_dispatchers! {
    Buffer,
    Callback,
    Compositor,
    Output,
    Region,
    Shm,
    ShmPool,
    Surface,
    Viewport,
    Viewporter,
}

impl HasWindowHandle for WlObject<WlSurface> {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Ok(unsafe {
            WindowHandle::borrow_raw(RawWindowHandle::Wayland(WaylandWindowHandle::new(
                self.proxy().as_raw(),
            )))
        })
    }
}

#[derive(Debug, Default)]
pub struct WlrLayerShellV1;

impl HasObjectType for WlrLayerShellV1 {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WlrLayerShellV1;
}

impl FromProxy for WlrLayerShellV1 {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlrLayerShellV1 {}

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
