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
    fmt, mem,
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
        WlSurfaceSetOpaqueRegionRequest, WpFractionalScaleManagerGetFractionalScaleRequest,
        WpFractionalScalePreferredScaleEvent, WpViewportSetDestinationRequest,
        WpViewporterGetViewportRequest, ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellLayer,
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

#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scale(u32);

impl Scale {
    pub const ONE: Self = Self(120);

    pub const fn new(frac_120: u32) -> Self {
        Self(frac_120)
    }

    pub const fn value(self) -> f32 {
        self.0 as f32 / 120.0
    }

    pub fn to_phisical(self, logical_size: UVec2) -> UVec2 {
        self.0 * logical_size / 120
    }

    pub fn to_logical(self, phisical_size: UVec2) -> UVec2 {
        120 * phisical_size / self.0
    }
}

impl fmt::Debug for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/120", self.0)
    }
}

#[derive(Default, Debug, Clone)]
pub struct MonitorInfo {
    pub phisical_size: UVec2,
    pub logical_size: UVec2,
    pub scale: Option<Scale>,
    pub name: MonitorName,
    pub output: WlObjectHandle<OldOutput>,
    pub surface: WlObjectHandle<Surface>,
    pub layer_surface: WlObjectHandle<LayerSurface>,
    pub viewport: WlObjectHandle<Viewport>,
    pub fractional_scale: Option<WlObjectHandle<FractionalScale>>,
}

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
        monitors.get(&id).map(|info| info.phisical_size)
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
pub struct FractionalScaleManager;

impl HasObjectType for FractionalScaleManager {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WpFractionalScaleManagerV1;
}

impl Dispatch for FractionalScaleManager {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

#[derive(Default)]
pub struct FractionalScale {
    output: WlObjectHandle<Output>,
}

impl HasObjectType for FractionalScale {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WpFractionalScaleV1;
}

impl Dispatch for FractionalScale {
    type State = ClientState;

    fn dispatch(
        &mut self,
        state: &Self::State,
        storage: &mut WlObjectStorage<Self::State>,
        event: WlMessage<'_>,
    ) {
        let Some(WpFractionalScalePreferredScaleEvent { scale }) = event.as_event() else {
            return;
        };

        storage.with_object(self.output, move |storage, output| {
            output.set_scale(Scale::new(scale)).update(state, storage);
        });
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
    pub output: WlObjectHandle<Output>,
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

        storage.with_object(self.output, move |storage, output| {
            output
                .set_surface_configure(serial, UVec2::new(width, height))
                .update(state, storage);
        });
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

#[derive(Clone, Copy, Debug)]
struct WaylandScale {
    value: Scale,
    object: WlObjectHandle<FractionalScale>,
}

enum Output {
    Active {
        monitor_id: MonitorId,
        name: MonitorName,
        logical_size: UVec2,
        scale: Option<WaylandScale>,
        output: WlObjectHandle<Self>,
        surface: WlObjectHandle<Surface>,
        layer: WlObjectHandle<LayerSurface>,
        viewport: WlObjectHandle<Viewport>,
    },
    AwaitingOutputInfo {
        monitor_id: MonitorId,
        output: WlObjectHandle<Self>,
        name: Option<MonitorName>,
        logical_size: Option<UVec2>,
    },
    AwaitingConfigure {
        monitor_id: MonitorId,
        name: MonitorName,
        logical_size: UVec2,
        scale: Option<Scale>,
        output: WlObjectHandle<Self>,
        surface: WlObjectHandle<Surface>,
        layer: WlObjectHandle<LayerSurface>,
        viewport: WlObjectHandle<Viewport>,
        fractional_scale: Option<WlObjectHandle<FractionalScale>>,
        serial: Option<u32>,
    },
    AwaitingScale {
        monitor_id: MonitorId,
        name: MonitorName,
        logical_size: UVec2,
        scale: Option<Scale>,
        output: WlObjectHandle<Self>,
        surface: WlObjectHandle<Surface>,
        layer: WlObjectHandle<LayerSurface>,
        viewport: WlObjectHandle<Viewport>,
        fractional_scale: WlObjectHandle<FractionalScale>,
    },
}

impl Output {
    pub const fn new(monitor_id: MonitorId, output: WlObjectHandle<Self>) -> Self {
        Self::AwaitingOutputInfo {
            monitor_id,
            output,
            name: None,
            logical_size: None,
        }
    }

    pub fn set_name(&mut self, name: MonitorName) -> &mut Self {
        match self {
            Self::Active { name: old_name, .. }
            | Self::AwaitingConfigure { name: old_name, .. } => *old_name = name,
            Self::AwaitingScale { name: old_name, .. } => *old_name = name,
            Self::AwaitingOutputInfo { name: old_name, .. } => *old_name = Some(name),
        }

        self
    }

    pub fn set_size(&mut self, size: UVec2) -> &mut Self {
        match self {
            Self::Active { logical_size, .. }
            | Self::AwaitingConfigure { logical_size, .. }
            | Self::AwaitingScale { logical_size, .. } => *logical_size = size,
            Self::AwaitingOutputInfo { logical_size, .. } => *logical_size = Some(size),
        }

        self
    }

    pub fn set_surface_configure(&mut self, serial: u32, size: UVec2) -> &mut Self {
        let Self::AwaitingConfigure {
            serial: this_serial,
            logical_size: this_size,
            ..
        } = self
        else {
            return self;
        };
        *this_serial = Some(serial);
        *this_size = size;

        self
    }

    pub fn set_scale(&mut self, scale: Scale) -> &mut Self {
        match self {
            Self::Active {
                scale: Some(wl_scale),
                ..
            } => wl_scale.value = scale,
            Self::AwaitingScale {
                scale: old_scale, ..
            }
            | Self::AwaitingConfigure {
                scale: old_scale, ..
            } => *old_scale = Some(scale),
            _ => {}
        }

        self
    }

    pub fn update(&mut self, state: &ClientState, storage: &mut WlObjectStorage<ClientState>) {
        match *self {
            Self::AwaitingOutputInfo {
                monitor_id,
                output,
                name: Some(ref mut name),
                logical_size: Some(logical_size),
            } => {
                let Some(globals) = state.globals else { return };

                let mut buf = WlStackMessageBuffer::new();
                let mut storage = Pin::new(storage);

                let surface: WlObjectHandle<Surface> = globals.compositor.create_object(
                    &mut buf,
                    storage.as_mut(),
                    WlCompositorCreateSurfaceRequest,
                );

                let layer_surface: WlObjectHandle<LayerSurface> =
                    globals.layer_shell.create_object_with(
                        &mut buf,
                        storage.as_mut(),
                        ZwlrLayerShellGetLayerSurfaceRequest {
                            surface: surface.id(),
                            output: Some(output.id()),
                            layer: ZwlrLayerShellLayer::Background,
                            namespace: WLR_NAMESPACE,
                        },
                        move |_| LayerSurface { output },
                    );

                let viewport: WlObjectHandle<Viewport> = globals.viewporter.create_object(
                    &mut buf,
                    storage.as_mut(),
                    WpViewporterGetViewportRequest {
                        surface: surface.id(),
                    },
                );

                let fractional_scale: Option<WlObjectHandle<FractionalScale>> =
                    globals.fractional_scale_manager.map(|m| {
                        m.create_object(
                            &mut buf,
                            storage.as_mut(),
                            WpFractionalScaleManagerGetFractionalScaleRequest {
                                surface: surface.id(),
                            },
                        )
                    });

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

                viewport.request(
                    &mut buf,
                    &storage,
                    WpViewportSetDestinationRequest {
                        width: logical_size.x.cast_signed(),
                        height: logical_size.y.cast_signed(),
                    },
                );

                surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

                *self = Self::AwaitingConfigure {
                    serial: None,
                    monitor_id,
                    name: mem::take(name),
                    logical_size,
                    scale: None,
                    output,
                    surface,
                    viewport,
                    layer: layer_surface,
                    fractional_scale,
                };
            }
            Self::AwaitingConfigure {
                monitor_id,
                ref mut name,
                logical_size,
                scale,
                output,
                surface,
                layer,
                viewport,
                fractional_scale,
                serial: Some(serial),
            } => {
                let Some(globals) = state.globals else { return };

                let mut buf = WlStackMessageBuffer::new();

                layer.request(
                    &mut buf,
                    storage,
                    ZwlrLayerSurfaceAckConfigureRequest { serial },
                );

                let mut storage = Pin::new(storage);

                let region: WlObjectHandle<Region> = globals.compositor.create_object(
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
                        width: logical_size.x.cast_signed(),
                        height: logical_size.y.cast_signed(),
                    },
                );

                surface.request(
                    &mut buf,
                    &storage.as_ref(),
                    WlSurfaceSetOpaqueRegionRequest {
                        region: Some(region.id()),
                    },
                );

                region.request(&mut buf, &storage.as_ref(), WlRegionDestroyRequest);

                storage.as_mut().release(region).unwrap();

                surface.request(&mut buf, &storage.as_ref(), WlSurfaceCommitRequest);

                *self = match (fractional_scale, scale) {
                    (None, _) => Self::Active {
                        monitor_id,
                        name: mem::take(name),
                        logical_size,
                        scale: None,
                        output,
                        surface,
                        layer,
                        viewport,
                    },
                    (Some(object), Some(value)) => Self::Active {
                        monitor_id,
                        name: mem::take(name),
                        logical_size,
                        scale: Some(WaylandScale { object, value }),
                        output,
                        surface,
                        layer,
                        viewport,
                    },
                    (Some(fractional_scale), None) => Self::AwaitingScale {
                        monitor_id,
                        name: mem::take(name),
                        logical_size,
                        scale,
                        output,
                        surface,
                        layer,
                        viewport,
                        fractional_scale,
                    },
                };
            }
            Self::AwaitingScale {
                monitor_id,
                ref mut name,
                logical_size,
                scale: Some(scale),
                output,
                surface,
                layer,
                viewport,
                fractional_scale,
            } => {
                *self = Self::Active {
                    monitor_id,
                    name: mem::take(name),
                    logical_size,
                    scale: Some(WaylandScale {
                        value: scale,
                        object: fractional_scale,
                    }),
                    output,
                    surface,
                    layer,
                    viewport,
                };
            }
            _ => {}
        }
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
            let name = unsafe { str::from_utf8_unchecked(event.name.to_bytes()) };
            self.set_name(MonitorName::from_str(name));
        } else if let Some(event) = message.as_event::<WlOutputModeEvent>() {
            if !event.flags.contains(WlOutputMode::CURRENT) {
                return;
            }

            let Ok(width) = u32::try_from(event.width) else {
                return;
            };
            let Ok(height) = u32::try_from(event.height) else {
                return;
            };

            self.set_size(UVec2::new(width, height));
        }

        self.update(state, storage);
    }
}

pub struct OldOutput {
    pub monitor_id: MonitorId,
    pub output_id: WlObjectId,
    pub size: Option<UVec2>,
    pub name: Option<MonitorName>,
    pub monitor_info: Option<MonitorInfo>,
    pub scale: Option<Scale>,
    pub is_init_done: bool,
}

impl OldOutput {
    pub const fn new(monitor_id: MonitorId, output_id: WlObjectId) -> Self {
        Self {
            monitor_id,
            output_id,
            size: None,
            name: None,
            monitor_info: None,
            scale: None,
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

    pub fn try_init_with_scale(
        &mut self,
        state: &ClientState,
        _: &mut WlObjectStorage<ClientState>,
        _: Scale,
    ) {
        if self.is_init_done {
            return;
        }

        let Some(mut info) = self.monitor_info.as_ref().cloned() else {
            return;
        };
        let Some(scale) = self.scale else {
            return;
        };

        info.scale = Some(scale);
        info.phisical_size = scale.to_phisical(info.logical_size);

        state.commit_monitor(self.monitor_id, info);

        self.is_init_done = true;
    }

    pub fn try_create_surface(
        &mut self,
        state: &ClientState,
        storage: &mut WlObjectStorage<ClientState>,
    ) {
        if self.monitor_info.is_some() {
            return;
        }

        let Some(globals) = state.globals else { return };
        let Some(logical_size) = self.size else {
            return;
        };
        let Some(name) = self.name.clone() else {
            return;
        };

        let phisical_size = logical_size;

        let mut buf = WlStackMessageBuffer::new();
        let mut storage = Pin::new(storage);

        let surface: WlObjectHandle<Surface> = globals.compositor.create_object(
            &mut buf,
            storage.as_mut(),
            WlCompositorCreateSurfaceRequest,
        );

        let output_id = self.output_id;
        let layer_surface: WlObjectHandle<LayerSurface> = globals.layer_shell.create_object_with(
            &mut buf,
            storage.as_mut(),
            ZwlrLayerShellGetLayerSurfaceRequest {
                surface: surface.id(),
                output: Some(self.output_id),
                layer: ZwlrLayerShellLayer::Background,
                namespace: WLR_NAMESPACE,
            },
            move |_| LayerSurface {
                output: WlObjectHandle::new(output_id),
            },
        );

        let viewport: WlObjectHandle<Viewport> = globals.viewporter.create_object(
            &mut buf,
            storage.as_mut(),
            WpViewporterGetViewportRequest {
                surface: surface.id(),
            },
        );

        let fractional_scale: Option<WlObjectHandle<FractionalScale>> =
            globals.fractional_scale_manager.map(|m| {
                m.create_object(
                    &mut buf,
                    storage.as_mut(),
                    WpFractionalScaleManagerGetFractionalScaleRequest {
                        surface: surface.id(),
                    },
                )
            });

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

        viewport.request(
            &mut buf,
            &storage,
            WpViewportSetDestinationRequest {
                width: logical_size.x.cast_signed(),
                height: logical_size.y.cast_signed(),
            },
        );

        surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

        let info = MonitorInfo {
            output: WlObjectHandle::new(self.output_id),
            scale: None,
            surface,
            layer_surface,
            logical_size,
            phisical_size,
            name,
            viewport,
            fractional_scale,
        };

        // if fractional_scale.is_none() {
        state.commit_monitor(self.monitor_id, info.clone());
        self.is_init_done = true;
        // }

        self.monitor_info = Some(info);
    }
}

impl HasObjectType for OldOutput {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Output;
}

impl Dispatch for OldOutput {
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

        self.try_create_surface(state, storage);
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
            move |_, _, proxy| OldOutput::new(monitor_id, proxy.id()),
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
    storage.release(info.viewport).unwrap();

    if let Some(frac_scale) = info.fractional_scale {
        storage.release(frac_scale).unwrap();
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
