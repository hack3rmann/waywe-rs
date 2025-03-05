use super::{Dispatch, WlObject};
use crate::{
    interface::registry::request::HasInterface,
    sys::{Interface, InterfaceObjectType, wire::Message},
};
use raw_window_handle::{
    HandleError, HasWindowHandle, RawWindowHandle, WaylandWindowHandle, WindowHandle,
};

#[derive(Debug, Default)]
pub struct WlSurface;

impl HasInterface for WlSurface {
    const INTERFACE: Interface = Interface {
        object_type: InterfaceObjectType::Surface,
        version: 6,
    };
}

impl Dispatch for WlSurface {
    fn dispatch(&mut self, _message: Message<'_>) {}
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
