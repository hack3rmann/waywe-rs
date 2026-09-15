pub mod client;
pub mod server;

use std::{env, sync::OnceLock};
use tracing::warn;

pub const BUFFER_SIZE: usize = 256;

pub(crate) fn socket_file_from_env() -> String {
    let runtime = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let uid = rustix::process::getuid();
        format!("/run/user/{}", uid.as_raw())
    });

    let display = if let Ok(wayland_socket) = std::env::var("WAYLAND_DISPLAY") {
        let mut i = 0;
        // if WAYLAND_DISPLAY is a full path, use only its final component
        for (j, ch) in wayland_socket.bytes().enumerate().rev() {
            if ch == b'/' {
                i = j + 1;
                break;
            }
        }
        wayland_socket[i..].to_owned()
    } else {
        warn!("WAYLAND_DISPLAY variable not set. Defaulting to wayland-0");
        "wayland-0.sock".to_owned()
    };

    format!("{runtime}/waywe-{display}.sock")
}

pub fn socket_file() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(socket_file_from_env)
}
