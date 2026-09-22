pub mod output;

use crate::wayland::output::{
    FractionalScaleManager, LayerSurface, MonitorInfo, Surface, handle_output,
};
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
        WlPointerEvent, WlRegistryEvent, WlRegistryGlobalEvent, WlRegistryGlobalRemoveEvent,
        WlSeatCapabilitiesEvent, WlSeatCapability, WlSeatGetPointerRequest,
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
    ResizeRequested {
        monitor_id: MonitorId,
        phisical_size: UVec2,
    },
    MonitorPlugged {
        id: MonitorId,
        name: MonitorName,
    },
    MonitorUnplugged {
        id: MonitorId,
        name: MonitorName,
    },
    // TODO(hack3rmann): implement approach from <https://github.com/cjacker/wl-find-cursor/blob/main/main.c>
    CursorMoved {
        position: UVec2,
    },
}

pub type MonitorId = WlObjectId;
pub type MonitorMap<T> = BTreeMap<MonitorId, T>;
pub type MonitorName = SmallString<[u8; 24]>;

#[derive(Default, Debug, Clone, Copy)]
pub struct Globals {
    pub compositor: WlObjectHandle<Compositor>,
    pub layer_shell: WlObjectHandle<LayerShell>,
    pub viewporter: WlObjectHandle<Viewporter>,
    pub fractional_scale_manager: Option<WlObjectHandle<FractionalScaleManager>>,
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
        monitors.get(&id).map(MonitorInfo::phisical_size)
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

    pub fn commit_monitor(&self, id: MonitorId, info: MonitorInfo) {
        {
            let mut names = self.monitor_names.write().unwrap();
            names.insert(info.name.clone(), id);
        }

        {
            let mut monitors = self.monitors.write().unwrap();
            monitors.insert(id, info.clone());
        }
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
pub struct Viewporter;

impl HasObjectType for Viewporter {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WpViewporter;
}

impl Dispatch for Viewporter {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

#[derive(Default)]
pub struct Viewport;

impl HasObjectType for Viewport {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WpViewport;
}

impl Dispatch for Viewport {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
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
        _: &mut WlObjectStorage<Self::State>,
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
    storage.release(info.layer).unwrap();

    if let Some(scale) = info.scale {
        storage.release(scale.object).unwrap();
        storage.release(scale.viewport).unwrap();
    }

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

        let viewporter = registry
            .bind::<Viewporter>(&mut buf, storage.as_mut())
            .unwrap();

        let fractional_scale_manager =
            registry.bind::<FractionalScaleManager>(&mut buf, storage.as_mut());

        client_state.globals = Some(Globals {
            compositor,
            layer_shell,
            viewporter,
            fractional_scale_manager,
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
