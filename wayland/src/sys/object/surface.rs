use super::{Dispatch, FromProxy, WlObject};
use crate::sys::{HasObjectType, ObjectType, proxy::WlProxy};
use raw_window_handle::{
    HandleError, HasWindowHandle, RawWindowHandle, WaylandWindowHandle, WindowHandle,
};

#[derive(Debug, Default)]
pub struct WlSurface;

impl HasObjectType for WlSurface {
    const OBJECT_TYPE: ObjectType = ObjectType::Surface;
}

impl FromProxy for WlSurface {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlSurface {}

impl HasWindowHandle for WlObject<WlSurface> {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Ok(unsafe {
            WindowHandle::borrow_raw(RawWindowHandle::Wayland(WaylandWindowHandle::new(
                self.proxy().as_raw(),
            )))
        })
    }
}
