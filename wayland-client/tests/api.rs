use rustix::{
    fs::Mode,
    mm::{MapFlags, ProtFlags},
    shm::OFlags,
};
use std::{
    mem,
    os::fd::{AsFd as _, OwnedFd},
    pin::pin,
    ptr, slice, thread,
    time::Duration,
};
use wayland_client::{
    Dispatch, FromProxy, HasObjectType, NoState, WlDisplay, WlMessage, WlObjectHandle,
    WlObjectStorage, WlObjectType, WlProxy, WlStackMessageBuffer,
    interface::{
        WlCompositorCreateRegionRequest, WlCompositorCreateSurfaceRequest, WlRegionDestroyRequest,
        WlShmCreatePoolRequest, WlShmFormat, WlShmPoolCreateBufferRequest, WlSurfaceAttachRequest,
        WlSurfaceCommitRequest, WlSurfaceDamageRequest, WlSurfaceSetBufferScaleRequest,
        WlSurfaceSetInputRegionRequest, WpViewporterGetViewportRequest,
        ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellLayer,
        ZwlrLayerSurfaceAckConfigureRequest, ZwlrLayerSurfaceAnchor,
        ZwlrLayerSurfaceConfigureEvent, ZwlrLayerSurfaceKeyboardInteractivity,
        ZwlrLayerSurfaceSetAnchorRequest, ZwlrLayerSurfaceSetExclusiveZoneRequest,
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest, ZwlrLayerSurfaceSetMarginRequest,
        ZwlrLayerSurfaceSetSizeRequest,
    },
};

macro_rules! define_empty_dispatchers {
    ( $( $Name:ident ),* $(,)? ) => {
        $(
            #[derive(Debug, Default)]
            pub struct $Name;

            impl wayland_client::HasObjectType for $Name {
                const OBJECT_TYPE: wayland_client::WlObjectType = wayland_client::WlObjectType:: $Name;
            }

            impl wayland_client::Dispatch for $Name {
                type State = wayland_client::NoState;

                const ALLOW_EMPTY_DISPATCH: bool = true;

                fn dispatch(
                    &mut self,
                    _state: &Self::State,
                    _storage: &mut wayland_client::WlObjectStorage<Self::State>,
                    _message: wayland_client::WlMessage<'_>,
                ) {
                    unreachable!()
                }
            }

            wayland_client::assert_dispatch_is_empty!( $Name );
        )*
    };
}

define_empty_dispatchers! {
    Buffer,
    Callback,
    Compositor,
    Output,
    Region,
    Shm,
    ShmPool,
    Surface,
    WpViewport,
    WpViewporter,
    LayerShell,
}

#[derive(Debug)]
pub struct WlLayerSurface {
    pub handle: WlObjectHandle<Self>,
}

impl HasObjectType for WlLayerSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
}

impl FromProxy for WlLayerSurface {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

impl Dispatch for WlLayerSurface {
    type State = NoState;

    fn dispatch(
        &mut self,
        _: &Self::State,
        storage: &mut WlObjectStorage<Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(ZwlrLayerSurfaceConfigureEvent { serial, .. }) = message.as_event() else {
            return;
        };

        let mut buf = WlStackMessageBuffer::new();

        self.handle.request(
            &mut buf,
            storage,
            ZwlrLayerSurfaceAckConfigureRequest { serial },
        );
    }
}

#[test]
fn just_connect_display() {
    _ = tracing_subscriber::fmt::try_init();

    let state = pin!(NoState);
    WlDisplay::connect(state.as_ref()).unwrap();
}

#[test]
#[should_panic]
fn get_protocol_error() {
    _ = tracing_subscriber::fmt::try_init();

    let mut buf = WlStackMessageBuffer::new();

    let state = pin!(NoState);
    let display = WlDisplay::connect(state.as_ref()).unwrap();
    let mut queue = pin!(display.take_main_queue().unwrap());
    let registry = display
        .create_registry(&mut buf, queue.as_mut().storage_mut())
        .handle();

    #[derive(Default)]
    pub struct WrongGlobal;

    impl HasObjectType for WrongGlobal {
        const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
    }

    impl Dispatch for WrongGlobal {
        type State = NoState;
        const ALLOW_EMPTY_DISPATCH: bool = true;
    }

    let _wrong_global: WlObjectHandle<WrongGlobal> = registry
        .bind(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    display.roundtrip(queue.as_mut(), state.as_ref());

    assert!(
        queue
            .as_ref()
            .storage()
            .object(registry)
            .interfaces()
            .contains_key(&WlObjectType::Compositor)
    );
}

#[test]
fn get_registry() {
    _ = tracing_subscriber::fmt::try_init();

    let mut buf = WlStackMessageBuffer::new();

    let state = pin!(NoState);
    let display = WlDisplay::connect(state.as_ref()).unwrap();
    let mut queue = pin!(display.take_main_queue().unwrap());
    let registry = display
        .create_registry(&mut buf, queue.as_mut().storage_mut())
        .handle();

    display.roundtrip(queue.as_mut(), state.as_ref());

    assert!(
        queue
            .as_ref()
            .storage()
            .object(registry)
            .interfaces()
            .contains_key(&WlObjectType::Compositor)
    );
}

#[test]
fn create_surface() {
    _ = tracing_subscriber::fmt::try_init();

    let mut buf = WlStackMessageBuffer::new();

    let state = pin!(NoState);
    let display = WlDisplay::connect(state.as_ref()).unwrap();
    let mut queue = pin!(display.take_main_queue().unwrap());
    let registry = display
        .create_registry(&mut buf, queue.as_mut().storage_mut())
        .handle();

    display.roundtrip(queue.as_mut(), state.as_ref());

    let compositor = registry
        .bind::<Compositor>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let surface: WlObjectHandle<Surface> = compositor.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlCompositorCreateSurfaceRequest,
    );

    assert_eq!(
        queue
            .as_ref()
            .storage()
            .object(surface)
            .proxy()
            .interface_name(),
        "wl_surface",
    );
}

#[test]
fn bind_wlr_shell() {
    _ = tracing_subscriber::fmt::try_init();

    let mut buf = WlStackMessageBuffer::new();

    let state = pin!(NoState);
    let display = WlDisplay::connect(state.as_ref()).unwrap();
    let mut queue = pin!(display.take_main_queue().unwrap());
    let registry = display
        .create_registry(&mut buf, queue.as_mut().storage_mut())
        .handle();

    display.roundtrip(queue.as_mut(), state.as_ref());

    let _layer_shell = registry
        .bind::<LayerShell>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    display.roundtrip(queue.as_mut(), state.as_ref());
}

fn open_shm() -> Result<(OwnedFd, String), rustix::io::Errno> {
    for i in 0.. {
        let wl_shm_path = format!("/wl_shm#{i}");

        match rustix::shm::open(
            &wl_shm_path,
            OFlags::EXCL | OFlags::RDWR | OFlags::CREATE | OFlags::TRUNC,
            Mode::RUSR | Mode::WUSR,
        ) {
            Ok(fd) => return Ok((fd, wl_shm_path)),
            Err(rustix::io::Errno::EXIST) => continue,
            Err(error) => return Err(error),
        };
    }

    unreachable!();
}

#[test]
fn white_rect() {
    _ = tracing_subscriber::fmt::try_init();

    let mut buf = WlStackMessageBuffer::new();

    let state = pin!(NoState);
    let display = WlDisplay::connect(state.as_ref()).unwrap();
    let mut queue = pin!(display.take_main_queue().unwrap());
    let registry = display
        .create_registry(&mut buf, queue.as_mut().storage_mut())
        .handle();

    display.roundtrip(queue.as_mut(), state.as_ref());

    let shm = registry
        .bind::<Shm>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let viewporter = registry
        .bind::<WpViewporter>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let compositor = registry
        .bind::<Compositor>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let surface: WlObjectHandle<Surface> = compositor.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlCompositorCreateSurfaceRequest,
    );

    let _viewport: WlObjectHandle<WpViewport> = viewporter.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WpViewporterGetViewportRequest {
            surface: surface.id(),
        },
    );

    let region: WlObjectHandle<Region> = compositor.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlCompositorCreateRegionRequest,
    );

    surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        WlSurfaceSetInputRegionRequest {
            region: Some(region.id()),
        },
    );

    region.request(&mut buf, &queue.as_ref().storage(), WlRegionDestroyRequest);
    queue.as_mut().storage_mut().release(region).unwrap();

    let output = registry
        .bind::<Output>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let layer_shell = registry
        .bind::<LayerShell>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let layer_surface: WlObjectHandle<WlLayerSurface> = layer_shell.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        ZwlrLayerShellGetLayerSurfaceRequest {
            surface: surface.id(),
            output: Some(output.id()),
            layer: ZwlrLayerShellLayer::Background,
            namespace: c"wallpaper-engine",
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetAnchorRequest {
            anchor: ZwlrLayerSurfaceAnchor::all(),
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetExclusiveZoneRequest { zone: -1 },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetMarginRequest {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest {
            keyboard_interactivity: ZwlrLayerSurfaceKeyboardInteractivity::None,
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetSizeRequest {
            width: BUFFER_WIDTH_PIXELS as u32,
            height: BUFFER_HEIGHT_PIXELS as u32,
        },
    );

    surface.request(&mut buf, &queue.as_ref().storage(), WlSurfaceCommitRequest);

    display.roundtrip(queue.as_mut(), state.as_ref());

    let (shm_fd, shm_path) = open_shm().unwrap();

    const BUFFER_WIDTH_PIXELS: usize = 2520;
    const BUFFER_HEIGHT_PIXELS: usize = 1680;
    const PIXEL_SIZE_BYTES: usize = mem::size_of::<u32>();
    const BUFFER_SIZE_PIXELS: usize = BUFFER_WIDTH_PIXELS * BUFFER_HEIGHT_PIXELS;
    const BUFFER_SIZE_BYTES: usize = BUFFER_SIZE_PIXELS * PIXEL_SIZE_BYTES;

    rustix::fs::ftruncate(&shm_fd, BUFFER_SIZE_BYTES as u64).unwrap();

    let shm_ptr = unsafe {
        rustix::mm::mmap(
            ptr::null_mut(),
            BUFFER_SIZE_BYTES,
            ProtFlags::READ | ProtFlags::WRITE,
            MapFlags::SHARED,
            &shm_fd,
            0,
        )
        .unwrap()
        .cast::<u32>()
    };

    rustix::shm::unlink(&shm_path).unwrap();

    assert!(!shm_ptr.is_null());
    assert!(shm_ptr.is_aligned());

    unsafe { shm_ptr.write_bytes(0xFF, BUFFER_SIZE_PIXELS) };

    let _buffer = unsafe { slice::from_raw_parts_mut(shm_ptr.cast::<u32>(), BUFFER_SIZE_PIXELS) };

    let shm_pool: WlObjectHandle<ShmPool> = shm.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlShmCreatePoolRequest {
            fd: shm_fd.as_fd(),
            size: BUFFER_SIZE_BYTES as i32,
        },
    );

    let buffer: WlObjectHandle<Buffer> = shm_pool.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlShmPoolCreateBufferRequest {
            offset: 0,
            width: BUFFER_WIDTH_PIXELS as i32,
            height: BUFFER_HEIGHT_PIXELS as i32,
            stride: (BUFFER_WIDTH_PIXELS * PIXEL_SIZE_BYTES) as i32,
            format: WlShmFormat::Xrgb8888,
        },
    );

    surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        WlSurfaceSetBufferScaleRequest { scale: 1 },
    );

    surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        WlSurfaceAttachRequest {
            buffer: Some(buffer.id()),
            x: 0,
            y: 0,
        },
    );

    surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        WlSurfaceDamageRequest {
            x: 0,
            y: 0,
            width: BUFFER_WIDTH_PIXELS as i32,
            height: BUFFER_HEIGHT_PIXELS as i32,
        },
    );

    surface.request(&mut buf, &queue.as_ref().storage(), WlSurfaceCommitRequest);

    display.roundtrip(queue.as_mut(), state.as_ref());

    thread::sleep(Duration::from_millis(200));
}
