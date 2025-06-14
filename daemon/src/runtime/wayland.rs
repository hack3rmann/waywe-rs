use glam::UVec2;
use raw_window_handle::{
    HasDisplayHandle as _, RawDisplayHandle, RawWindowHandle, WaylandWindowHandle,
};
use std::{
    ffi::CStr,
    pin::Pin,
    sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed},
};
use wayland_client::{
    WlSmallVecMessageBuffer,
    interface::{
        WlCompositorCreateSurfaceRequest, WlSurfaceCommitRequest, WlSurfaceSetBufferScaleRequest,
        ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellLayer,
        ZwlrLayerSurfaceAckConfigureRequest, ZwlrLayerSurfaceAnchor,
        ZwlrLayerSurfaceConfigureEvent, ZwlrLayerSurfaceKeyboardInteractivity,
        ZwlrLayerSurfaceSetAnchorRequest, ZwlrLayerSurfaceSetExclusiveZoneRequest,
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest, ZwlrLayerSurfaceSetMarginRequest,
    },
    object::{HasObjectType, WlObjectType},
    sys::{
        display::WlDisplay,
        object::{
            FromProxy, WlObject, WlObjectHandle, dispatch::Dispatch, event_queue::WlEventQueue,
            registry::WlRegistry,
        },
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{WlMessage, WlStackMessageBuffer},
    },
};

#[derive(Default, Debug)]
pub struct ClientState {
    pub monitor_width: AtomicU32,
    pub monitor_height: AtomicU32,
    pub resize_requested: AtomicBool,
}

impl ClientState {
    pub fn monitor_size(&self) -> UVec2 {
        UVec2::new(
            self.monitor_width.load(Relaxed),
            self.monitor_height.load(Relaxed),
        )
    }
}

pub struct Compositor;

impl HasObjectType for Compositor {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
}

impl Dispatch for Compositor {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for Compositor {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

pub struct LayerShell;

impl HasObjectType for LayerShell {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerShell;
}

impl Dispatch for LayerShell {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for LayerShell {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

pub struct Surface;

impl HasObjectType for Surface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
}

impl Dispatch for Surface {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for Surface {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

pub struct LayerSurface {
    handle: WlObjectHandle<Self>,
}

impl HasObjectType for LayerSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
}

impl Dispatch for LayerSurface {
    type State = ClientState;

    fn dispatch(
        &mut self,
        state: &Self::State,
        storage: &mut WlObjectStorage<Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(ZwlrLayerSurfaceConfigureEvent {
            serial,
            width,
            height,
        }) = message.as_event()
        else {
            return;
        };

        state.resize_requested.store(
            state.monitor_size() != UVec2::ZERO
                && state.monitor_size() != UVec2::new(width, height),
            Relaxed,
        );
        state.monitor_width.store(width, Relaxed);
        state.monitor_height.store(height, Relaxed);

        let mut buf = WlSmallVecMessageBuffer::<3>::new();

        self.handle.request(
            &mut buf,
            storage,
            ZwlrLayerSurfaceAckConfigureRequest { serial },
        );
    }
}

impl FromProxy for LayerSurface {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

pub const WLR_NAMESPACE: &CStr = c"waywe-runtime";

pub trait SurfaceExtension {
    fn raw_window_handle(&self) -> RawWindowHandle;
}

impl SurfaceExtension for WlObject<Surface> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Wayland(WaylandWindowHandle::new(self.proxy().as_raw()))
    }
}

pub struct WaylandHandle {
    pub registry: WlObjectHandle<WlRegistry<ClientState>>,
    pub compositor: WlObjectHandle<Compositor>,
    pub layer_shell: WlObjectHandle<LayerShell>,
    pub surface: WlObjectHandle<Surface>,
    pub layer_surface: WlObjectHandle<LayerSurface>,
}

pub struct Wayland {
    pub(crate) client_state: Pin<Box<ClientState>>,
    pub(crate) main_queue: Pin<Box<WlEventQueue<ClientState>>>,
    pub(crate) display: WlDisplay<ClientState>,
    pub handle: WaylandHandle,
}

impl Wayland {
    pub fn new() -> Self {
        let client_state = Box::pin(ClientState::default());
        let display = WlDisplay::connect(client_state.as_ref()).unwrap();
        let mut main_queue = Box::pin(display.take_main_queue().unwrap());

        let mut buf = WlStackMessageBuffer::new();

        let registry = display.create_registry(&mut buf, main_queue.as_mut().storage_mut());

        display.roundtrip(main_queue.as_mut(), client_state.as_ref());

        let compositor = registry
            .bind::<Compositor>(&mut buf, main_queue.as_mut().storage_mut())
            .unwrap();

        let layer_shell = registry
            .bind::<LayerShell>(&mut buf, main_queue.as_mut().storage_mut())
            .unwrap();

        let surface: WlObjectHandle<Surface> = compositor.create_object(
            &mut buf,
            main_queue.as_mut().storage_mut(),
            WlCompositorCreateSurfaceRequest,
        );

        let layer_surface: WlObjectHandle<LayerSurface> = layer_shell.create_object(
            &mut buf,
            main_queue.as_mut().storage_mut(),
            ZwlrLayerShellGetLayerSurfaceRequest {
                surface: surface.id(),
                output: None,
                layer: ZwlrLayerShellLayer::Background,
                namespace: WLR_NAMESPACE,
            },
        );

        layer_surface.request(
            &mut buf,
            &main_queue.as_ref().storage(),
            ZwlrLayerSurfaceSetAnchorRequest {
                anchor: ZwlrLayerSurfaceAnchor::all(),
            },
        );

        layer_surface.request(
            &mut buf,
            &main_queue.as_ref().storage(),
            ZwlrLayerSurfaceSetExclusiveZoneRequest { zone: -1 },
        );

        layer_surface.request(
            &mut buf,
            &main_queue.as_ref().storage(),
            ZwlrLayerSurfaceSetMarginRequest {
                top: 0,
                right: 0,
                bottom: 0,
                left: 0,
            },
        );

        layer_surface.request(
            &mut buf,
            &main_queue.as_ref().storage(),
            ZwlrLayerSurfaceSetKeyboardInteractivityRequest {
                keyboard_interactivity: ZwlrLayerSurfaceKeyboardInteractivity::None,
            },
        );

        surface.request(
            &mut buf,
            &main_queue.as_ref().storage(),
            WlSurfaceSetBufferScaleRequest { scale: 1 },
        );

        surface.request(
            &mut buf,
            &main_queue.as_ref().storage(),
            WlSurfaceCommitRequest,
        );

        display.roundtrip(main_queue.as_mut(), client_state.as_ref());

        let screen_size = client_state.monitor_size();

        assert_ne!(screen_size.x, 0);
        assert_ne!(screen_size.y, 0);

        Self {
            client_state,
            display,
            main_queue,
            handle: WaylandHandle {
                registry,
                compositor,
                layer_shell,
                surface,
                layer_surface,
            },
        }
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        self.display.display_handle().unwrap().as_raw()
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        self.main_queue
            .as_ref()
            .storage()
            .object(self.handle.surface)
            .raw_window_handle()
    }
}

impl Default for Wayland {
    fn default() -> Self {
        Self::new()
    }
}
