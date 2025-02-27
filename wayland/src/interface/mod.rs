pub mod callback;
pub mod compositor;
pub mod display;
pub mod registry;
pub mod shm;
pub mod shm_pool;
pub mod surface;
pub mod zwlr_layer_shell_v1;
pub mod zwlr_layer_surface_v1;

use super::object::ObjectId;
use crate::{
    c_api::wl_proxy,
    sys::{
        InterfaceObjectType,
        ffi::wl_proxy_marshal_array_constructor,
        proxy::AsProxy,
        wire::{Message, MessageBuffer, OpCode},
    },
};
use std::ptr;

pub use {
    callback::event::Done as WlCallbackDoneEvent,
    compositor::request::{
        CreateRegion as WlCompositorCreateRegion, CreateSurface as WlCompositorCreateSurface,
    },
    display::{
        event::{DeleteId as WlDisplayDeleteIdEvent, Error as WlDisplayErrorEvent},
        request::{GetRegistry as WlDisplayGetRegistryRequest, Sync as WlDisplaySyncRequest},
        wl_enum::Error as WlDisplayErrorEnum,
    },
    registry::{
        event::{Global as WlRegistryGlobalEvent, GlobalRemove as WlRegistryGlobalRemoveEvent},
        request::Bind as WlRegistryBindRequest,
    },
    shm::{request::CreatePool as WlShmCreatePoolRequest, wl_enum::Format as WlShmFormat},
    shm_pool::request::CreateBuffer as WlShmPoolCreateBufferRequest,
    surface::{
        event::{Enter as WlSurfaceEnterEvent, Leave as WlSurfaceLeaveEvent},
        request::{
            Attach as WlSurfaceAttachRequest, Commit as WlSurfaceCommitRequest,
            Damage as WlSurfaceDamageRequest, Destroy as WlSurfaceDestroyRequest,
            Frame as WlSurfaceFrameRequest, SetInputRegion as WlSurfaceSetInputRegionRequest,
            SetOpaqueRegion as SetOpaqueRegionRequest,
        },
        wl_enum::Error as WlSurfaceError,
    },
};

/// An [`ObjectId`] bundled with an interface name and a version
#[deprecated]
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct NewId<'s> {
    pub id: ObjectId,
    pub interface: &'s str,
    pub version: u32,
}

/// Represents requests on Wayland's interfaces
pub trait Request<'b>: Sized {
    /// The parent object for the request
    type ParentProxy: AsProxy;

    /// The opcode for the request
    const CODE: OpCode;
    const OUTGOING_INTERFACE: Option<InterfaceObjectType> = None;

    /// Builds the message on the top of given message buffer
    fn build_message(
        self,
        parent: &'b Self::ParentProxy,
        buf: &'b mut impl MessageBuffer,
    ) -> Message<'b>;

    unsafe fn send_raw(
        self,
        parent: &'b Self::ParentProxy,
        buf: &'b mut impl MessageBuffer,
    ) -> *mut wl_proxy {
        let message = self.build_message(parent, buf);
        let interface = Self::OUTGOING_INTERFACE
            .map(|i| &raw const *i.backend_interface())
            .unwrap_or(ptr::null());

        unsafe {
            wl_proxy_marshal_array_constructor(
                parent.as_proxy().as_raw().as_ptr(),
                message.opcode.into(),
                message.arguments.as_ptr().cast_mut(),
                interface,
            )
        }
    }
}

/// Represents events on Wayland's interfaces
pub trait Event<'s>: Sized {
    /// The opcode for the event
    const CODE: OpCode;

    /// Tries to read the given message as an event of implementor type
    fn from_message(message: Message<'s>) -> Option<Self>;
}
