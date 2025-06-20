use super::protocol_from_str;
use crate::xml::{Interface, InterfaceEntry, Message};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::{ffi::CString, fs, path::PathBuf};
use syn::{Ident, LitCStr, LitInt, LitStr, parse2};

pub fn include_wl_interfaces(token_stream: TokenStream) -> TokenStream {
    let str_lit = parse2::<LitStr>(token_stream).expect("expecting string literal");
    let path = PathBuf::from(str_lit.value());
    let file_contents = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read file '{}': {err}", path.display()));

    let protocol = protocol_from_str(&file_contents)
        .unwrap_or_else(|err| panic!("failed to parse protocol from {}: {err}", path.display()));

    let interfaces = protocol.interface.as_slice();
    let modules = interfaces.iter().map(interface_to_module);

    quote! { #( #modules )* }
}

pub fn interface_to_module(interface: &Interface) -> TokenStream {
    let module_docs = format!("`wl_interface`s for {}", interface.name);
    let const_docs = format!("`Interface` for {}", interface.name);
    let messages_docs = format!("`WlMessages` for {}", interface.name);
    let wl_interface_docs = format!("`wl_interface` for {}", interface.name);

    let module = Ident::new(&interface.name, Span::call_site());

    let interface_name_cstring = CString::new(interface.name.as_bytes())
        .expect("interface name expected to be a valid c-string");
    let interface_name_cstr_lit = LitCStr::new(&interface_name_cstring, Span::call_site());

    assert_ne!(interface.version, 0);
    let interface_version = interface.version.to_string();
    let interface_version_int_lit = LitInt::new(&interface_version, Span::call_site());

    let requests = interface
        .entries
        .iter()
        .filter_map(|entry| match entry {
            InterfaceEntry::Request(message) => Some(message),
            _ => None,
        })
        .map(|message| message_to_struct(interface, message));

    let events = interface
        .entries
        .iter()
        .filter_map(|entry| match entry {
            InterfaceEntry::Event(message) => Some(message),
            _ => None,
        })
        .map(|message| message_to_struct(interface, message));

    let requests_wl_messages = interface
        .entries
        .iter()
        .filter_map(|entry| match entry {
            InterfaceEntry::Request(message) => Some(message),
            _ => None,
        })
        .enumerate()
        .map(|(i, request)| message_to_wl_message(request, i, MessageType::Request));

    let events_wl_messages = interface
        .entries
        .iter()
        .filter_map(|entry| match entry {
            InterfaceEntry::Event(message) => Some(message),
            _ => None,
        })
        .enumerate()
        .map(|(i, event)| message_to_wl_message(event, i, MessageType::Event));

    quote! {
        #[doc = #module_docs ]
        pub mod #module {
            #[doc = #const_docs ]
            pub const INTERFACE: ::wayland_sys::Interface<'static>
                = ::wayland_sys::Interface {
                    name: #interface_name_cstr_lit,
                    version: ::std::num::NonZeroU32::new(#interface_version_int_lit).unwrap(),
                    methods: &[
                        #( #requests ),*
                    ],
                    events: &[
                        #( #events ),*
                    ],
                };

            #[doc = #messages_docs ]
            pub static WL_MESSAGES: ::wayland_sys::InterfaceWlMessages<'static>
                = ::wayland_sys::InterfaceWlMessages {
                    methods: &[
                        #( #requests_wl_messages ),*
                    ],
                    events: &[
                        #( #events_wl_messages ),*
                    ],
                };

            #[doc = #wl_interface_docs ]
            pub static WL_INTERFACE: ::wayland_sys::wl_interface
                = ::wayland_sys::wl_interface {
                    name: INTERFACE.name.as_ptr(),
                    version: INTERFACE.version.get() as i32,
                    method_count: WL_MESSAGES.methods.len() as i32,
                    methods: WL_MESSAGES.methods.as_ptr(),
                    event_count: WL_MESSAGES.events.len() as i32,
                    events: WL_MESSAGES.events.as_ptr(),
                };
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub enum MessageType {
    #[default]
    Request,
    Event,
}

impl MessageType {
    pub const fn str(self) -> &'static str {
        match self {
            Self::Request => "methods",
            Self::Event => "events",
        }
    }
}

pub fn maybe_name_to_outgoing_interface(parent: &Interface, name: Option<&str>) -> TokenStream {
    match name {
        Some(name) => {
            let module = Ident::new(name, Span::call_site());

            if name == parent.name {
                quote! { ::wayland_sys::OutgoingInterface::This }
            } else {
                quote! {
                    ::wayland_sys::OutgoingInterface::Other(
                        &super:: #module ::INTERFACE
                    )
                }
            }
        }
        None => quote! { ::wayland_sys::OutgoingInterface::None },
    }
}

pub fn message_to_wl_message(message: &Message, index: usize, ty: MessageType) -> TokenStream {
    let index_string = index.to_string();
    let index_lit = LitInt::new(&index_string, Span::call_site());

    let outgoing_interfaces = message
        .arg
        .iter()
        .map(|arg| arg.interface.as_deref())
        .map(|name| name.map(|name| Ident::new(name, Span::call_site())))
        .map(|module| match module {
            Some(module) => quote! {
                ::core::option::Option::Some(
                    &super:: #module ::WL_INTERFACE
                )
            },
            None => quote! {
                ::core::option::Option::None
            },
        });

    let slice_name = Ident::new(ty.str(), Span::call_site());

    quote! {
        ::wayland_sys::wl_message {
            name: INTERFACE. #slice_name [ #index_lit ].name.as_ptr(),
            signature: INTERFACE. #slice_name [ #index_lit ].signature.as_ptr(),
            types: {
                static REF: &[Option<&::wayland_sys::wl_interface>] = &[
                    #( #outgoing_interfaces ),*
                ];

                if !REF.is_empty() {
                    // Safety: transmuting `*const Option<&T>` to `*const *const T`
                    // None transmutes to `ptr::null()`
                    unsafe {
                        ::std::mem::transmute::<
                            *const Option<&::wayland_sys::wl_interface>,
                            *const *const ::wayland_sys::wl_interface,
                        >(REF.as_ptr())
                    }
                } else {
                    ::std::ptr::dangling()
                }
            },
        }
    }
}

pub fn message_to_struct(parent: &Interface, message: &Message) -> TokenStream {
    let request_name_c_str =
        CString::new(message.name.as_ref()).expect("expecting a valid c-string");
    let request_name_field_literal = LitCStr::new(&request_name_c_str, Span::call_site());

    let request_signature = message.signature();
    let request_signature_literal = LitCStr::new(&request_signature, Span::call_site());

    let outgoing_interfaces = message
        .arg
        .iter()
        .map(|arg| arg.interface.as_deref())
        .map(|name| maybe_name_to_outgoing_interface(parent, name));

    quote! {
        ::wayland_sys::InterfaceMessage {
            name: #request_name_field_literal,
            signature: #request_signature_literal,
            outgoing_interfaces: &[
                #( #outgoing_interfaces ),*
            ],
        }
    }
}
