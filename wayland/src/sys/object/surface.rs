use super::{Dispatch, WlObject, WlObjectHandle, buffer::WlBuffer};
use crate::{
    interface::{
        registry::request::HasInterface, Request, WlSurfaceAttachRequest, WlSurfaceDamageRequest, WlSurfaceDestroyRequest
    },
    sys::{
        object_storage::WlObjectStorage, wire::{Message, MessageBuffer}, Interface, InterfaceObjectType
    },
};
use glam::{IVec2, UVec2};
use raw_window_handle::{
    HandleError, HasWindowHandle, RawWindowHandle, WaylandWindowHandle, WindowHandle,
};

#[derive(Debug, Default)]
pub struct WlSurface;

impl WlObject<WlSurface> {
    pub fn destroy(&self, buf: &mut impl MessageBuffer) {
        _ = unsafe { WlSurfaceDestroyRequest.send_raw(&self.proxy, buf) };
    }
}

impl WlSurface {
    pub fn attach(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
        surface: WlObjectHandle<Self>,
        buffer: WlObjectHandle<WlBuffer>,
        pos: IVec2,
    ) {
        _ = unsafe {
            WlSurfaceAttachRequest {
                buffer: Some(storage.object(buffer).proxy()),
                x: pos.x,
                y: pos.y,
            }
            .send_raw(storage.object(surface).proxy(), buf)
        };
    }

    pub fn damage(
        buf: &mut impl MessageBuffer,
        storage: &mut WlObjectStorage,
        surface: WlObjectHandle<Self>,
        pos: IVec2,
        size: UVec2,
    ) {
        _ = unsafe {
            WlSurfaceDamageRequest {
                x: pos.x,
                y: pos.y,
                width: size.x as i32,
                height: size.y as i32,
            }
            .send_raw(storage.object(surface).proxy(), buf)
        };
    }
}

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
