use rustix::{
    fs::{self, FlockOperation},
    io::Errno,
    net::{self, AddressFamily, SocketAddrUnix, SocketFlags, SocketType},
};
use scopeguard::defer;
use std::{
    env,
    ffi::{OsString, c_void},
    fmt::Write as _,
    mem::MaybeUninit,
    os::fd::{IntoRawFd, OwnedFd},
    path::PathBuf,
    pin::pin,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};
use tracing::warn;
use wayland_client::{
    interface::{
        Event as _, WlOutputEvent, WlOutputGeometryEvent, WlOutputSubpixel, WlOutputTransform,
    },
    object::{HasObjectType, WlObjectType},
    sys::{
        display::WlDisplay,
        object::{FromProxy, dispatch::Dispatch},
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{WlMessage, WlStackMessageBuffer},
    },
};
use wayland_sys::*;

struct SocketInfo {
    pub socket: OwnedFd,
    pub display_name: OsString,
    pub socket_path: PathBuf,
}

fn create_new_display_name() -> OsString {
    let Some(taken_name) = env::var_os("WAYLAND_DISPLAY") else {
        return OsString::from("wayland-0");
    };

    let mut name = OsString::new();

    for i in 1.. {
        name.clear();
        name.push("wayland-");
        write!(&mut name, "{i}").unwrap();

        if name != taken_name {
            break;
        }
    }

    name
}

fn create_wayland_socket() -> SocketInfo {
    let xdg_runtime_dir: PathBuf = env::var_os("XDG_RUNTIME_DIR")
        .unwrap_or_else(|| {
            warn!("XDG_RUNTIME_DIR env variable not set");

            let real_user_id = rustix::process::getuid();
            OsString::from(format!("/run/user/{}", real_user_id.as_raw()))
        })
        .into();

    let display_name = create_new_display_name();
    dbg!(&display_name);

    let socket_path = {
        let mut dir = xdg_runtime_dir;
        dir.push(&display_name);
        dir
    };

    dbg!(&socket_path);

    let addr = SocketAddrUnix::new(&socket_path).expect("addr is correct");

    let socket = net::socket_with(
        AddressFamily::UNIX,
        SocketType::STREAM,
        SocketFlags::CLOEXEC,
        None,
    )
    .unwrap();

    fs::flock(&socket, FlockOperation::LockExclusive).unwrap();

    loop {
        match net::bind_unix(&socket, &addr) {
            Ok(()) => break,
            Err(Errno::ADDRINUSE) => {
                warn!(
                    ?socket_path,
                    "socket address already in use, trying to remove",
                );
                fs::unlink(&socket_path).unwrap();
            }
            Err(other) => panic!("{other:?}"),
        }
    }

    net::listen(&socket, 0).unwrap();

    SocketInfo {
        socket,
        socket_path,
        display_name,
    }
}

pub struct WlOutput {
    pub wl_resource: *mut wl_resource,
}

impl Drop for WlOutput {
    fn drop(&mut self) {
        unsafe { wl_resource_destroy(self.wl_resource) };
    }
}

#[derive(Default)]
pub struct ServerState {
    pub client_outputs: Vec<Box<WlOutput>>,
}

static DISPLAY: AtomicPtr<wl_display> = AtomicPtr::new(ptr::null_mut());

fn setup_signals() {
    extern "C" fn signal_handler(_signal_number: i32) {
        unsafe { wl_display_terminate(DISPLAY.load(Ordering::Relaxed)) };
    }

    let sa_mask = {
        let mut uninit = MaybeUninit::<libc::sigset_t>::zeroed();
        unsafe { libc::sigemptyset(uninit.as_mut_ptr()) };
        unsafe { uninit.assume_init() }
    };

    let action = libc::sigaction {
        sa_sigaction: signal_handler as usize,
        sa_mask,
        sa_flags: 0,
        sa_restorer: None,
    };

    for signal in [libc::SIGINT, libc::SIGQUIT, libc::SIGTERM, libc::SIGHUP] {
        assert_eq!(0, unsafe {
            libc::sigaction(signal, &raw const action, ptr::null_mut())
        });
    }
}

#[test]
fn run_server() {
    _ = tracing_subscriber::fmt::try_init();

    let display = wl_display_create();
    assert!(!display.is_null());

    DISPLAY.store(display, Ordering::Relaxed);
    setup_signals();

    defer! {
        unsafe { wl_display_destroy(display) };
    }

    let SocketInfo {
        socket,
        display_name,
        socket_path,
    } = create_wayland_socket();

    defer! {
        fs::unlink(&socket_path).unwrap();
    }

    assert_eq!(0, unsafe {
        wl_display_add_socket_fd(display, socket.into_raw_fd())
    });

    const CHANGE_WAYLAND_DISPLAY_ENV: bool = false;

    if CHANGE_WAYLAND_DISPLAY_ENV {
        unsafe { env::set_var("WAYLAND_DISPLAY", &display_name) };
        _ = dbg!(env::var("WAYLAND_DISPLAY"));
    }

    let mut server_state = ServerState::default();

    static WL_OUTPUT_IMPL: WlOutputInterface = WlOutputInterface {
        release: wl_output_handle_release,
    };

    unsafe extern "C" fn wl_output_handle_bind(
        client: *mut wl_client,
        data: *mut c_void,
        _version: u32,
        id: u32,
    ) {
        let server_state = unsafe { data.cast::<ServerState>().as_mut().unwrap_unchecked() };

        let wl_resource = unsafe {
            wl_resource_create(
                client,
                &raw const wl_output_interface,
                wl_output_interface.version,
                id,
            )
        };

        let mut output = Box::new(WlOutput { wl_resource });

        unsafe {
            wl_resource_set_implementation(
                wl_resource,
                (&raw const WL_OUTPUT_IMPL).cast(),
                (&raw mut *output).cast(),
                wl_output_handle_resource_destroy,
            )
        };

        let mut geometry_args = [
            WlArgument { i: 0 },   // x
            WlArgument { i: 0 },   // y
            WlArgument { i: 999 }, // width in millimeters
            WlArgument { i: 666 }, // height in millimeters
            WlArgument {
                i: WlOutputSubpixel::Unknown as i32,
            },
            // manufacturer desc
            WlArgument {
                s: c"the best monitor manufacturer".as_ptr(),
            },
            // monitor desc
            WlArgument {
                s: c"sexy monitor 777".as_ptr(),
            },
            WlArgument {
                i: WlOutputTransform::Normal as i32,
            },
        ];

        unsafe {
            wl_resource_post_event_array(
                wl_resource,
                WlOutputGeometryEvent::CODE as u32,
                geometry_args.as_mut_ptr().cast(),
            )
        };

        server_state.client_outputs.push(output);
    }

    #[repr(C)]
    struct WlOutputInterface {
        pub release: unsafe extern "C" fn(client: *mut wl_client, resource: *mut wl_resource),
    }

    unsafe extern "C" fn wl_output_handle_resource_destroy(resource: *mut wl_resource) {
        let _wl_output = unsafe {
            wl_resource_get_user_data(resource)
                .cast::<WlOutput>()
                .as_mut()
                .unwrap_unchecked()
        };
    }

    unsafe extern "C" fn wl_output_handle_release(_: *mut wl_client, resource: *mut wl_resource) {
        unsafe { wl_resource_destroy(resource) };
    }

    let output_global = unsafe {
        wl_global_create(
            display,
            &raw const wl_output_interface,
            4,
            (&raw mut server_state).cast(),
            wl_output_handle_bind,
        )
    };

    defer! {
        unsafe { wl_global_destroy(output_global) };
    }

    unsafe { wl_display_run(display) };
}

#[derive(Debug)]
struct ClientState;

struct ClientOutput;

impl HasObjectType for ClientOutput {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Output;
}

impl FromProxy for ClientOutput {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for ClientOutput {
    type State = ClientState;

    fn dispatch(
        &mut self,
        _state: &Self::State,
        _storage: &mut WlObjectStorage<Self::State>,
        message: WlMessage<'_>,
    ) {
        dbg!(message.as_event::<WlOutputEvent>());
    }
}

#[test]
fn run_client() {
    _ = tracing_subscriber::fmt::try_init();

    unsafe { env::set_var("WAYLAND_DISPLAY", "wayland-2") };

    let client_state = pin!(ClientState);
    let mut buf = WlStackMessageBuffer::new();

    let display = WlDisplay::connect(client_state.as_ref()).unwrap();

    let mut main_queue = pin!(display.take_main_queue().unwrap());

    let registry = display.create_registry(&mut buf, main_queue.as_mut().storage_mut());

    display.roundtrip(main_queue.as_mut(), client_state.as_ref());

    dbg!(main_queue.as_ref().storage().object_data(registry));

    let _output = registry
        .bind::<ClientOutput>(&mut buf, main_queue.as_mut().storage_mut())
        .unwrap();

    display.roundtrip(main_queue.as_mut(), client_state.as_ref());
}
