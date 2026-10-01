mod output;

use crate::output::{FractionalScaleManager, MonitorInfo, Surface, handle_output};
use calloop::{EventIterator, Interest, Mode, Poll, PostAction, Readiness, Token, TokenFactory};
use glam::UVec2;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandWindowHandle, WindowHandle,
};
use rustix::io::Errno;
use std::{
    collections::HashMap,
    error::Error,
    ffi::CStr,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
};
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
use waywe_runtime::platform::{MonitorMap, PlatformEvent, PlatformEventSource, WaywePlatform};

pub(crate) type MonitorId = WlObjectId;

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Globals {
    pub compositor: WlObjectHandle<Compositor>,
    pub layer_shell: WlObjectHandle<LayerShell>,
    pub viewporter: WlObjectHandle<Viewporter>,
    pub fractional_scale_manager: Option<WlObjectHandle<FractionalScaleManager>>,
}

#[derive(Default)]
pub(crate) struct ClientState {
    pub stored_events: Mutex<Vec<PlatformEvent>>,
    pub monitors: RwLock<HashMap<MonitorId, MonitorInfo>>,
    pub monitor_names: RwLock<MonitorMap<MonitorId>>,
    pub globals: Option<Globals>,
}

#[derive(Default)]
pub(crate) struct Compositor;

impl HasObjectType for Compositor {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
}

impl Dispatch for Compositor {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

pub(crate) struct Seat {
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
pub(crate) struct Viewporter;

impl HasObjectType for Viewporter {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WpViewporter;
}

impl Dispatch for Viewporter {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

#[derive(Default)]
pub(crate) struct Viewport;

impl HasObjectType for Viewport {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WpViewport;
}

impl Dispatch for Viewport {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

#[derive(Default)]
pub(crate) struct Pointer;

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
        events.push(PlatformEvent::CursorMoved { position });
    }
}

#[derive(Default)]
pub(crate) struct LayerShell;

impl HasObjectType for LayerShell {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerShell;
}

impl Dispatch for LayerShell {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

pub(crate) struct Region;

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
    let Some(output) = ({
        let monitors = state.monitors.read().unwrap();
        monitors.get(&monitor_id).map(|i| i.output)
    }) else {
        return;
    };

    storage.with_object(output, |storage, output| {
        output.set_remove().update(state, storage);
    });
    storage.release(output).unwrap();
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

pub(crate) const WLR_NAMESPACE: &CStr = c"waywe-runtime";

pub(crate) trait SurfaceExtension {
    fn raw_window_handle(&self) -> RawWindowHandle;
}

impl SurfaceExtension for WlObject<Surface> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Wayland(WaylandWindowHandle::new(self.proxy().as_raw()))
    }
}

pub(crate) struct WaylandInner {
    pub client_state: Pin<Box<ClientState>>,
    pub main_queue: RwLock<Pin<Box<WlEventQueue<ClientState>>>>,
    pub display: WlDisplay<ClientState>,
}

impl WaylandInner {
    pub(crate) fn dispatch_pending(&self) -> usize {
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
    pub(crate) fn prepare_poll(&self) -> usize {
        let mut main_queue = self.main_queue.write().unwrap();

        self.display
            .prepare_poll(main_queue.as_mut(), self.client_state.as_ref())
            .expect("failed to prepare poll")
    }

    pub(crate) fn unprepare_poll(&self) {
        self.display.cancel_read();
    }

    pub(crate) fn flush(&self) -> Result<usize, Errno> {
        self.display.flush()
    }

    pub(crate) fn new() -> Self {
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
        }
    }

    pub(crate) fn raw_display_handle(&self) -> RawDisplayHandle {
        self.display.display_handle().unwrap().as_raw()
    }
}

impl Default for WaylandInner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct Wayland(Arc<WaylandInner>);

struct WaylandSurface {
    wayland: Wayland,
    surface: WlObjectHandle<Surface>,
}

impl HasDisplayHandle for WaylandSurface {
    fn display_handle(&self) -> Result<DisplayHandle<'static>, HandleError> {
        Ok(unsafe { DisplayHandle::borrow_raw(self.wayland.0.raw_display_handle()) })
    }
}

impl HasWindowHandle for WaylandSurface {
    fn window_handle(&self) -> Result<WindowHandle<'static>, HandleError> {
        let handle = {
            let queue = self.wayland.0.main_queue.read().unwrap();
            queue
                .as_ref()
                .storage()
                .object(self.surface)
                .raw_window_handle()
        };

        Ok(unsafe { WindowHandle::borrow_raw(handle) })
    }
}

impl WaywePlatform for Wayland {
    fn get_surface(&self, monitor_name: &str) -> wgpu::SurfaceTarget<'static> {
        let monitor_id = {
            let names = self.0.client_state.monitor_names.read().unwrap();
            names[monitor_name]
        };
        let surface = {
            let monitors = self.0.client_state.monitors.read().unwrap();
            monitors[&monitor_id].surface
        };

        WaylandSurface {
            wayland: self.clone(),
            surface,
        }
        .into()
    }

    fn event_source(&self) -> Box<dyn PlatformEventSource> {
        Box::new(WaylandEventSource::new(self.clone()))
    }

    fn drain_stored_events(&self, handle: &mut dyn FnMut(PlatformEvent)) {
        let mut events = self.0.client_state.stored_events.lock().unwrap();

        for event in events.drain(..) {
            handle(event);
        }
    }
}

pub(crate) struct WaylandEventSource {
    wayland: Wayland,
    token: Option<Token>,
}

impl WaylandEventSource {
    pub(crate) const fn new(wayland: Wayland) -> Self {
        Self {
            wayland,
            token: None,
        }
    }
}

impl PlatformEventSource for WaylandEventSource {
    fn process_events(
        &mut self,
        _: Readiness,
        _: Token,
        callback: &mut dyn FnMut(PlatformEvent),
    ) -> Result<PostAction, Box<dyn Error + Send + Sync>> {
        self.wayland.0.dispatch_pending();
        self.wayland.drain_stored_events(callback);

        match self.wayland.0.flush() {
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

        unsafe { poll.register(&self.wayland.0.display, Interest::READ, Mode::Level, token) }
    }

    fn reregister(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        let token = token_factory.token();
        self.token = Some(token);

        poll.reregister(&self.wayland.0.display, Interest::READ, Mode::Level, token)
    }

    fn unregister(&mut self, poll: &mut Poll) -> calloop::Result<()> {
        poll.unregister(&self.wayland.0.display)
    }

    fn before_sleep(&mut self) -> calloop::Result<Option<(Readiness, Token)>> {
        let n_dispatched = self.wayland.0.prepare_poll();

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
            self.wayland.0.unprepare_poll();
            return;
        }

        match self.wayland.0.display.read_events() {
            Ok(()) | Err(Errno::AGAIN) => {}
            Err(error) => panic!("failed to read events: {error}"),
        }
    }
}
