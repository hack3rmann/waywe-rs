pub mod registry;
pub mod generated {
    wayland_scanner::include_interfaces!([
        "wayland-protocols/wayland.xml",
        "wayland-protocols/stable/xdg-shell/xdg-shell.xml",
        "wayland-protocols/stable/viewporter/viewporter.xml",
        "wayland-protocols/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml",
    ]);
}

use crate::{
    object::HasObjectType,
    sys::{
        object::dispatch::State,
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{MessageBuffer, OpCode, WlMessage},
    },
};
use std::ptr::{self, NonNull};
use wayland_sys::wl_proxy_marshal_array_constructor;

pub use generated::prelude::*;
pub use registry::{
    event::Global as WlRegistryGlobalEvent, request::Bind as WlRegistryBindRequest,
};

impl WlObjectType {
    /// The name of this interface
    pub const fn interface_name(self) -> &'static str {
        unsafe { self.interface().name_str_unchecked() }
    }

    /// The for the particular request in this interface
    ///
    /// # Error
    ///
    /// Will return [`None`] if there is no request for this `opcode`
    pub const fn request_name(self, opcode: OpCode) -> Option<&'static str> {
        let index = opcode as usize;

        // HACK(const-fn): should be slice.get(index)
        if index >= self.interface().methods.len() {
            return None;
        }

        Some(unsafe { self.interface().methods[index].name_str_unchecked() })
    }

    /// The for the particular event in this interface
    ///
    /// # Error
    ///
    /// Will return [`None`] if there is no event for this `opcode`
    pub const fn event_name(self, opcode: OpCode) -> Option<&'static str> {
        let index = opcode as usize;

        // HACK(const-fn): should be slice.get(index)
        if index >= self.interface().methods.len() {
            return None;
        }

        Some(unsafe { self.interface().events[opcode as usize].name_str_unchecked() })
    }
}

pub use generated::WlObjectType;

pub trait ObjectParent {
    const CHILD_TYPE: WlObjectType;
}

/// Represents requests on Wayland's interfaces
pub trait Request<'s>: Sized + HasObjectType {
    /// The opcode for the request
    const CODE: OpCode;

    /// The type of an interface object of which will be created by libwayland
    const OUTGOING_INTERFACE: Option<WlObjectType> = None;

    /// Builds the message on the top of given message buffer
    fn build_message<'m, S: State>(
        self,
        buf: &'m mut impl MessageBuffer,
        storage: &'m WlObjectStorage<'_, S>,
    ) -> WlMessage<'m>
    where
        's: 'm;

    /// # Safety
    ///
    /// - `parent` proxy should match the parent interface
    /// - resulting `WlProxy` object should be owned
    unsafe fn send<S: State>(
        self,
        buf: &mut impl MessageBuffer,
        storage: &WlObjectStorage<'_, S>,
        parent: &WlProxy,
    ) -> Option<WlProxy> {
        let message = self.build_message(buf, storage);
        let interface = Self::OUTGOING_INTERFACE
            .map(|i| &raw const *i.backend_interface())
            .unwrap_or(ptr::null());

        let raw_proxy = unsafe {
            wl_proxy_marshal_array_constructor(
                parent.as_raw().as_ptr(),
                message.opcode.into(),
                message.arguments.as_ptr().cast_mut(),
                interface,
            )
        };

        Some(unsafe { WlProxy::from_raw(NonNull::new(raw_proxy)?) })
    }
}

/// Represents events on Wayland's interfaces
pub trait Event<'s>: Sized {
    /// The opcode for the event
    const CODE: OpCode;

    /// Tries to read the given message as an event of implementor type
    fn from_message(message: WlMessage<'s>) -> Option<Self>;
}
