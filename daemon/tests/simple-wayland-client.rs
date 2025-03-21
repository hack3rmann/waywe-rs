use std::pin::{Pin, pin};
use wayland::{
    Dispatch, HasObjectType, SmallVecMessageBuffer, StackMessageBuffer, WlDisplay, WlObjectHandle,
    WlObjectType, WlProxy, WlRegistry,
    interface::{
        WlCompositorCreateSurfaceRequest, WlToplevelConfigureEvent, WlWmBaseGetXdgSurfaceRequest,
        WlWmBasePingEvent, WlWmBasePongRequest, WlXdgSurfaceAckConfigureRequest,
        WlXdgSurfaceConfigureEvent, WlXdgSurfaceGetToplevelRequest,
    },
    sys::{
        object::{FromProxy, dispatch::State},
        wire::WlMessage,
    },
};

#[derive(Clone, Debug, Default)]
pub struct ClientState {
    pub should_resize: bool,
    pub ready_to_resize: bool,
    pub should_close: bool,
    pub next_width: u32,
    pub next_height: u32,
}

impl State for ClientState {}

pub struct WlCompositor;

impl Dispatch for WlCompositor {
    type State = ClientState;
}

impl FromProxy for WlCompositor {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl HasObjectType for WlCompositor {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
}

pub struct WlWmBase {
    pub handle: WlObjectHandle<Self>,
}

impl Dispatch for WlWmBase {
    type State = ClientState;

    fn dispatch(
        &mut self,
        _state: Pin<&mut Self::State>,
        storage: Pin<&mut wayland::WlObjectStorage<'_, Self::State>>,
        message: WlMessage<'_>,
    ) {
        let Some(WlWmBasePingEvent { serial }) = message.as_event() else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<1>::new();

        self.handle
            .request(&mut buf, &storage, WlWmBasePongRequest { serial });
    }
}

impl FromProxy for WlWmBase {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

impl HasObjectType for WlWmBase {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WmBase;
}

pub struct WlSurface;

impl Dispatch for WlSurface {
    type State = ClientState;
}

impl HasObjectType for WlSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
}

impl FromProxy for WlSurface {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

pub struct WlXdgSurface {
    pub handle: WlObjectHandle<Self>,
}

impl Dispatch for WlXdgSurface {
    type State = ClientState;

    fn dispatch(
        &mut self,
        mut state: Pin<&mut Self::State>,
        storage: Pin<&mut wayland::WlObjectStorage<'_, Self::State>>,
        message: WlMessage<'_>,
    ) {
        let Some(WlXdgSurfaceConfigureEvent { serial }) = message.as_event() else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<1>::new();

        self.handle.request(
            &mut buf,
            &storage,
            WlXdgSurfaceAckConfigureRequest { serial },
        );

        if state.should_resize {
            state.ready_to_resize = true;
        }
    }
}

impl HasObjectType for WlXdgSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::XdgSurface;
}

impl FromProxy for WlXdgSurface {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

pub struct WlToplevel;

impl Dispatch for WlToplevel {
    type State = ClientState;

    fn dispatch(
        &mut self,
        mut state: Pin<&mut Self::State>,
        _storage: Pin<&mut wayland::WlObjectStorage<'_, Self::State>>,
        message: WlMessage<'_>,
    ) {
        let Some(WlToplevelConfigureEvent { width, height, .. }) = message.as_event() else {
            return;
        };

        if width != 0 && height != 0 {
            state.should_resize = true;
            state.next_width = width;
            state.next_height = height;
        }
    }
}

impl HasObjectType for WlToplevel {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Toplevel;
}

impl FromProxy for WlToplevel {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

#[test]
fn simple_wayland_client() {
    let mut client_state = pin!(ClientState::default());

    let display = WlDisplay::connect(client_state.as_mut()).unwrap();

    let mut buf = StackMessageBuffer::new();
    let mut storage = pin!(display.create_storage());

    let registry = display.create_registry(&mut buf, storage.as_mut());

    display.dispatch_all_pending(storage.as_mut(), client_state.as_mut());

    let compositor =
        WlRegistry::bind::<WlCompositor>(&mut buf, storage.as_mut(), registry).unwrap();

    let wm_base = WlRegistry::bind::<WlWmBase>(&mut buf, storage.as_mut(), registry).unwrap();

    display.dispatch_all_pending(storage.as_mut(), client_state.as_mut());

    let surface: WlObjectHandle<WlSurface> =
        compositor.create_object(&mut buf, storage.as_mut(), WlCompositorCreateSurfaceRequest);

    let xdg_surface: WlObjectHandle<WlXdgSurface> = wm_base.create_object(
        &mut buf,
        storage.as_mut(),
        WlWmBaseGetXdgSurfaceRequest {
            surface: surface.id(),
        },
    );

    let _toplevel: WlObjectHandle<WlToplevel> =
        xdg_surface.create_object(&mut buf, storage.as_mut(), WlXdgSurfaceGetToplevelRequest);
}
