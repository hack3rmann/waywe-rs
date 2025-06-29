use rustix::{
    fs::{self, FlockOperation},
    io::Errno,
    net::{self, AddressFamily, SocketAddrUnix, SocketFlags, SocketType},
};
use scopeguard::defer;
use std::{
    env,
    ffi::OsString,
    fmt::Write as _,
    os::fd::{IntoRawFd, OwnedFd},
    path::PathBuf,
};
use tracing::warn;
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

#[test]
fn run_server() {
    _ = tracing_subscriber::fmt::try_init();

    let display = unsafe { wl_display_create() };
    assert!(!display.is_null());

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

    unsafe { env::set_var("WAYLAND_DISPLAY", &display_name) };

    _ = dbg!(env::var("WAYLAND_DISPLAY"));

    unsafe { wl_display_run(display) };
}
