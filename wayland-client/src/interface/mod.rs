//! Data structs representing all wayland requests, events and enums

pub mod registry;

/// Interfaces generated via `wayland-scanner`
pub mod generated {
    wayland_scanner::include_interfaces!([
        "wayland-protocols/wayland.xml",
        "wayland-protocols/stable/xdg-shell/xdg-shell.xml",
        "wayland-protocols/stable/viewporter/viewporter.xml",
        "wayland-protocols/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml",
    ]);
}

use crate::{
    ffi,
    object::HasObjectType,
    sys::{
        object::dispatch::State,
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{OpCode, WlMessage, WlMessageBuffer},
    },
};
use std::ptr::{self, NonNull};

pub use generated::WlObjectType;
pub use generated::prelude::*;
pub use registry::{
    event::Global as WlRegistryGlobalEvent, request::Bind as WlRegistryBindRequest,
};

impl WlObjectType {
    /// The name of this interface
    pub const fn interface_name(self) -> &'static str {
        unsafe { self.interface().name_str_unchecked() }
    }

    /// The name for the particular request in this interface
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

/// Indicator that request produces an object
pub trait ObjectParent {
    /// Type of the child object
    const CHILD_TYPE: WlObjectType;
}

/// Represents requests on Wayland's interfaces
pub trait Request<'s>: Sized + HasObjectType {
    /// The opcode for the request
    const CODE: OpCode;

    /// The type of an interface object of which will be created by libwayland
    const CHILD_TYPE: Option<WlObjectType>;

    /// Builds the message on top of the given message buffer
    fn build_message<'m, S: State>(
        self,
        buf: &'m mut impl WlMessageBuffer,
        storage: &'m WlObjectStorage<S>,
    ) -> WlMessage<'m>
    where
        's: 'm;
}

/// Represents events on Wayland's interfaces
pub trait Event<'s>: Sized {
    /// The opcode for the event
    const CODE: OpCode;

    /// Tries to read the given message as an event of implementor type
    fn from_message(message: WlMessage<'s>) -> Option<Self>;
}

/// # Safety
///
/// - `parent` proxy must match the parent interface
/// - resulting `WlProxy` object must be consumed to an object storage
pub(crate) unsafe fn send_request_raw<'s, S: State, R: Request<'s>>(
    request: R,
    buf: &mut impl WlMessageBuffer,
    storage: &WlObjectStorage<S>,
    parent: &WlProxy,
) -> Option<WlProxy> {
    let message = request.build_message(buf, storage);
    let interface = R::CHILD_TYPE
        .map(|i| &raw const *i.backend_interface())
        .unwrap_or(ptr::null());

    let raw_proxy = unsafe {
        ffi::wl_proxy_marshal_array_constructor(
            parent.as_raw().as_ptr(),
            message.opcode.into(),
            message.arguments.as_ptr().cast_mut(),
            interface,
        )
    };

    Some(unsafe { WlProxy::from_raw(NonNull::new(raw_proxy)?) })
}
