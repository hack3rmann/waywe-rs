use glam::UVec2;
use raw_window_handle::{
    HasDisplayHandle as _, RawDisplayHandle, RawWindowHandle, WaylandWindowHandle,
};
use smallvec::SmallVec;
use std::{
    ffi::CStr,
    pin::Pin,
    sync::atomic::{AtomicBool, AtomicU32, Ordering::*},
};
use wayland_client::{
    interface::{
        WlCompositorCreateRegionRequest, WlCompositorCreateSurfaceRequest, WlRegionAddRequest,
        WlRegionDestroyRequest, WlSurfaceCommitRequest, WlSurfaceSetBufferScaleRequest,
        WlSurfaceSetOpaqueRegionRequest, ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellLayer,
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
pub struct MonitorSize {
    pub width: AtomicU32,
    pub height: AtomicU32,
}

impl MonitorSize {
    pub fn get(&self) -> UVec2 {
        UVec2::new(self.width.load(Relaxed), self.height.load(Relaxed))
    }
}

#[derive(Default, Debug)]
pub struct ClientState {
    pub monitors: SmallVec<[MonitorSize; 2]>,
    pub resize_requested: AtomicBool,
}

impl ClientState {
    pub fn monitor_size(&self, index: usize) -> Option<UVec2> {
        Some(self.monitors.get(index)?.get())
    }

    pub fn aspect_ratio(&self, index: usize) -> Option<f32> {
        let size = self.monitor_size(index)?;
        Some(size.x as f32 / size.y as f32)
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
    monitor_index: usize,
    handle: WlObjectHandle<Self>,
    surface: WlObjectHandle<Surface>,
    compositor: WlObjectHandle<Compositor>,
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
            state.monitor_size(self.monitor_index) != Some(UVec2::ZERO)
                && state.monitor_size(self.monitor_index) != Some(UVec2::new(width, height)),
            Release,
        );
        state.monitors[self.monitor_index]
            .width
            .store(width, Relaxed);
        state.monitors[self.monitor_index]
            .height
            .store(height, Relaxed);

        let mut buf = WlStackMessageBuffer::new();

        self.handle.request(
            &mut buf,
            storage,
            ZwlrLayerSurfaceAckConfigureRequest { serial },
        );

        let mut storage = Pin::new(storage);

        let region: WlObjectHandle<Region> = self.compositor.create_object(
            &mut buf,
            storage.as_mut(),
            WlCompositorCreateRegionRequest,
        );

        region.request(
            &mut buf,
            &storage.as_ref(),
            WlRegionAddRequest {
                x: 0,
                y: 0,
                width: width.cast_signed(),
                height: height.cast_signed(),
            },
        );

        self.surface.request(
            &mut buf,
            &storage.as_ref(),
            WlSurfaceSetOpaqueRegionRequest {
                region: Some(region.id()),
            },
        );

        region.request(&mut buf, &storage.as_ref(), WlRegionDestroyRequest);

        storage.as_mut().release(region).unwrap();

        self.surface
            .request(&mut buf, &storage.as_ref(), WlSurfaceCommitRequest);
    }
}

pub struct Region;

impl Dispatch for Region {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for Region {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl HasObjectType for Region {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Region;
}

pub struct Output;

impl HasObjectType for Output {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Output;
}

impl Dispatch for Output {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for Output {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
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

#[derive(Clone, Copy, Debug)]
pub struct MonitorSurface {
    pub surface: WlObjectHandle<Surface>,
    pub layer_surface: WlObjectHandle<LayerSurface>,
}

pub struct WaylandHandle {
    pub registry: WlObjectHandle<WlRegistry<ClientState>>,
    pub compositor: WlObjectHandle<Compositor>,
    pub outputs: SmallVec<[WlObjectHandle<Output>; 4]>,
    pub surfaces: SmallVec<[MonitorSurface; 2]>,
}

pub struct Wayland {
    pub(crate) client_state: Pin<Box<ClientState>>,
    pub(crate) main_queue: Pin<Box<WlEventQueue<ClientState>>>,
    pub(crate) display: WlDisplay<ClientState>,
    pub handle: WaylandHandle,
}

impl Wayland {
    pub fn new() -> Self {
        let mut client_state = Box::pin(ClientState::default());
        let display = WlDisplay::connect(client_state.as_ref()).unwrap();
        let mut main_queue = Box::pin(display.take_main_queue().unwrap());

        let mut buf = WlStackMessageBuffer::new();

        let registry = display.create_registry(&mut buf, main_queue.as_mut().storage_mut());

        display.roundtrip(main_queue.as_mut(), client_state.as_ref());

        let outputs = registry
            .bind_all::<Output>(&mut buf, main_queue.as_mut().storage_mut())
            .collect::<Option<SmallVec<[WlObjectHandle<Output>; 4]>>>()
            .unwrap();

        assert!(!outputs.is_empty());

        let compositor = registry
            .bind::<Compositor>(&mut buf, main_queue.as_mut().storage_mut())
            .unwrap();

        // TODO(hack3rmann): handle it with wl_registry.global/wl_registry.global_remove events
        let surfaces = outputs
            .iter()
            .enumerate()
            .map(|(monitor_index, &output)| {
                client_state.monitors.push(MonitorSize::default());

                let layer_shell = registry
                    .bind::<LayerShell>(&mut buf, main_queue.as_mut().storage_mut())
                    .unwrap();

                let surface: WlObjectHandle<Surface> = compositor.create_object(
                    &mut buf,
                    main_queue.as_mut().storage_mut(),
                    WlCompositorCreateSurfaceRequest,
                );

                let layer_surface: WlObjectHandle<LayerSurface> = layer_shell.create_object_with(
                    &mut buf,
                    main_queue.as_mut().storage_mut(),
                    ZwlrLayerShellGetLayerSurfaceRequest {
                        surface: surface.id(),
                        output: Some(output.id()),
                        layer: ZwlrLayerShellLayer::Background,
                        namespace: WLR_NAMESPACE,
                    },
                    move |proxy| LayerSurface {
                        monitor_index,
                        handle: WlObjectHandle::new(proxy.id()),
                        surface,
                        compositor,
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

                MonitorSurface {
                    surface,
                    layer_surface,
                }
            })
            .collect();

        display.roundtrip(main_queue.as_mut(), client_state.as_ref());

        for monitor_size in &client_state.monitors {
            let screen_size = monitor_size.get();

            assert_ne!(screen_size.x, 0);
            assert_ne!(screen_size.y, 0);
        }

        Self {
            client_state,
            display,
            main_queue,
            handle: WaylandHandle {
                registry,
                compositor,
                outputs,
                surfaces,
            },
        }
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        self.display.display_handle().unwrap().as_raw()
    }

    /// # Note
    ///
    /// Guaranteed to have at least one entry
    pub fn raw_window_handles(&self) -> impl Iterator<Item = RawWindowHandle> {
        self.handle.surfaces.iter().map(|&surface| {
            self.main_queue
                .as_ref()
                .storage()
                .object(surface.surface)
                .raw_window_handle()
        })
    }
}

impl Default for Wayland {
    fn default() -> Self {
        Self::new()
    }
}
