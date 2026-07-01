use calloop::{
    EventIterator, EventSource, Interest, Mode, Poll, PostAction, Readiness, Token, TokenFactory,
};
use glam::UVec2;
use raw_window_handle::{
    HasDisplayHandle as _, RawDisplayHandle, RawWindowHandle, WaylandWindowHandle,
};
use rustix::io::Errno;
use smallstr::SmallString;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::CStr,
    ops::Deref,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
};
use thiserror::Error;
use wayland_client::{
    interface::{
        WlCompositorCreateRegionRequest, WlCompositorCreateSurfaceRequest, WlOutputMode,
        WlOutputModeEvent, WlOutputNameEvent, WlPointerEvent, WlRegionAddRequest,
        WlRegionDestroyRequest, WlRegistryEvent, WlRegistryGlobalEvent,
        WlRegistryGlobalRemoveEvent, WlSeatCapabilitiesEvent, WlSeatCapability,
        WlSeatGetPointerRequest, WlSurfaceCommitRequest, WlSurfaceSetBufferScaleRequest,
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

#[derive(Clone, Debug, PartialEq)]
pub enum WaylandEvent {
    ResizeRequested { monitor_id: MonitorId, size: UVec2 },
    MonitorPlugged { id: MonitorId, name: MonitorName },
    MonitorUnplugged { id: MonitorId, name: MonitorName },
    // TODO(hack3rmann): implement approach from <https://github.com/cjacker/wl-find-cursor/blob/main/main.c>
    CursorMoved { position: UVec2 },
}

pub type MonitorId = WlObjectId;
pub type MonitorMap<T> = BTreeMap<MonitorId, T>;
pub type MonitorName = SmallString<[u8; 32]>;

#[derive(Default, Debug)]
pub struct MonitorInfo {
    pub size: UVec2,
    pub name: MonitorName,
    pub output: WlObjectHandle<Output>,
    pub surface: WlObjectHandle<Surface>,
    pub layer_surface: WlObjectHandle<LayerSurface>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Globals {
    pub compositor: WlObjectHandle<Compositor>,
    pub layer_shell: WlObjectHandle<LayerShell>,
}

#[derive(Default)]
pub struct ClientState {
    pub stored_events: Mutex<Vec<WaylandEvent>>,
    pub monitors: RwLock<MonitorMap<MonitorInfo>>,
    pub monitor_names: RwLock<HashMap<MonitorName, MonitorId>>,
    pub globals: Option<Globals>,
}

impl ClientState {
    pub fn monitor_size(&self, id: MonitorId) -> Option<UVec2> {
        let monitors = self.monitors.read().unwrap();
        monitors.get(&id).map(|info| info.size)
    }

    pub fn monitor_name(&self, id: MonitorId) -> Option<MonitorName> {
        let monitors = self.monitors.read().unwrap();
        Some(monitors.get(&id)?.name.clone())
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

#[derive(Default)]
pub struct Compositor;

impl HasObjectType for Compositor {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
}

impl Dispatch for Compositor {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

pub struct Seat {
    handle: WlObjectHandle<Self>,
    pointer: Option<WlObjectHandle<Pointer>>,
}

impl FromProxy for Seat {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
            pointer: None,
        }
    }
}

impl HasObjectType for Seat {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Seat;
}

impl Dispatch for Seat {
    type State = ClientState;

    fn dispatch(
        &mut self,
        _: &Self::State,
        storage: &mut WlObjectStorage<Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(event) = message.as_event::<WlSeatCapabilitiesEvent>() else {
            return;
        };

        if !event.capabilities.contains(WlSeatCapability::POINTER) {
            return;
        }

        let mut storage = Pin::new(storage);
        let mut buf = WlStackMessageBuffer::new();

        let pointer: WlObjectHandle<Pointer> =
            self.handle
                .create_object(&mut buf, storage.as_mut(), WlSeatGetPointerRequest);

        self.pointer = Some(pointer)
    }
}

#[derive(Default)]
pub struct Pointer;

impl HasObjectType for Pointer {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Pointer;
}

impl Dispatch for Pointer {
    type State = ClientState;

    fn dispatch(
        &mut self,
        state: &Self::State,
        _storage: &mut WlObjectStorage<Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(event) = message.as_event::<WlPointerEvent>() else {
            return;
        };

        let WlPointerEvent::Motion(motion) = event else {
            return;
        };

        let position = UVec2::new(
            motion.surface_x.to_int().cast_unsigned(),
            motion.surface_y.to_int().cast_unsigned(),
        );

        let mut events = state.stored_events.lock().unwrap();
        events.push(WaylandEvent::CursorMoved { position });
    }
}

#[derive(Default)]
pub struct LayerShell;

impl HasObjectType for LayerShell {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerShell;
}

impl Dispatch for LayerShell {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

#[derive(Default)]
pub struct Surface;

impl HasObjectType for Surface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
}

impl Dispatch for Surface {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

pub struct LayerSurface {
    pub is_initial_configure_done: bool,
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

        {
            let mut monitors = state.monitors.write().unwrap();
            let mut events = state.stored_events.lock().unwrap();
            let monitor = monitors.get_mut(&self.monitor_id).unwrap();

            let is_resized = monitor.size != size;
            monitor.size = size;

            if !self.is_initial_configure_done {
                events.push(WaylandEvent::MonitorPlugged {
                    id: self.monitor_id,
                    name: monitor.name.clone(),
                });
            } else if is_resized {
                events.push(WaylandEvent::ResizeRequested {
                    monitor_id: self.monitor_id,
                    size,
                });
            }
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
                width: size.x.cast_signed(),
                height: size.y.cast_signed(),
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

        self.is_initial_configure_done = true;
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
    pub output_id: WlObjectId,
    pub size: Option<UVec2>,
    pub name: Option<MonitorName>,
    pub is_init_done: bool,
}

impl Output {
    pub const fn new(monitor_id: MonitorId, output_id: WlObjectId) -> Self {
        Self {
            monitor_id,
            output_id,
            size: None,
            name: None,
            is_init_done: false,
        }
    }

    pub fn handle_name(&mut self, event: WlOutputNameEvent) {
        // Safety: name is an ASCII string which is a valid utf-8 string
        let name = unsafe { str::from_utf8_unchecked(event.name.to_bytes()) };
        self.name = Some(MonitorName::from_str(name));
    }

    pub fn handle_mode(&mut self, event: WlOutputModeEvent) {
        if !event.flags.contains(WlOutputMode::CURRENT) {
            return;
        }

        self.size = Some(UVec2::new(
            u32::try_from(event.width).unwrap(),
            u32::try_from(event.height).unwrap(),
        ));
    }

    pub fn try_init_info(
        &mut self,
        state: &ClientState,
        storage: &mut WlObjectStorage<ClientState>,
    ) {
        if self.is_init_done {
            return;
        }

        let Some(globals) = state.globals else { return };
        let Some(size) = self.size else { return };
        let Some(name) = self.name.clone() else {
            return;
        };

        {
            let mut names = state.monitor_names.write().unwrap();
            names.insert(name.clone(), self.monitor_id);
        }

        let mut buf = WlStackMessageBuffer::new();
        let mut storage = Pin::new(storage);

        let surface: WlObjectHandle<Surface> = globals.compositor.create_object(
            &mut buf,
            storage.as_mut(),
            WlCompositorCreateSurfaceRequest,
        );

        let monitor_id = self.monitor_id;
        let layer_surface: WlObjectHandle<LayerSurface> = globals.layer_shell.create_object_with(
            &mut buf,
            storage.as_mut(),
            ZwlrLayerShellGetLayerSurfaceRequest {
                surface: surface.id(),
                output: Some(self.output_id),
                layer: ZwlrLayerShellLayer::Background,
                namespace: WLR_NAMESPACE,
            },
            move |proxy| LayerSurface {
                is_initial_configure_done: false,
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

        {
            let mut monitors = state.monitors.write().unwrap();

            monitors.insert(
                self.monitor_id,
                MonitorInfo {
                    output: WlObjectHandle::new(self.output_id),
                    surface,
                    layer_surface,
                    size,
                    name,
                },
            );
        }

        self.is_init_done = true;
    }
}

impl HasObjectType for Output {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Output;
}

impl Dispatch for Output {
    type State = ClientState;

    fn dispatch(
        &mut self,
        state: &Self::State,
        storage: &mut WlObjectStorage<Self::State>,
        message: WlMessage<'_>,
    ) {
        if let Some(event) = message.as_event::<WlOutputNameEvent>() {
            self.handle_name(event);
        } else if let Some(event) = message.as_event::<WlOutputModeEvent>() {
            self.handle_mode(event);
        }

        self.try_init_info(state, storage);
    }
}
pub fn handle_output(
    registry: WlObjectHandle<WlRegistry<ClientState>>,
    mut storage: Pin<&mut WlObjectStorage<ClientState>>,
    monitor_id: WlObjectId,
) {
    let mut buf = WlStackMessageBuffer::new();

    registry
        .bind_from_fn_by_id(
            &mut buf,
            storage.as_mut(),
            monitor_id,
            move |_, _, proxy| Output::new(monitor_id, proxy.id()),
        )
        .unwrap();
}

pub(crate) fn handle_global(
    registry: &mut WlRegistry<ClientState>,
    _: &ClientState,
    storage: &mut WlObjectStorage<ClientState>,
    global: WlRegistryGlobalEvent<'_>,
) {
    if global.interface != c"wl_output" {
        return;
    }

    let monitor_id = unsafe { WlObjectId::new_unchecked(global.name) };

    handle_output(registry.handle(), Pin::new(storage), monitor_id);
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

    {
        let mut names = state.monitor_names.write().unwrap();
        _ = names.remove(&info.name);
    }

    storage.release(info.output).unwrap();
    storage.release(info.surface).unwrap();
    storage.release(info.layer_surface).unwrap();

    {
        let mut events = state.stored_events.lock().unwrap();
        events.push(WaylandEvent::MonitorUnplugged {
            id: monitor_id,
            name: info.name,
        });
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

pub struct WaylandInner {
    pub client_state: Pin<Box<ClientState>>,
    pub main_queue: RwLock<Pin<Box<WlEventQueue<ClientState>>>>,
    pub display: WlDisplay<ClientState>,
    pub registry: WlObjectHandle<WlRegistry<ClientState>>,
}

impl WaylandInner {
    pub fn display_roundtrip(&self) {
        let mut main_queue = self.main_queue.write().unwrap();

        self.display
            .roundtrip(main_queue.as_mut(), self.client_state.as_ref());
    }

    pub fn dispatch_pending(&self) -> usize {
        let mut total_dispatched = 0;

        let mut main_queue = self.main_queue.write().unwrap();

        loop {
            let n_dispatched = self
                .display
                .dispatch_pending(main_queue.as_mut(), self.client_state.as_ref());

            if n_dispatched == 0 {
                break;
            }

            total_dispatched += n_dispatched;
        }

        total_dispatched
    }

    #[track_caller]
    pub fn prepare_poll(&self) -> usize {
        let mut main_queue = self.main_queue.write().unwrap();

        self.display
            .prepare_poll(main_queue.as_mut(), self.client_state.as_ref())
            .expect("failed to prepare poll")
    }

    pub fn unprepare_poll(&self) {
        self.display.cancel_read();
    }

    pub fn flush(&self) -> Result<usize, Errno> {
        self.display.flush()
    }

    pub fn new() -> Self {
        let mut client_state = Box::pin(ClientState::default());
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

        let _seat = registry.bind::<Seat>(&mut buf, storage.as_mut()).unwrap();

        client_state.globals = Some(Globals {
            compositor,
            layer_shell,
        });

        const N_INIT_ROUNDTRIPS: usize = 2;

        // NOTE(hack3rmann): try to do the intial setup before anything else
        for _ in 0..N_INIT_ROUNDTRIPS {
            display.roundtrip(queue.as_mut(), client_state.as_ref());
        }

        Self {
            client_state,
            display,
            main_queue: RwLock::new(queue),
            registry,
        }
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        self.display.display_handle().unwrap().as_raw()
    }

    pub fn drain_stored_events(&self, mut handle: impl FnMut(WaylandEvent)) {
        let mut events = self.client_state.stored_events.lock().unwrap();

        for event in events.drain(..) {
            handle(event);
        }
    }

    pub fn process_events(&self, mut handle: impl FnMut(WaylandEvent)) {
        self.drain_stored_events(&mut handle);
    }
}

impl Default for WaylandInner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct Wayland(Arc<WaylandInner>);

impl Deref for Wayland {
    type Target = WaylandInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct WaylandEventSource {
    wayland: Wayland,
    token: Option<Token>,
}

impl WaylandEventSource {
    pub const fn new(wayland: Wayland) -> Self {
        Self {
            wayland,
            token: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum WaylandProcessEventsError {
    #[error("WlDisplay::flush failed")]
    FlushFailed(#[from] Errno),
}

impl EventSource for WaylandEventSource {
    type Event = WaylandEvent;
    type Metadata = ();
    type Ret = ();
    type Error = WaylandProcessEventsError;

    const NEEDS_EXTRA_LIFECYCLE_EVENTS: bool = true;

    fn process_events<F>(
        &mut self,
        _: Readiness,
        _: Token,
        mut callback: F,
    ) -> Result<PostAction, Self::Error>
    where
        F: FnMut(Self::Event, &mut Self::Metadata) -> Self::Ret,
    {
        self.wayland.dispatch_pending();
        self.wayland
            .drain_stored_events(|event| callback(event, &mut ()));

        match self.wayland.flush() {
            Ok(_) | Err(Errno::AGAIN) => {}
            Err(error) => panic!("failed to flush display: {error}"),
        }

        Ok(PostAction::Continue)
    }

    fn register(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        let token = token_factory.token();
        self.token = Some(token);

        unsafe { poll.register(&self.wayland.display, Interest::READ, Mode::Level, token) }
    }

    fn reregister(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        let token = token_factory.token();
        self.token = Some(token);

        poll.reregister(&self.wayland.display, Interest::READ, Mode::Level, token)
    }

    fn unregister(&mut self, poll: &mut Poll) -> calloop::Result<()> {
        poll.unregister(&self.wayland.display)
    }

    fn before_sleep(&mut self) -> calloop::Result<Option<(Readiness, Token)>> {
        let n_dispatched = self.wayland.prepare_poll();

        if n_dispatched == 0 {
            return Ok(None);
        }

        let readiness = Readiness {
            readable: false,
            writable: false,
            error: false,
        };

        Ok(self.token.map(|t| (readiness, t)))
    }

    fn before_handle_events(&mut self, events: EventIterator<'_>) {
        let contains_us = events
            .into_iter()
            .any(|(readiness, token)| readiness.readable && self.token == Some(token));

        if !contains_us {
            self.wayland.unprepare_poll();
            return;
        }

        match self.wayland.display.read_events() {
            Ok(()) | Err(Errno::AGAIN) => {}
            Err(error) => panic!("failed to read events: {error}"),
        }
    }
}
