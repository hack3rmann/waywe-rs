use crate::xml::{Interface, InterfaceEntry, Message, ProtocolFile};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use smallvec::SmallVec;
use std::{ffi::CString, fs, path::PathBuf};
use syn::{Ident, LitCStr, LitInt, LitStr, parse2};

pub fn include_wl_interfaces(token_stream: TokenStream) -> TokenStream {
    let str_lit = parse2::<LitStr>(token_stream).expect("expecting string literal");
    let path = PathBuf::from(str_lit.value());
    let file_contents = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read file '{}': {err}", path.display()));

    let protocol = xml_serde::from_str::<ProtocolFile>(&file_contents)
        .unwrap_or_else(|err| panic!("failed to parse protocol from {}: {err}", path.display()));

    let interfaces = protocol.protocol.interface.as_slice();
    let modules = interfaces.iter().map(interface_to_module);

    quote! { #( #modules )* }
}

pub fn interface_to_module(interface: &Interface) -> TokenStream {
    let module = Ident::new(&interface.name, Span::call_site());

    let interface_name_string = interface.name.to_uppercase();
    let interface_name_cstring = CString::new(interface_name_string.as_bytes())
        .expect("interface name expected to be a valid c-string");
    let interface_name = Ident::new(&interface_name_string, Span::call_site());
    let interface_name_cstr_lit = LitCStr::new(&interface_name_cstring, Span::call_site());

    let interface_version = interface.version.to_string();
    let interface_version_int_lit = LitInt::new(&interface_version, Span::call_site());

    let requests = interface
        .entries
        .iter()
        .filter_map(|entry| match entry {
            InterfaceEntry::Request(message) => Some(message),
            _ => None,
        })
        .map(message_to_struct);

    let events = interface
        .entries
        .iter()
        .filter_map(|entry| match entry {
            InterfaceEntry::Event(message) => Some(message),
            _ => None,
        })
        .map(message_to_struct);

    quote! {
        pub mod #module {
            pub const #interface_name: ::wayland_sys::Interface<'static>
                = ::wayland_sys::Interface {
                    name: #interface_name_cstr_lit,
                    version: #interface_version_int_lit,
                    methods: &[
                        #( #requests ),*
                    ],
                    events: &[
                        #( #events ),*
                    ],
                };
        }
    }
}

pub fn message_to_struct(request: &Message) -> TokenStream {
    let request_name_c_str =
        CString::new(request.name.as_ref()).expect("expecting a valid c-string");
    let request_name_field_literal = LitCStr::new(&request_name_c_str, Span::call_site());

    let request_signature = request.signature();
    let request_signature_literal = LitCStr::new(&request_signature, Span::call_site());

    let outgoing_interfaces_strings = request
        .arg
        .iter()
        .filter_map(|arg| arg.interface.as_deref())
        .map(str::to_ascii_uppercase)
        .collect::<SmallVec<[_; 2]>>();

    let outgoing_interfaces_modules = request
        .arg
        .iter()
        .filter_map(|arg| arg.interface.as_deref())
        .map(|name| Ident::new(name, Span::call_site()));

    let outgoing_interfaces = outgoing_interfaces_strings
        .iter()
        .map(|s| Ident::new(s, Span::call_site()));

    quote! {
        ::wayland_sys::InterfaceMessage {
            name: #request_name_field_literal,
            signature: #request_signature_literal,
            outgoing_interfaces: &[
                #( & super :: #outgoing_interfaces_modules :: #outgoing_interfaces ),*
            ],
        }
    }
}
