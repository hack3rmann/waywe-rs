pub mod wayland;

use std::collections::HashMap;
use std::{env, error::Error, os::unix::net::UnixStream};
use wayland::object::{ObjectId, ObjectIdProvider};
use wayland::wire::{self, Message, MessageBuffer, MessageBuildError, MessageReader};
use wayland::interface::{Event, WlDisplayErrorEvent};

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
    buf: &mut MessageBuffer,
) -> Result<HashMap<String, InterfaceDesc>, MessageBuildError> {
    Message::builder(buf)
        .object_id(ObjectId::WL_DISPLAY)
        .opcode(1)
        .uint(2)
        .build_send(sock)?;

    let mut registry = HashMap::<String, InterfaceDesc>::new();

    for _ in 0..53 {
        wire::read_message_into(sock, buf)?;

        let message = Message::from_u32_slice(buf.as_slice());
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

fn _wl_bind(
    sock: &mut UnixStream,
    buf: &mut MessageBuffer,
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

    let mut _id_provider = ObjectIdProvider::new();
    let mut buf = MessageBuffer::new();

    let _registry = get_registry(&mut sock, &mut buf)?;

    Message::builder(&mut buf)
        .object_id(ObjectId::WL_DISPLAY)
        .opcode(42)
        .uint(69)
        .build_send(&mut sock)?;

    dbg!(WlDisplayErrorEvent::recv(&mut sock, &mut buf)?);

    Ok(())
}
