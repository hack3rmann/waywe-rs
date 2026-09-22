use crate::wayland::{
    ClientState, Globals, MonitorId, MonitorName, Region, Viewport, WLR_NAMESPACE, WaylandEvent,
};
use glam::UVec2;
use std::{fmt, mem, num::NonZeroU32, pin::Pin};
use wayland_client::{
    interface::{
        WlCompositorCreateRegionRequest, WlCompositorCreateSurfaceRequest, WlOutputEvent,
        WlOutputMode, WlOutputModeEvent, WlOutputNameEvent, WlRegionAddRequest,
        WlRegionDestroyRequest, WlSurfaceCommitRequest, WlSurfaceSetBufferScaleRequest,
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
        object::{WlObjectHandle, dispatch::Dispatch, registry::WlRegistry},
        object_storage::WlObjectStorage,
        wire::{WlMessage, WlStackMessageBuffer},
    },
};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scale(NonZeroU32);

impl Scale {
    pub const ONE: Self = Self(NonZeroU32::new(120).unwrap());

    pub const fn new(frac_120: u32) -> Self {
        Self(match NonZeroU32::new(frac_120) {
            Some(value) => value,
            None => NonZeroU32::new(120).unwrap(),
        })
    }

    pub const fn value(self) -> f32 {
        self.0.get() as f32 / 120.0
    }

    pub fn to_phisical(self, logical_size: UVec2) -> UVec2 {
        self.0.get() * logical_size / 120
    }

    pub fn to_logical(self, phisical_size: UVec2) -> UVec2 {
        120 * phisical_size / self.0.get()
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::ONE
    }
}

impl fmt::Debug for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/120", self.0)
    }
}

#[derive(Default, Debug, Clone)]
pub struct MonitorInfo {
    pub monitor_id: MonitorId,
    pub name: MonitorName,
    pub logical_size: UVec2,
    pub scale: Option<WaylandScale>,
    pub output: WlObjectHandle<Output>,
    pub surface: WlObjectHandle<Surface>,
    pub layer: WlObjectHandle<LayerSurface>,
}

impl MonitorInfo {
    pub fn phisical_size(&self) -> UVec2 {
        self.scale
            .map(|s| s.value)
            .unwrap_or(Scale::ONE)
            .to_phisical(self.logical_size)
    }
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

#[derive(Clone, Copy, Debug)]
pub struct WaylandScale {
    pub value: Scale,
    pub object: WlObjectHandle<FractionalScale>,
    pub viewport: WlObjectHandle<Viewport>,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputTransaction {
    is_create: bool,
    is_remove: bool,
    configure: Option<(u32, UVec2)>,
    scale: Option<Scale>,
}

impl OutputTransaction {
    pub const EMPTY: Self = Self {
        is_create: false,
        is_remove: false,
        configure: None,
        scale: None,
    };

    pub const CREATE: Self = Self {
        is_create: true,
        ..Self::EMPTY
    };

    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = Some(scale);
    }

    pub fn set_configure(&mut self, serial: u32, size: UVec2) {
        self.configure = Some((serial, size));
    }

    pub fn set_remove(&mut self) {
        self.is_remove = true;
    }

    pub fn commit(&mut self) -> Option<OutputAction> {
        let result = match *self {
            Self {
                is_create: true, ..
            } => OutputAction::Create,
            Self {
                is_remove: true, ..
            } => OutputAction::Remove,
            Self {
                is_create: false,
                is_remove: false,
                configure: Some((serial, size)),
                scale,
            } => OutputAction::Configure {
                serial,
                size,
                scale,
            },
            _ => return None,
        };

        *self = Self::EMPTY;
        Some(result)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputAction {
    Create,
    Remove,
    Configure {
        serial: u32,
        size: UVec2,
        scale: Option<Scale>,
    },
}

#[derive(Debug)]
pub enum Output {
    Active {
        info: MonitorInfo,
        transaction: OutputTransaction,
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
        viewport: Option<WlObjectHandle<Viewport>>,
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
    pub fn active(info: MonitorInfo) -> Self {
        Self::Active {
            info,
            transaction: OutputTransaction::CREATE,
        }
    }

    pub const fn new(monitor_id: MonitorId, output: WlObjectHandle<Self>) -> Self {
        Self::AwaitingOutputInfo {
            monitor_id,
            output,
            name: None,
            logical_size: None,
        }
    }

    pub fn set_remove(&mut self) -> &mut Self {
        let Self::Active { transaction, .. } = self else {
            return self;
        };

        transaction.set_remove();
        self
    }

    pub fn set_name(&mut self, name: MonitorName) -> &mut Self {
        match self {
            Self::Active { .. } => unimplemented!("monitor renaming"),
            Self::AwaitingConfigure { name: old_name, .. } => *old_name = name,
            Self::AwaitingScale { name: old_name, .. } => *old_name = name,
            Self::AwaitingOutputInfo { name: old_name, .. } => *old_name = Some(name),
        }

        self
    }

    pub fn set_size(&mut self, size: UVec2) -> &mut Self {
        match self {
            Self::Active { .. } => unimplemented!("monitor resize without configure"),
            Self::AwaitingConfigure { logical_size, .. }
            | Self::AwaitingScale { logical_size, .. } => *logical_size = size,
            Self::AwaitingOutputInfo { logical_size, .. } => *logical_size = Some(size),
        }

        self
    }

    pub fn set_surface_configure(&mut self, serial: u32, size: UVec2) -> &mut Self {
        match self {
            Output::Active { transaction, .. } => transaction.set_configure(serial, size),
            Output::AwaitingConfigure {
                serial: old_serial,
                logical_size: old_size,
                ..
            } => {
                *old_serial = Some(serial);
                *old_size = size;
            }
            _ => {}
        }

        self
    }

    pub fn set_scale(&mut self, scale: Scale) -> &mut Self {
        match self {
            Self::Active { transaction, .. } => transaction.set_scale(scale),
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

    fn handle_configure(
        mut storage: Pin<&mut WlObjectStorage<ClientState>>,
        globals: &Globals,
        layer: WlObjectHandle<LayerSurface>,
        surface: WlObjectHandle<Surface>,
        scale: Option<WaylandScale>,
        serial: u32,
        logical_size: UVec2,
    ) {
        let mut buf = WlStackMessageBuffer::new();

        layer.request(
            &mut buf,
            storage.as_mut().get_mut(),
            ZwlrLayerSurfaceAckConfigureRequest { serial },
        );

        let region: WlObjectHandle<Region> = globals.compositor.create_object(
            &mut buf,
            storage.as_mut(),
            WlCompositorCreateRegionRequest,
        );

        let width = logical_size.x.cast_signed();
        let height = logical_size.y.cast_signed();

        region.request(
            &mut buf,
            &storage.as_ref(),
            WlRegionAddRequest {
                x: 0,
                y: 0,
                width,
                height,
            },
        );

        surface.request(
            &mut buf,
            &storage.as_ref(),
            WlSurfaceSetOpaqueRegionRequest {
                region: Some(region.id()),
            },
        );

        if let Some(scale) = scale {
            scale.viewport.request(
                &mut buf,
                &storage.as_ref(),
                WpViewportSetDestinationRequest { width, height },
            );
        }

        region.request(&mut buf, &storage.as_ref(), WlRegionDestroyRequest);

        storage.as_mut().release(region).unwrap();

        surface.request(&mut buf, &storage.as_ref(), WlSurfaceCommitRequest);
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
                let mut storage = Pin::new(&mut *storage);

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

                let fractional_scale: Option<WlObjectHandle<FractionalScale>> =
                    globals.fractional_scale_manager.map(|m| {
                        m.create_object_with(
                            &mut buf,
                            storage.as_mut(),
                            WpFractionalScaleManagerGetFractionalScaleRequest {
                                surface: surface.id(),
                            },
                            move |_| FractionalScale { output },
                        )
                    });

                let viewport: Option<WlObjectHandle<Viewport>> = fractional_scale.map(|_| {
                    globals.viewporter.create_object(
                        &mut buf,
                        storage.as_mut(),
                        WpViewporterGetViewportRequest {
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

                if let Some(viewport) = viewport {
                    viewport.request(
                        &mut buf,
                        &storage,
                        WpViewportSetDestinationRequest {
                            width: logical_size.x.cast_signed(),
                            height: logical_size.y.cast_signed(),
                        },
                    );
                }

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

                Self::handle_configure(
                    Pin::new(storage),
                    &globals,
                    layer,
                    surface,
                    None,
                    serial,
                    logical_size,
                );

                *self = match (fractional_scale, viewport, scale) {
                    (None, _, _) | (_, None, _) => Self::active(MonitorInfo {
                        monitor_id,
                        name: mem::take(name),
                        logical_size,
                        scale: None,
                        output,
                        surface,
                        layer,
                    }),
                    (Some(object), Some(viewport), Some(value)) => Self::active(MonitorInfo {
                        monitor_id,
                        name: mem::take(name),
                        logical_size,
                        scale: Some(WaylandScale {
                            object,
                            value,
                            viewport,
                        }),
                        output,
                        surface,
                        layer,
                    }),
                    (Some(fractional_scale), Some(viewport), None) => Self::AwaitingScale {
                        monitor_id,
                        name: mem::take(name),
                        logical_size,
                        scale,
                        output,
                        surface,
                        layer,
                        fractional_scale,
                        viewport,
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
                *self = Self::active(MonitorInfo {
                    monitor_id,
                    name: mem::take(name),
                    logical_size,
                    scale: Some(WaylandScale {
                        value: scale,
                        object: fractional_scale,
                        viewport,
                    }),
                    output,
                    surface,
                    layer,
                });
            }
            _ => {}
        }

        let Self::Active { info, transaction } = self else {
            return;
        };

        let Some(transaction) = transaction.commit() else {
            return;
        };

        match transaction {
            OutputAction::Create => {
                {
                    let mut monitors = state.monitors.write().unwrap();
                    monitors.insert(info.monitor_id, info.clone());
                }

                {
                    let mut names = state.monitor_names.write().unwrap();
                    names.insert(info.name.clone(), info.monitor_id);
                }

                {
                    let mut events = state.stored_events.lock().unwrap();
                    events.push(WaylandEvent::MonitorPlugged {
                        id: info.monitor_id,
                        name: info.name.clone(),
                    });
                }
            }
            OutputAction::Remove => {
                {
                    let mut monitors = state.monitors.write().unwrap();
                    monitors.remove(&info.monitor_id);
                }

                {
                    let mut names = state.monitor_names.write().unwrap();
                    names.remove(&info.name);
                }

                storage.release(info.surface).unwrap();
                storage.release(info.layer).unwrap();

                if let Some(scale) = info.scale {
                    storage.release(scale.object).unwrap();
                    storage.release(scale.viewport).unwrap();
                }

                {
                    let mut events = state.stored_events.lock().unwrap();
                    events.push(WaylandEvent::MonitorUnplugged {
                        id: info.monitor_id,
                        name: info.name.clone(),
                    });
                }
            }
            OutputAction::Configure {
                serial,
                size,
                scale,
            } => {
                let old_phisical_size = info.phisical_size();

                info.logical_size = size;
                if let (Some(old), Some(new_scale)) = (&mut info.scale, scale) {
                    old.value = new_scale;
                }

                let Some(globals) = state.globals else {
                    return;
                };

                Self::handle_configure(
                    Pin::new(storage),
                    &globals,
                    info.layer,
                    info.surface,
                    info.scale,
                    serial,
                    size,
                );

                if old_phisical_size != info.phisical_size() {
                    let mut events = state.stored_events.lock().unwrap();
                    events.push(WaylandEvent::ResizeRequested {
                        monitor_id: info.monitor_id,
                        phisical_size: info.phisical_size(),
                    });
                }
            }
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
        let Some(event) = message.as_event::<WlOutputEvent>() else {
            return;
        };

        match event {
            WlOutputEvent::Mode(WlOutputModeEvent {
                width,
                height,
                flags,
                ..
            }) => {
                if !flags.contains(WlOutputMode::CURRENT) {
                    return;
                }

                self.set_size(UVec2::new(width.cast_unsigned(), height.cast_unsigned()));
            }
            WlOutputEvent::Name(WlOutputNameEvent { name }) => {
                let name_str = unsafe { str::from_utf8_unchecked(name.to_bytes()) };
                self.set_name(MonitorName::from_str(name_str));
            }
            _ => return,
        }

        self.update(state, storage);
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
            move |_, _, proxy| Output::new(monitor_id, WlObjectHandle::new(proxy.id())),
        )
        .unwrap();
}
