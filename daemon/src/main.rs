pub mod wayland;

use std::collections::HashMap;
use std::{env, error::Error, io::Write, os::unix::net::UnixStream};
use wayland::object::{ObjectId, ObjectIdProvider, WL_DISPLAY, WL_REGISTRY};
use wayland::wire::{self, Message, MessageReader};

fn get_socket_path() -> Option<String> {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
    let display_name = env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| String::from("wayland-0"));

    Some(format!("{xdg_runtime_dir}/{display_name}"))
}

#[derive(Clone, Default, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
struct InterfaceDesc {
    object_name: u32,
    version: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let socket_path = get_socket_path().expect("failed to get wayland socket path");
    let mut sock = UnixStream::connect(socket_path)?;

    let mut _id_provider = ObjectIdProvider::new();

    let mut buf = Vec::new();

    Message::builder(&mut buf)
        .object_id(WL_DISPLAY)
        .request_id(1)
        .uint(2)
        .build()
        .unwrap()
        .send(&mut sock)?;

    let mut registry = HashMap::<String, InterfaceDesc>::new();

    for _ in 0..53 {
        wire::read_message_into(&mut sock, &mut buf)?;

        let message = Message::from_u32_slice(&buf);
        let mut reader = MessageReader::new(&message);

        let object_name = reader.read_u32().unwrap();
        let interface_name = reader.read_str().unwrap();
        let version = reader.read_u32().unwrap();

        registry.insert(
            interface_name.to_owned(),
            InterfaceDesc {
                object_name,
                version,
            },
        );
    }

    let _wl_shm_desc = registry["wl_shm"];

    Ok(())
}
