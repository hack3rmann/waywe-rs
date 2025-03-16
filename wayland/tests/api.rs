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
use tracing_test::traced_test;
use wayland::{
    init::connect_wayland_socket,
    interface::{
        LayerSurfaceSetAnchorRequest, LayerSurfaceSetExclusiveZoneRequest,
        LayerSurfaceSetKeyboardInteractivityRequest, LayerSurfaceSetMarginRequest,
        LayerSurfaceSetSizeRequest, WlCompositorCreateRegion, WlCompositorCreateSurface,
        WlRegionDestroyRequest, WlShmCreatePoolRequest, WlShmFormat, WlShmPoolCreateBufferRequest,
        WlSurfaceAttachRequest, WlSurfaceCommitRequest, WlSurfaceDamageRequest,
        WlSurfaceSetBufferScaleRequest, WlSurfaceSetInputRegionRequest,
        WpViewporterGetViewportRequest, ZwlrLayerShellGetLayerSurfaceRequest,
        ZwlrLayerShellV1Layer,
        zwlr_layer_surface_v1::wl_enum::{Anchor, KeyboardInteractivity},
    },
    object::WlObjectType,
    sys::{
        display::WlDisplay,
        object::default_impl::{
            WlCompositor, WlOutput, WlShm,
            WlViewporter, WlrLayerShellV1,
        },
        object::registry::WlRegistry,
        wire::SmallVecMessageBuffer,
    },
};

unsafe fn connect_display() -> WlDisplay {
    let wayland_sock = unsafe { connect_wayland_socket().unwrap() };
    WlDisplay::connect_to_fd(wayland_sock).unwrap()
}

#[test]
fn just_connect_display() {
    _ = unsafe { connect_display() };
}

#[test]
fn get_registry() {
    let mut buf = SmallVecMessageBuffer::<8>::new();

    // Safety: called once on the start of the program
    let display = unsafe { connect_display() };
    let mut storage = pin!(display.create_storage());
    let registry = display.create_registry(&mut buf, storage.as_mut());

    display.dispatch_all_pending(storage.as_mut());

    assert!(
        storage
            .object(registry)
            .interfaces
            .contains_key(&WlObjectType::Compositor)
    );
}

#[test]
fn create_surface() {
    let mut buf = SmallVecMessageBuffer::<8>::new();

    // Safety: called once on the start of the program
    let display = unsafe { connect_display() };
    let mut storage = pin!(display.create_storage());
    let registry = display.create_registry(&mut buf, storage.as_mut());

    display.dispatch_all_pending(storage.as_mut());

    let compositor =
        WlRegistry::bind::<WlCompositor>(&mut buf, storage.as_mut(), registry).unwrap();

    let surface = compositor.create_object(&mut buf, storage.as_mut(), WlCompositorCreateSurface);

    assert_eq!(
        storage.object(surface).proxy().interface_name(),
        "wl_surface",
    );
}

#[test]
fn bind_wlr_shell() {
    let mut buf = SmallVecMessageBuffer::<8>::new();

    // Safety: called once on the start of the program
    let display = unsafe { connect_display() };
    let mut storage = pin!(display.create_storage());
    let registry = display.create_registry(&mut buf, storage.as_mut());

    display.dispatch_all_pending(storage.as_mut());

    let _layer_shell =
        WlRegistry::bind::<WlrLayerShellV1>(&mut buf, storage.as_mut(), registry).unwrap();

    display.dispatch_all_pending(storage.as_mut());
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
#[traced_test]
fn white_rect() {
    let mut buf = SmallVecMessageBuffer::<8>::new();

    // Safety: called once on the start of the program
    let display = unsafe { connect_display() };
    let mut storage = pin!(display.create_storage());
    let registry = display.create_registry(&mut buf, storage.as_mut());

    display.dispatch_all_pending(storage.as_mut());

    let shm = WlRegistry::bind::<WlShm>(&mut buf, storage.as_mut(), registry).unwrap();

    let viewporter =
        WlRegistry::bind::<WlViewporter>(&mut buf, storage.as_mut(), registry).unwrap();

    let compositor =
        WlRegistry::bind::<WlCompositor>(&mut buf, storage.as_mut(), registry).unwrap();

    let surface = compositor.create_object(&mut buf, storage.as_mut(), WlCompositorCreateSurface);

    let _viewport = viewporter.create_object(
        &mut buf,
        storage.as_mut(),
        WpViewporterGetViewportRequest { surface },
    );

    let region = compositor.create_object(&mut buf, storage.as_mut(), WlCompositorCreateRegion);

    surface.request(
        &mut buf,
        &storage,
        WlSurfaceSetInputRegionRequest {
            region: Some(region),
        },
    );

    region.request(&mut buf, &storage, WlRegionDestroyRequest);
    storage.release(region).unwrap();

    let output = WlRegistry::bind::<WlOutput>(&mut buf, storage.as_mut(), registry).unwrap();

    let layer_shell =
        WlRegistry::bind::<WlrLayerShellV1>(&mut buf, storage.as_mut(), registry).unwrap();

    let layer_surface = layer_shell.create_object(
        &mut buf,
        storage.as_mut(),
        ZwlrLayerShellGetLayerSurfaceRequest {
            surface,
            output: Some(output),
            layer: ZwlrLayerShellV1Layer::Background,
            namespace: c"wallpaper-engine",
        },
    );

    layer_surface.request(
        &mut buf,
        &storage,
        LayerSurfaceSetAnchorRequest {
            anchor: Anchor::all(),
        },
    );

    layer_surface.request(
        &mut buf,
        &storage,
        LayerSurfaceSetExclusiveZoneRequest { zone: -1 },
    );

    layer_surface.request(&mut buf, &storage, LayerSurfaceSetMarginRequest::zero());

    layer_surface.request(
        &mut buf,
        &storage,
        LayerSurfaceSetKeyboardInteractivityRequest {
            keyboard_interactivity: KeyboardInteractivity::None,
        },
    );

    layer_surface.request(
        &mut buf,
        &storage,
        LayerSurfaceSetSizeRequest {
            width: BUFFER_WIDTH_PIXELS as u32,
            height: BUFFER_HEIGHT_PIXELS as u32,
        },
    );

    surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

    display.dispatch_all_pending(storage.as_mut());

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

    let shm_pool = shm.create_object(
        &mut buf,
        storage.as_mut(),
        WlShmCreatePoolRequest {
            fd: shm_fd.as_fd(),
            size: BUFFER_SIZE_BYTES as i32,
        },
    );

    let buffer = shm_pool.create_object(
        &mut buf,
        storage.as_mut(),
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
        &storage,
        WlSurfaceSetBufferScaleRequest { scale: 1 },
    );

    surface.request(
        &mut buf,
        &storage,
        WlSurfaceAttachRequest {
            buffer: Some(buffer),
            x: 0,
            y: 0,
        },
    );

    surface.request(
        &mut buf,
        &storage,
        WlSurfaceDamageRequest {
            x: 0,
            y: 0,
            width: BUFFER_WIDTH_PIXELS as i32,
            height: BUFFER_HEIGHT_PIXELS as i32,
        },
    );

    surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

    display.dispatch_all_pending(storage.as_mut());

    thread::sleep(Duration::from_millis(200));
}
