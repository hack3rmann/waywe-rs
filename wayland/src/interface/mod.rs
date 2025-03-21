pub mod callback;
pub mod compositor;
pub mod display;
pub mod layer_shell;
pub mod layer_surface;
pub mod region;
pub mod registry;
pub mod shm;
pub mod shm_pool;
pub mod surface;
pub mod toplevel;
pub mod viewporter;
pub mod wm_base;
pub mod xdg_surface;

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

pub use {
    callback::event::Done as WlCallbackDoneEvent,
    generated::wayland::compositor::request::{
        CreateRegion as WlCompositorCreateRegionRequest,
        CreateSurface as WlCompositorCreateSurfaceRequest,
    },
    display::{
        event::{DeleteId as WlDisplayDeleteIdEvent, Error as WlDisplayErrorEvent},
        request::{GetRegistry as WlDisplayGetRegistryRequest, Sync as WlDisplaySyncRequest},
        wl_enum::Error as WlDisplayErrorEnum,
    },
    layer_shell::{
        request::{
            Destroy as WlLayerShellDestroyRequest,
            GetLayerSurface as WlLayerShellGetLayerSurfaceRequest,
        },
        wl_enum::Layer as WlLayerShellLayer,
    },
    layer_surface::{
        event::Configure as WlLayerSurfaceConfigureEvent,
        request::{
            AckConfigure as WlLayerSurfaceAckConfigureRequest,
            SetAnchor as WlLayerSurfaceSetAnchorRequest,
            SetExclusiveZone as WlLayerSurfaceSetExclusiveZoneRequest,
            SetKeyboardInteractivity as WlLayerSurfaceSetKeyboardInteractivityRequest,
            SetMargin as WlLayerSurfaceSetMarginRequest, SetSize as WlLayerSurfaceSetSizeRequest,
        },
    },
    region::request::Destroy as WlRegionDestroyRequest,
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
            Frame as WlSurfaceFrameRequest, SetBufferScale as WlSurfaceSetBufferScaleRequest,
            SetInputRegion as WlSurfaceSetInputRegionRequest,
            SetOpaqueRegion as WlSurfaceSetOpaqueRegionRequest,
        },
        wl_enum::Error as WlSurfaceError,
    },
    toplevel::{
        event::{Close as WlToplevelCloseEvent, Configure as WlToplevelConfigureEvent},
        request::{SetAppId as WlToplevelSetAppIdRequest, SetTitle as WlToplevelSetTitleRequest},
    },
    viewporter::request::GetViewport as WlViewporterGetViewportRequest,
    wm_base::{
        event::Ping as WlWmBasePingEvent,
        request::{GetXdgSurface as WlWmBaseGetXdgSurfaceRequest, Pong as WlWmBasePongRequest},
    },
    xdg_surface::{
        event::Configure as WlXdgSurfaceConfigureEvent,
        request::{
            AckConfigure as WlXdgSurfaceAckConfigureRequest,
            GetToplevel as WlXdgSurfaceGetToplevelRequest,
        },
    },
};

pub mod generated {
    wayland_scanner::include_interfaces!([
        "wayland-protocols/wayland.xml",
        "wayland-protocols/stable/xdg-shell/xdg-shell.xml",
        "wayland-protocols/stable/viewporter/viewporter.xml",
        "wayland-protocols/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml",
    ]);
}

pub trait ObjectParent {
    const CHILD_TYPE: generated::WlObjectType;
}

/// Represents requests on Wayland's interfaces
pub trait Request<'s>: Sized + HasObjectType {
    /// The opcode for the request
    const CODE: OpCode;

    /// The type of an interface object of which will be created by libwayland
    const OUTGOING_INTERFACE: Option<generated::WlObjectType> = None;

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
