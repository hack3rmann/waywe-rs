use rustix::{
    fs::{self, FlockOperation},
    io::Errno,
    net::{self, AddressFamily, SocketAddrUnix, SocketFlags, SocketType},
};
use std::{env, ffi::OsString, fmt::Write as _, os::fd::OwnedFd, path::PathBuf};
use tracing::warn;

#[derive(Debug)]
pub struct WaylandSocketCreateInfo {
    pub socket: OwnedFd,
    pub display_name: OsString,
    pub socket_path: PathBuf,
}

pub fn other_display_name_from_env() -> OsString {
    let Some(taken_name) = env::var_os("WAYLAND_DISPLAY") else {
        return OsString::from("wayland-0");
    };

    let mut name = OsString::new();

    // TODO(hack3rmann): increment display number in-place
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

pub fn create_wayland_socket() -> Result<WaylandSocketCreateInfo, Errno> {
    let xdg_runtime_dir: PathBuf = env::var_os("XDG_RUNTIME_DIR")
        .unwrap_or_else(|| {
            warn!("XDG_RUNTIME_DIR env variable not set");

            let real_user_id = rustix::process::getuid();
            OsString::from(format!("/run/user/{}", real_user_id.as_raw()))
        })
        .into();

    let display_name = other_display_name_from_env();

    let socket_path = {
        let mut dir = xdg_runtime_dir;
        dir.push(&display_name);
        dir
    };

    let addr = SocketAddrUnix::new(&socket_path).expect("addr is correct");

    let socket = net::socket_with(
        AddressFamily::UNIX,
        SocketType::STREAM,
        // NOTE(hack3rmann): wayland socket *should* be `CLOEXEC`
        SocketFlags::CLOEXEC,
        None,
    )?;

    // NOTE(hack3rmann): wayland socket should be exclusively locked
    fs::flock(&socket, FlockOperation::LockExclusive)?;

    // HACK(hack3rmann): unbounded loop
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
            Err(error) => return Err(error),
        }
    }

    net::listen(&socket, 0)?;

    Ok(WaylandSocketCreateInfo {
        socket,
        socket_path,
        display_name,
    })
}
