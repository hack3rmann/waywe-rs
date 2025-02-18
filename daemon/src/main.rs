pub mod wayland;

use std::{env, error::Error, io::Write, os::unix::net::UnixStream};
use wayland::wire::{self, Message, MessageReader};

fn get_socket_path() -> Option<String> {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
    let display_name = env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| String::from("wayland-0"));

    Some(format!("{xdg_runtime_dir}/{display_name}"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let socket_path = get_socket_path().expect("failed to get wayland socket path");
    let mut sock = UnixStream::connect(socket_path)?;

    sock.write_all(bytemuck::cast_slice(&[0x00000001, 0x000C0001, 0x00000002]))?;

    let mut buf = Vec::new();
    wire::read_message_into(&mut sock, &mut buf)?;

    let message = Message::from_u32_slice(&buf);

    println!("message: {message:?},\nheader: {:?}", message.header());

    let mut reader = MessageReader::new(&message);

    let numeric_name = reader.read_u32().unwrap();
    let interface_name = reader.read_str().unwrap();
    let version = reader.read_u32().unwrap();

    println!("name: {numeric_name},\ninterface: '{interface_name}',\nversion: {version}");

    Ok(())
}
