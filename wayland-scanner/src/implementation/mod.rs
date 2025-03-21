pub mod our;
pub mod wl;

use crate::xml::{Arg, ArgType, InterfaceEntry, Message, Protocol, ProtocolFile};
use smallvec::smallvec;
use std::borrow::Cow;

pub fn registry_bind_message<'s>() -> Message<'s> {
    Message {
        name: Cow::Borrowed("bind"),
        since: None,
        description: None,
        arg: smallvec![
            Arg {
                name: Cow::Borrowed("name"),
                ty: ArgType::Uint,
                summary: Some(Cow::Borrowed("unique numeric name of the object")),
                interface: None,
                allow_null: false,
                enumeration: None,
            },
            Arg {
                name: Cow::Borrowed("interface"),
                ty: ArgType::String,
                summary: Some(Cow::Borrowed("interface of the object")),
                interface: None,
                allow_null: false,
                enumeration: None,
            },
            Arg {
                name: Cow::Borrowed("version"),
                ty: ArgType::Uint,
                summary: Some(Cow::Borrowed("version of the object's interface")),
                interface: None,
                allow_null: false,
                enumeration: None,
            },
            Arg {
                name: Cow::Borrowed("id"),
                ty: ArgType::NewId,
                summary: Some(Cow::Borrowed("bounded object")),
                interface: None,
                allow_null: false,
                enumeration: None,
            },
        ],
    }
}

pub fn protocol_from_str(source: &str) -> Result<Protocol<'_>, xml_serde::Error> {
    let mut protocol = xml_serde::from_str::<ProtocolFile>(source)?;

    // replace arguments for wl_registry::bind
    for interface in &mut protocol.protocol.interface {
        if interface.name != "wl_registry" {
            continue;
        }

        for entry in &mut interface.entries {
            let InterfaceEntry::Request(message) = entry else {
                continue;
            };

            if message.name == "bind" {
                *message = registry_bind_message();
            }
        }
    }

    Ok(protocol.protocol)
}
