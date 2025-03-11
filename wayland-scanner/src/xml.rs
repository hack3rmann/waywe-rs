use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Unexpected, Visitor},
};
use smallvec::SmallVec;
use std::{borrow::Cow, ffi::CString, fmt, str::FromStr};
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug)]
pub struct Description<'s> {
    #[serde(rename = "$attr:summary")]
    pub summary: Option<Cow<'s, str>>,
    #[serde(rename = "$value")]
    pub body: Option<Cow<'s, str>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ArgType {
    Int,
    Uint,
    NewId,
    Object,
    String,
    Fd,
    Fixed,
    Array,
}

impl ArgType {
    pub const fn byte(self) -> u8 {
        match self {
            Self::Int => b'i',
            Self::Uint => b'u',
            Self::NewId => b'n',
            Self::Object => b'o',
            Self::String => b's',
            Self::Fd => b'h',
            Self::Fixed => b'f',
            Self::Array => b'a',
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Arg<'s> {
    #[serde(rename = "$attr:name")]
    pub name: Cow<'s, str>,
    #[serde(rename = "$attr:type")]
    pub ty: ArgType,
    #[serde(rename = "$attr:summary")]
    pub summary: Option<Cow<'s, str>>,
    #[serde(rename = "$attr:interface")]
    pub interface: Option<Cow<'s, str>>,
    #[serde(rename = "$attr:allow_null", default)]
    pub allow_null: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<'s> {
    #[serde(rename = "$attr:name")]
    pub name: Cow<'s, str>,
    #[serde(borrow)]
    pub description: Option<Description<'s>>,
    #[serde(default)]
    pub arg: SmallVec<[Arg<'s>; Message::MAX_N_ARGS]>,
}

impl Message<'_> {
    pub const MAX_N_ARGS: usize = 8;

    pub fn signature(&self) -> CString {
        let bytes = self
            .arg
            .iter()
            .map(|arg| arg.ty.byte())
            // Required by the safety arg below
            .chain([0])
            .collect::<Vec<_>>();

        // # Safety
        //
        // - all arguments have non-zero byte representation
        // - the last zero has pushed by us (see above)
        unsafe { CString::from_vec_with_nul_unchecked(bytes) }
    }
}

#[derive(Debug, PartialEq, Default, Clone, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct MaybeHexU32(pub u32);

impl Serialize for MaybeHexU32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for MaybeHexU32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MaybeHexU32Visitor;

        impl Visitor<'_> for MaybeHexU32Visitor {
            type Value = MaybeHexU32;

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|InvalidMaybeHexU32(value)| {
                    E::invalid_value(Unexpected::Str(&value), &"unsigned 32-bit integer")
                })
            }

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("expecting unsigned 32-bit integer")
            }
        }

        deserializer.deserialize_str(MaybeHexU32Visitor)
    }
}

impl FromStr for MaybeHexU32 {
    type Err = InvalidMaybeHexU32;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() >= 2 {
            if let Some(hex_prefix) = s.as_bytes().get(0..2) {
                if hex_prefix == b"0x" {
                    return u32::from_str_radix(&s[2..], 16)
                        .map(MaybeHexU32)
                        .map_err(|_| InvalidMaybeHexU32(s.to_owned()));
                }
            }
        }

        s.parse::<u32>()
            .map(MaybeHexU32)
            .map_err(|_| InvalidMaybeHexU32(s.to_owned()))
    }
}

#[derive(Debug, Error)]
#[error("failed to parse {0} as u32")]
pub struct InvalidMaybeHexU32(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct EnumEntry<'s> {
    #[serde(rename = "$attr:name")]
    pub name: Cow<'s, str>,
    #[serde(rename = "$attr:value")]
    pub value: MaybeHexU32,
    #[serde(rename = "$attr:summary")]
    pub summary: Option<Cow<'s, str>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Enum<'s> {
    #[serde(rename = "$attr:name")]
    pub name: Cow<'s, str>,
    #[serde(borrow)]
    pub description: Option<Description<'s>>,
    pub entry: SmallVec<[EnumEntry<'s>; Enum::MAX_N_ENTRIES]>,
}

impl Enum<'_> {
    pub const MAX_N_ENTRIES: usize = 16;
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum InterfaceEntry<'s> {
    #[serde(borrow)]
    Request(Message<'s>),
    #[serde(borrow)]
    Event(Message<'s>),
    #[serde(borrow)]
    Enum(Enum<'s>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Interface<'s> {
    #[serde(rename = "$attr:name")]
    pub name: Cow<'s, str>,
    #[serde(rename = "$attr:version")]
    pub version: u32,
    #[serde(borrow)]
    pub description: Option<Description<'s>>,
    #[serde(rename = "$value")]
    pub entries: SmallVec<[InterfaceEntry<'s>; Interface::MAX_N_ENTRIES]>,
}

impl Interface<'_> {
    pub const MAX_N_ENTRIES: usize = 16;
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename = "protocol")]
pub struct Protocol<'s> {
    #[serde(rename = "$attr:name")]
    pub name: Cow<'s, str>,
    #[serde(borrow)]
    pub copyright: Cow<'s, str>,
    pub interface: Vec<Interface<'s>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProtocolFile<'s> {
    #[serde(borrow)]
    pub protocol: Protocol<'s>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn serialize() {
        use smallvec::smallvec;

        let proto = ProtocolFile {
            protocol: Protocol {
                name: Cow::from("wayland"),
                copyright: Cow::from("some copyright string"),
                interface: vec![
                    Interface {
                        name: "wl_display".into(),
                        version: 1,
                        description: Some(Description {
                            summary: Some("wl_display desc".into()),
                            body: Some("interface of wl_display".into()),
                        }),
                        entries: smallvec![InterfaceEntry::Request(Message {
                            name: "get_registry".into(),
                            description: Some(Description {
                                summary: Some("get registry".into()),
                                body: Some("a request to get registry".into()),
                            }),
                            arg: smallvec![
                                Arg {
                                    name: "arg1".into(),
                                    ty: ArgType::Uint,
                                    summary: Some("first argument".into()),
                                    interface: Some("wl_registry".into()),
                                    allow_null: false,
                                },
                                Arg {
                                    name: "arg2".into(),
                                    ty: ArgType::Uint,
                                    summary: Some("second argument".into()),
                                    interface: Some("wl_registry".into()),
                                    allow_null: false,
                                }
                            ],
                        })],
                    },
                    Interface {
                        name: "wl_display".into(),
                        version: 1,
                        description: Some(Description {
                            summary: Some("wl_display desc".into()),
                            body: Some("interface of wl_display".into()),
                        }),
                        entries: smallvec![InterfaceEntry::Request(Message {
                            name: "get_registry".into(),
                            description: Some(Description {
                                summary: Some("get registry".into()),
                                body: Some("a request to get registry".into()),
                            }),
                            arg: smallvec![Arg {
                                name: "arg1".into(),
                                ty: ArgType::Uint,
                                summary: Some("first argument".into()),
                                interface: Some("wl_registry".into()),
                                allow_null: false,
                            }],
                        })],
                    },
                ],
            },
        };

        eprintln!(
            "{}",
            xml_serde::to_string_custom(
                &proto,
                xml_serde::Options {
                    include_schema_location: false
                }
            )
            .unwrap()
        );
    }

    #[test]
    fn deserialize_wayland_xml() {
        let proto_string = fs::read_to_string("../wayland-protocols/wayland.xml").unwrap();
        let proto = xml_serde::from_str::<ProtocolFile>(&proto_string).unwrap();

        dbg!(&proto);
    }
}
