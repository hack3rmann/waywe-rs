pub mod wayland;

use rustix::path::Arg;
use std::collections::HashMap;
use std::{env, error::Error, os::unix::net::UnixStream};
use wayland::object::{ObjectId, ObjectIdProvider};
use wayland::wire::{self, Message, MessageBuildError, MessageReader};

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

fn get_registry(
    sock: &mut UnixStream,
    buf: &mut Vec<u32>,
) -> Result<HashMap<String, InterfaceDesc>, MessageBuildError> {
    Message::builder(buf)
        .object_id(ObjectId::WL_DISPLAY)
        .opcode(1)
        .uint(2)
        .build_send(sock)?;

    let mut registry = HashMap::<String, InterfaceDesc>::new();

    for _ in 0..53 {
        wire::read_message_into(sock, buf)?;

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

    Ok(registry)
}

fn wl_bind(
    sock: &mut UnixStream,
    buf: &mut Vec<u32>,
    object_name: u32,
    id: ObjectId,
) -> Result<(), MessageBuildError> {
    Message::builder(buf)
        .object_id(ObjectId::WL_REGISTRY)
        .opcode(0)
        .uint(object_name)
        .uint(id.into())
        .build_send(sock)
}

fn main() -> Result<(), Box<dyn Error>> {
    let socket_path = get_socket_path().expect("failed to get wayland socket path");
    let mut sock = UnixStream::connect(socket_path)?;

    let mut id_provider = ObjectIdProvider::new();
    let mut buf = Vec::new();

    let registry = get_registry(&mut sock, &mut buf)?;

    let wl_compositor = registry["wl_compositor"];
    let wl_compositor_id = id_provider.next_id();

    wl_bind(
        &mut sock,
        &mut buf,
        dbg!(wl_compositor.object_name),
        wl_compositor_id,
    )?;

    loop {
        wire::read_message_into(&mut sock, &mut buf)?;
        let message = Message::from_u32_slice(&mut buf);
        dbg!(message.as_bytes().to_string_lossy(), message.header());
    }
}
