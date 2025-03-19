use super::{WlObject, WlObjectHandle};
use crate::{
    SmallVecMessageBuffer, WlObjectStorage,
    interface::{Event, WlLayerSurfaceAckConfigureRequest, WlLayerSurfaceConfigureEvent},
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
    LayerShell,
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
pub struct WlLayerSurface {
    pub handle: WlObjectHandle<Self>,
}

impl HasObjectType for WlLayerSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
}

impl FromProxy for WlLayerSurface {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

impl Dispatch for WlLayerSurface {
    fn dispatch(&mut self, storage: Pin<&mut WlObjectStorage>, message: WlMessage<'_>) {
        let Some(WlLayerSurfaceConfigureEvent { serial, .. }) =
            WlLayerSurfaceConfigureEvent::from_message(message)
        else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<1>::new();

        self.handle.request(
            &mut buf,
            storage.as_ref().get_ref(),
            WlLayerSurfaceAckConfigureRequest { serial },
        );
    }
}
