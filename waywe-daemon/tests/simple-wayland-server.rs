use scopeguard::defer;
use std::{
    env,
    ffi::c_void,
    mem::MaybeUninit,
    pin::pin,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
    thread,
    time::Duration,
};
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
use wayland_server::WlDisplay as WlServerDisplay;
use wayland_sys::*;

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
fn run_simple_unsafe_server() {
    _ = tracing_subscriber::fmt::try_init();

    let display = WlServerDisplay::create().unwrap();

    DISPLAY.store(display.as_raw().as_ptr(), Ordering::Relaxed);
    setup_signals();

    let _timeout_join = thread::spawn(move || {
        const TIMEOUT: Duration = Duration::from_millis(500);
        thread::sleep(TIMEOUT);
        unsafe { wl_display_terminate(DISPLAY.load(Ordering::Relaxed)) };
    });

    const CHANGE_WAYLAND_DISPLAY_ENV: bool = false;

    if CHANGE_WAYLAND_DISPLAY_ENV {
        unsafe { env::set_var("WAYLAND_DISPLAY", display.name()) };
        _ = dbg!(env::var("WAYLAND_DISPLAY"));
    }

    let mut server_state = ServerState::default();

    #[repr(C)]
    struct WlOutputInterface {
        pub release: unsafe extern "C" fn(client: *mut wl_client, resource: *mut wl_resource),
    }

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
            display.as_raw().as_ptr(),
            &raw const wl_output_interface,
            4,
            (&raw mut server_state).cast(),
            wl_output_handle_bind,
        )
    };

    defer! {
        unsafe { wl_global_destroy(output_global) };
    }

    unsafe { wl_display_run(display.as_raw().as_ptr()) };

    dbg!("done");
}

#[derive(Debug)]
struct ClientState;

#[derive(Default)]
struct ClientOutput;

impl HasObjectType for ClientOutput {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Output;
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
#[ignore = "custom server may not run"]
fn run_simple_client_for_custom_server() {
    _ = tracing_subscriber::fmt::try_init();

    unsafe { env::set_var("WAYLAND_DISPLAY", "wayland-2") };

    let client_state = pin!(ClientState);
    let mut buf = WlStackMessageBuffer::new();

    let display = WlDisplay::connect(client_state.as_ref()).unwrap();

    let mut main_queue = pin!(display.take_main_queue().unwrap());

    let registry = display
        .create_registry(&mut buf, main_queue.as_mut().storage_mut())
        .handle();

    display.roundtrip(main_queue.as_mut(), client_state.as_ref());

    dbg!(main_queue.as_ref().storage().object_data(registry));

    let _output = registry
        .bind::<ClientOutput>(&mut buf, main_queue.as_mut().storage_mut())
        .unwrap();

    display.roundtrip(main_queue.as_mut(), client_state.as_ref());
}
