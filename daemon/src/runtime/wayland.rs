use crate::event::EventEmitter;
use glam::UVec2;
use raw_window_handle::{
    HasDisplayHandle as _, RawDisplayHandle, RawWindowHandle, WaylandWindowHandle,
};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::CStr,
    pin::Pin,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering::*},
    },
};
use wayland_client::{
    interface::{
        WlCompositorCreateRegionRequest, WlCompositorCreateSurfaceRequest, WlOutputNameEvent,
        WlRegionAddRequest, WlRegionDestroyRequest, WlRegistryEvent, WlRegistryGlobalEvent,
        WlRegistryGlobalRemoveEvent, WlSurfaceCommitRequest, WlSurfaceSetBufferScaleRequest,
        WlSurfaceSetOpaqueRegionRequest, ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellLayer,
        ZwlrLayerSurfaceAckConfigureRequest, ZwlrLayerSurfaceAnchor,
        ZwlrLayerSurfaceConfigureEvent, ZwlrLayerSurfaceKeyboardInteractivity,
        ZwlrLayerSurfaceSetAnchorRequest, ZwlrLayerSurfaceSetExclusiveZoneRequest,
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest, ZwlrLayerSurfaceSetMarginRequest,
    },
    object::{HasObjectType, WlObjectId, WlObjectType},
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WaylandEvent {
    ResizeRequested { monitor_id: MonitorId, size: UVec2 },
    MonitorPlugged { id: MonitorId },
    MonitorUnplugged { id: MonitorId },
}

pub type MonitorId = WlObjectId;
pub type MonitorMap<T> = BTreeMap<MonitorId, T>;

#[derive(Default, Debug)]
pub struct MonitorInfo {
    pub size: Option<UVec2>,
    pub name: Option<Arc<str>>,
    pub output: WlObjectHandle<Output>,
    pub surface: WlObjectHandle<Surface>,
    pub layer_surface: WlObjectHandle<LayerSurface>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Globals {
    pub compositor: WlObjectHandle<Compositor>,
    pub layer_shell: WlObjectHandle<LayerShell>,
}

pub struct ClientState {
    pub events: Mutex<EventEmitter>,
    pub monitors: RwLock<MonitorMap<MonitorInfo>>,
    pub monitor_names: RwLock<HashMap<Arc<str>, MonitorId>>,
    pub globals: Option<Globals>,
    pub resize_requested: AtomicBool,
}

impl ClientState {
    pub fn new(events: EventEmitter) -> Self {
        Self {
            events: Mutex::new(events),
            monitors: RwLock::new(MonitorMap::default()),
            monitor_names: RwLock::new(HashMap::default()),
            globals: None,
            resize_requested: AtomicBool::new(false),
        }
    }

    pub fn monitor_size(&self, id: MonitorId) -> Option<UVec2> {
        self.monitors
            .read()
            .unwrap()
            .get(&id)
            .and_then(|info| info.size)
    }

    pub fn monitor_name(&self, id: MonitorId) -> Option<Arc<str>> {
        let monitors = self.monitors.read().unwrap();
        let monitor = monitors.get(&id).unwrap();
        Some(Arc::clone(monitor.name.as_ref()?))
    }

    pub fn monitor_id(&self, name: &str) -> Option<MonitorId> {
        let names = self.monitor_names.read().unwrap();
        names.get(name).copied()
    }

    pub fn aspect_ratio(&self, id: MonitorId) -> Option<f32> {
        let size = self.monitor_size(id)?;
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
    pub monitor_id: MonitorId,
    pub handle: WlObjectHandle<Self>,
    pub surface: WlObjectHandle<Surface>,
    pub compositor: WlObjectHandle<Compositor>,
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

        let size = UVec2::new(width, height);

        state.resize_requested.store(
            state.monitor_size(self.monitor_id) != Some(UVec2::ZERO)
                && state.monitor_size(self.monitor_id) != Some(size),
            Release,
        );

        {
            let mut monitors = state.monitors.write().unwrap();
            let monitor = monitors.get_mut(&self.monitor_id).unwrap();

            // this is resize if and only if monitor is ininialized
            // and size is changed indeed
            match monitor.size {
                Some(prev_size) if prev_size != size => {
                    state
                        .events
                        .lock()
                        .unwrap()
                        .emit(WaylandEvent::ResizeRequested {
                            monitor_id: self.monitor_id,
                            size,
                        })
                        .unwrap();
                }
                Some(_same_size) => {}
                None => {
                    let mut events = state.events.lock().unwrap();
                    events
                        .emit(WaylandEvent::MonitorPlugged {
                            id: self.monitor_id,
                        })
                        .unwrap();
                }
            }

            monitor.size = Some(size);
        }

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

pub struct Output {
    pub monitor_id: MonitorId,
}

impl HasObjectType for Output {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Output;
}

impl Dispatch for Output {
    type State = ClientState;

    fn dispatch(
        &mut self,
        state: &Self::State,
        _storage: &mut WlObjectStorage<Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(WlOutputNameEvent { name }) = message.as_event() else {
            return;
        };

        // Safety: name is an ASCII string which is a valid utf-8 string
        let name = Arc::from(unsafe { str::from_utf8_unchecked(name.to_bytes()) });

        let mut monitors = state.monitors.write().unwrap();
        let monitor = monitors.get_mut(&self.monitor_id).unwrap();
        monitor.name = Some(Arc::clone(&name));

        let mut names = state.monitor_names.write().unwrap();
        names.insert(name, self.monitor_id);
    }
}

pub fn handle_output(
    registry: WlObjectHandle<WlRegistry<ClientState>>,
    state: &ClientState,
    storage: Pin<&mut WlObjectStorage<ClientState>>,
    monitor_id: WlObjectId,
) {
    let Some(globals) = state.globals else {
        return;
    };

    let mut buf = WlStackMessageBuffer::new();
    let mut storage = Pin::new(storage);

    let output = registry
        .bind_from_fn_by_id(&mut buf, storage.as_mut(), monitor_id, |_, _, _| Output {
            monitor_id,
        })
        .unwrap();

    let surface: WlObjectHandle<Surface> = globals.compositor.create_object(
        &mut buf,
        storage.as_mut(),
        WlCompositorCreateSurfaceRequest,
    );

    let layer_surface: WlObjectHandle<LayerSurface> = globals.layer_shell.create_object_with(
        &mut buf,
        storage.as_mut(),
        ZwlrLayerShellGetLayerSurfaceRequest {
            surface: surface.id(),
            output: Some(output.id()),
            layer: ZwlrLayerShellLayer::Background,
            namespace: WLR_NAMESPACE,
        },
        move |proxy| LayerSurface {
            monitor_id,
            handle: WlObjectHandle::new(proxy.id()),
            surface,
            compositor: globals.compositor,
        },
    );

    layer_surface.request(
        &mut buf,
        &storage,
        ZwlrLayerSurfaceSetAnchorRequest {
            anchor: ZwlrLayerSurfaceAnchor::all(),
        },
    );

    layer_surface.request(
        &mut buf,
        &storage,
        ZwlrLayerSurfaceSetExclusiveZoneRequest { zone: -1 },
    );

    layer_surface.request(
        &mut buf,
        &storage,
        ZwlrLayerSurfaceSetMarginRequest {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        },
    );

    layer_surface.request(
        &mut buf,
        &storage,
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest {
            keyboard_interactivity: ZwlrLayerSurfaceKeyboardInteractivity::None,
        },
    );

    surface.request(
        &mut buf,
        &storage,
        WlSurfaceSetBufferScaleRequest { scale: 1 },
    );

    surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

    let mut monitors = state.monitors.write().unwrap();

    monitors.insert(
        monitor_id,
        MonitorInfo {
            output,
            surface,
            layer_surface,
            size: None,
            name: None,
        },
    );
}

pub(crate) fn handle_global(
    registry: &mut WlRegistry<ClientState>,
    state: &ClientState,
    storage: &mut WlObjectStorage<ClientState>,
    global: WlRegistryGlobalEvent<'_>,
) {
    if global.interface != c"wl_output" {
        return;
    }

    let monitor_id = unsafe { WlObjectId::new_unchecked(global.name) };

    handle_output(registry.handle(), state, Pin::new(storage), monitor_id);
}

pub(crate) fn handle_global_remove(
    registry: &mut WlRegistry<ClientState>,
    state: &ClientState,
    storage: &mut WlObjectStorage<ClientState>,
    global: WlRegistryGlobalRemoveEvent,
) {
    let global_name = WlObjectId::new(global.name).unwrap();

    if registry.type_of(global_name) != Some(WlObjectType::Output) {
        return;
    }

    let monitor_id = global_name;
    let mut monitors = state.monitors.write().unwrap();

    let Some(info) = monitors.remove(&monitor_id) else {
        return;
    };

    if let Some(name) = info.name {
        let mut names = state.monitor_names.write().unwrap();
        _ = names.remove(&name);
    }

    storage.release(info.output).unwrap();
    storage.release(info.surface).unwrap();
    storage.release(info.layer_surface).unwrap();

    {
        let mut events = state.events.lock().unwrap();
        events
            .emit(WaylandEvent::MonitorUnplugged { id: monitor_id })
            .unwrap();
    }
}

pub(crate) fn registry_dispatch(
    registry: &mut WlRegistry<ClientState>,
    state: &ClientState,
    storage: &mut WlObjectStorage<ClientState>,
    event: WlRegistryEvent<'_>,
) {
    match event {
        WlRegistryEvent::Global(global) => {
            handle_global(registry, state, storage, global);
        }
        WlRegistryEvent::GlobalRemove(global) => {
            handle_global_remove(registry, state, storage, global);
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

#[derive(Clone, Copy, Debug)]
pub struct MonitorSurface {
    pub surface: WlObjectHandle<Surface>,
    pub layer_surface: WlObjectHandle<LayerSurface>,
}

pub struct Wayland {
    pub client_state: Pin<Box<ClientState>>,
    pub main_queue: Pin<Box<WlEventQueue<ClientState>>>,
    pub display: WlDisplay<ClientState>,
    pub registry: WlObjectHandle<WlRegistry<ClientState>>,
}

impl Wayland {
    pub fn new(events: EventEmitter) -> Self {
        let mut client_state = Box::pin(ClientState::new(events));
        let display = WlDisplay::connect(client_state.as_ref()).unwrap();
        let mut queue = Box::pin(display.take_main_queue().unwrap());

        let mut buf = WlStackMessageBuffer::new();

        let registry = display
            .create_registry(&mut buf, queue.as_mut().storage_mut())
            .with_dispatcher(registry_dispatch)
            .handle();

        // fill the registry first
        display.roundtrip(queue.as_mut(), client_state.as_ref());

        let mut storage = queue.as_mut().storage_mut();

        let compositor = registry
            .bind::<Compositor>(&mut buf, storage.as_mut())
            .unwrap();

        let layer_shell = registry
            .bind::<LayerShell>(&mut buf, storage.as_mut())
            .unwrap();

        client_state.globals = Some(Globals {
            compositor,
            layer_shell,
        });

        let n_outputs = storage.object_data(registry).count_of(WlObjectType::Output);

        for output_index in 0..n_outputs {
            let monitor_id = storage
                .object_data(registry)
                .name_of_index(WlObjectType::Output, output_index)
                .unwrap();

            handle_output(registry, &client_state, storage.as_mut(), monitor_id);
        }

        Self {
            client_state,
            display,
            main_queue: queue,
            registry,
        }
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        self.display.display_handle().unwrap().as_raw()
    }
}
