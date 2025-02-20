pub mod wayland;

use std::collections::HashMap;
use std::{env, error::Error, os::unix::net::UnixStream};
use wayland::interface::{
    AnyEvent, Event, NewId, Request, WlCallbackDoneEvent, WlDisplayDeleteIdEvent,
    WlDisplayGetRegistryRequest, WlDisplaySyncRequest, WlRegistryBindRequest,
    WlRegistryGlobalEvent,
};
use wayland::object::ObjectId;
use wayland::wire::{self, Message, MessageBuffer, MessageBuildError};

fn get_socket_path() -> Option<String> {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
    let display_name = env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| String::from("wayland-0"));

    Some(format!("{xdg_runtime_dir}/{display_name}"))
}

#[derive(Clone, Default, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
struct InterfaceDesc {
    object_name: ObjectId,
    version: u32,
}

fn get_registry(
    sock: &mut UnixStream,
    buf: &mut MessageBuffer,
) -> Result<HashMap<String, InterfaceDesc>, MessageBuildError> {
    WlDisplayGetRegistryRequest {
        registry: ObjectId::WL_REGISTRY,
    }
    .send(sock, buf)?;

    let mut registry = HashMap::<String, InterfaceDesc>::new();

    WlDisplaySyncRequest {
        callback: ObjectId::WL_CALLBACK,
    }
    .send(sock, buf)?;

    loop {
        wire::read_message_into(sock, buf)?;
        let message = Message::from_u32_slice(buf.as_slice());

        let Some(global) = WlRegistryGlobalEvent::from_message(message) else {
            let Some(_done) = WlCallbackDoneEvent::from_message(message) else {
                panic!("wrong message");
            };

            break;
        };

        registry.insert(
            global.interface.to_owned(),
            InterfaceDesc {
                object_name: global.name,
                version: global.version,
            },
        );
    }

    let remove_id = WlDisplayDeleteIdEvent::recv(sock, buf)?;

    assert_eq!(remove_id.id, ObjectId::WL_CALLBACK.into());

    Ok(registry)
}

fn main() -> Result<(), Box<dyn Error>> {
    let socket_path = get_socket_path().expect("failed to get wayland socket path");

    let mut sock = UnixStream::connect(socket_path)?;
    let mut buf = MessageBuffer::new();

    let registry = get_registry(&mut sock, &mut buf)?;
    let wl_compositor_interface = "wl_compositor";
    let wl_compositor = registry[wl_compositor_interface];

    WlRegistryBindRequest {
        name: wl_compositor.object_name,
        new_id: NewId {
            id: ObjectId::WL_COMPOSITOR,
            interface: wl_compositor_interface,
            version: wl_compositor.version,
        },
    }
    .send(&mut sock, &mut buf)?;

    wire::read_message_into(&mut sock, &mut buf)?;
    let event = AnyEvent::from(buf.get_message());
    dbg!(event);

    Ok(())
}
