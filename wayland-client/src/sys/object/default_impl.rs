use super::{WlObject, WlObjectHandle, dispatch::NoState};
use crate::{
    SmallVecMessageBuffer, WlObjectStorage,
    interface::{WlObjectType, Event, ZwlrLayerSurfaceAckConfigureRequest, ZwlrLayerSurfaceConfigureEvent},
    object::HasObjectType,
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
    ( $( $Name:ident ),* $(,)? ) => {
        $(
            #[derive(Debug, Default)]
            pub struct $Name;

            impl $crate::object::HasObjectType for $Name {
                const OBJECT_TYPE: $crate::object::WlObjectType = $crate::object::WlObjectType:: $Name;
            }

            impl $crate::sys::object::FromProxy for $Name {
                fn from_proxy(_: &$crate::sys::proxy::WlProxy) -> Self { Self }
            }

            impl $crate::sys::object::dispatch::Dispatch for $Name {
                type State = $crate::sys::object::dispatch::NoState;

                const ALLOW_EMPTY_DISPATCH: bool = true;

                fn dispatch(
                    &mut self,
                    _state: Pin<&mut Self::State>,
                    _storage: Pin<&mut WlObjectStorage<'_, Self::State>>,
                    _message: WlMessage<'_>,
                ) {
                    unreachable!()
                }
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
    WpViewport,
    WpViewporter,
    LayerShell,
}

impl HasWindowHandle for WlObject<Surface> {
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
    type State = NoState;

    fn dispatch(
        &mut self,
        _: Pin<&mut Self::State>,
        storage: Pin<&mut WlObjectStorage<'_, Self::State>>,
        message: WlMessage<'_>,
    ) {
        let Some(ZwlrLayerSurfaceConfigureEvent { serial, .. }) =
            ZwlrLayerSurfaceConfigureEvent::from_message(message)
        else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<1>::new();

        self.handle.request(
            &mut buf,
            storage.as_ref().get_ref(),
            ZwlrLayerSurfaceAckConfigureRequest { serial },
        );
    }
}
