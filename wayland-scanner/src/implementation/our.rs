use super::protocol_from_str;
use crate::{
    fmt::{DocDescription, format_doc_string, remove_offsets},
    xml::{Arg, ArgType, Enum, Interface, InterfaceEntry, Message},
};
use convert_case::{Case, Casing as _};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::{fs, ops::Deref};
use syn::{
    LitInt, LitStr, Result as ParseResult, Token, bracketed,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    token::Bracket,
};

pub struct LitStrArray {
    pub _bracket_token: Bracket,
    pub string_literals: Punctuated<LitStr, Token![,]>,
}

impl Parse for LitStrArray {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let literals;

        Ok(Self {
            _bracket_token: bracketed!(literals in input),
            string_literals: literals.parse_terminated(<LitStr as Parse>::parse, Token![,])?,
        })
    }
}

pub fn include_interfaces(token_stream: TokenStream) -> TokenStream {
    let str_lit_array =
        parse2::<LitStrArray>(token_stream).expect("expecting array of string literals");

    let file_contents = str_lit_array
        .string_literals
        .iter()
        .map(|str_lit| {
            fs::read_to_string(str_lit.value())
                .unwrap_or_else(|err| panic!("failed to read file '{}': {err}", str_lit.value()))
        })
        .collect::<Vec<_>>();

    let protocols = file_contents
        .iter()
        .map(|contents| protocol_from_str(contents).expect("failed to parse protocol"))
        .collect::<Vec<_>>();

    let interface_names = protocols
        .iter()
        .flat_map(|protocol| {
            protocol
                .interface
                .iter()
                .map(|interface| interface.name.as_ref())
        })
        .collect::<Vec<_>>();

    let object_type_enum_variants_names = interface_names
        .iter()
        .map(|name| strip_interface_name(name).to_case(Case::Pascal))
        .collect::<Vec<_>>();

    let object_type_enum_variants = object_type_enum_variants_names
        .iter()
        .map(|name| Ident::new(name, Span::call_site()));

    let object_type_enum_variants_docs = interface_names
        .iter()
        .map(|name| format!("Type of `{name}` interface"));

    let protocol_modules = protocols.iter().map(|protocol| {
        let protocol_module_name = Ident::new(&protocol.name, Span::call_site());
        let modules = protocol.interface.iter().map(interface_to_module);

        let docs = format!(
            "# Protocol `{}`\n\n## Copyright\n\n{}",
            protocol.name,
            remove_offsets(&protocol.copyright),
        );

        quote! {
            #[doc = #docs ]
            pub mod #protocol_module_name {
                #( #modules )*
            }
        }
    });

    let phf_map_entries = interface_names
        .iter()
        .zip(&object_type_enum_variants_names)
        .map(|(interface_name, enum_entry_name)| {
            let enum_entry_ident = Ident::new(enum_entry_name, Span::call_site());
            quote! { #interface_name => WlObjectType:: #enum_entry_ident , }
        });

    let interface_map_entries =
        interface_names.iter().zip(&object_type_enum_variants_names)
        .map(|(interface_name, enum_entry_name)| {
            let enum_entry_ident = Ident::new(enum_entry_name, Span::call_site());
            let interface_ident = Ident::new(interface_name, Span::call_site());

            quote! { Self:: #enum_entry_ident => &crate::sys::protocol:: #interface_ident ::INTERFACE , }
        });

    let wl_interface_map_entries =
        interface_names.iter().zip(&object_type_enum_variants_names)
        .map(|(interface_name, enum_entry_name)| {
            let enum_entry_ident = Ident::new(enum_entry_name, Span::call_site());
            let interface_ident = Ident::new(interface_name, Span::call_site());

            quote! { Self:: #enum_entry_ident => &crate::sys::protocol:: #interface_ident ::WL_INTERFACE , }
        });

    let pub_uses = protocols.iter().map(|protocol| {
        let protocol_module = Ident::new(&protocol.name, Span::call_site());

        let interfaces = protocol.interface.iter().map(|interface| {
            let interface_module =
                Ident::new(strip_interface_name(&interface.name), Span::call_site());
            let interface_prefix = strip_v1_suffix(&interface.name).to_case(Case::Pascal);

            let interface_items = interface.entries.iter().map(|entry| match entry {
                InterfaceEntry::Request(request) => {
                    let name = request.name.to_case(Case::Pascal);
                    let ident = Ident::new(&name, Span::call_site());
                    let short_ident = format_ident!("{interface_prefix}{name}Request");

                    quote! { request:: #ident as #short_ident }
                }
                InterfaceEntry::Event(event) => {
                    let name = event.name.to_case(Case::Pascal);
                    let ident = Ident::new(&name, Span::call_site());
                    let short_ident = format_ident!("{interface_prefix}{name}Event");

                    quote! { event:: #ident as #short_ident }
                }
                InterfaceEntry::Enum(en) => {
                    let name = en.name.to_case(Case::Pascal);
                    let ident = Ident::new(&name, Span::call_site());
                    let short_ident = format_ident!("{interface_prefix}{name}");

                    quote! { wl_enum:: #ident as #short_ident }
                }
            });

            let has_event = interface
                .entries
                .iter()
                .any(|entry| matches!(entry, InterfaceEntry::Event(..)));

            let event_export = has_event.then(|| {
                let short_ident = format_ident!("{interface_prefix}Event");
                quote! { event::Event as #short_ident }
            });

            quote! {
                #interface_module ::{
                    #( #interface_items , )*
                    #event_export
                }
            }
        });

        quote! {
            #protocol_module ::{
                #( #interfaces ),*
            }
        }
    });

    quote! {
        #[doc = "Type of libwayland object"]
        #[derive(Debug, PartialEq, Default, Clone, Copy, Eq, PartialOrd, Ord, Hash)]
        pub enum WlObjectType {
            #[default]
            #(
                #[doc = #object_type_enum_variants_docs ]
                #object_type_enum_variants
            ),*
        }

        impl WlObjectType {
            #[doc = "Constructs [`WlObjectType`] from its name"]
            pub fn from_interface_name(name: &str) -> ::std::option::Option<Self> {
                static MAP: ::phf::Map<&'static str, WlObjectType> = ::phf::phf_map! {
                    #( #phf_map_entries )*
                };

                MAP.get(name).copied()
            }

            #[doc = "The [`Interface`](wayland_sys::Interface) generated by wayland-scanner for this object type"]
            pub const fn interface(self) -> &'static ::wayland_sys::Interface<'static> {
                match self {
                    #( #interface_map_entries )*
                }
            }

            #[doc = "The [`wl_interface`](wayland_sys::wl_interface) generated by wayland-scanner for this object type"]
            pub const fn backend_interface(self) -> &'static ::wayland_sys::wl_interface {
                match self {
                    #( #wl_interface_map_entries )*
                }
            }
        }

        #( #protocol_modules )*

        #[doc = "Shorthands for all interfaces data"]
        pub mod prelude {
            pub use super::{ #( #pub_uses ),* };
        }
    }
}

fn interface_to_module(interface: &Interface) -> TokenStream {
    let docs = format_doc_string(DocDescription::from_outer(interface.description.as_ref()));
    let request_docs = format!("Requests for {}", interface.name);
    let event_docs = format!("Events for {}", interface.name);
    let wl_enum_docs = format!("Enums for {}", interface.name);

    let module_name = Ident::new(strip_interface_name(&interface.name), Span::call_site());

    let requests = interface
        .entries
        .iter()
        .filter_map(|e| match e {
            InterfaceEntry::Request(r) => Some(r),
            _ => None,
        })
        .enumerate()
        .map(|(i, r)| request_to_impl(interface, r, i));

    let events = interface
        .entries
        .iter()
        .filter_map(|e| match e {
            InterfaceEntry::Event(r) => Some(r),
            _ => None,
        })
        .enumerate()
        .map(|(i, e)| event_to_impl(interface, e, i));

    let composite_event_data = interface
        .entries
        .iter()
        .filter_map(|e| match e {
            InterfaceEntry::Event(e) => Some(e),
            _ => None,
        })
        .collect::<Vec<_>>();

    let composite_event = impl_composite_event(interface, &composite_event_data);

    let enums = interface
        .entries
        .iter()
        .filter_map(|e| match e {
            InterfaceEntry::Enum(r) => Some(r),
            _ => None,
        })
        .map(enum_to_impl);

    quote! {
        #[doc = #docs ]
        pub mod #module_name {
            #[doc = #request_docs ]
            pub mod request {
                #( #requests )*
            }

            #[doc = #event_docs ]
            pub mod event {
                #composite_event

                #( #events )*
            }

            #[doc = #wl_enum_docs ]
            pub mod wl_enum {
                #( #enums )*
            }
        }
    }
}

pub fn strip_interface_name(name: &str) -> &str {
    if name.starts_with("wl_") {
        unsafe { name.get_unchecked(3..) }
    } else if name.starts_with("zwlr_") && name.ends_with("_v1") {
        unsafe { name.get_unchecked(5..name.len() - 3) }
    } else {
        name
    }
}

pub fn strip_v1_suffix(name: &str) -> &str {
    if name.ends_with("_v1") {
        unsafe { name.get_unchecked(..name.len() - 3) }
    } else {
        name
    }
}

fn enum_reference_to_path(name: &str) -> TokenStream {
    if let Some((interface_name, enum_name)) = name.split_once('.') {
        let interface_module_ident =
            Ident::new(strip_interface_name(interface_name), Span::call_site());

        let enum_name_pascal = enum_name.to_case(Case::Pascal);
        let enum_ident = Ident::new(&enum_name_pascal, Span::call_site());

        quote! { super::super:: #interface_module_ident ::wl_enum:: #enum_ident }
    } else {
        let enum_name_pascal = name.to_case(Case::Pascal);
        let enum_ident = Ident::new(&enum_name_pascal, Span::call_site());

        quote! { super::wl_enum:: #enum_ident }
    }
}

fn array_element_type_name(interface_name: &str, message_name: &str) -> TokenStream {
    match (interface_name, message_name) {
        // scancodes for pressed keys
        ("wl_keyboard", "enter") => quote! { u32 },
        ("xdg_toplevel", "configure") => quote! { super::super::xdg_toplevel::wl_enum::State },
        ("xdg_toplevel", "wm_capabilities") => {
            quote! { super::super::xdg_toplevel::wl_enum::WmCapabilities }
        }
        _ => panic!("unknown array type in {interface_name}.{message_name}"),
    }
}

fn request_to_impl(interface: &Interface, request: &Message, index: usize) -> TokenStream {
    let docs = format_doc_string(DocDescription::from_outer(request.description.as_ref()));

    let interface_name = strip_interface_name(&interface.name);
    let interface_name_pascal_case = interface_name.to_case(Case::Pascal);
    let interface_name_ident = Ident::new(&interface_name_pascal_case, Span::call_site());

    let request_name_pascal_case = request.name.to_case(Case::Pascal);
    let request_struct_name = Ident::new(&request_name_pascal_case, Span::call_site());

    let index_string = index.to_string();
    let opcode_literal = LitInt::new(&index_string, Span::call_site());

    let has_lifetime = request
        .arg
        .iter()
        .any(|argument| matches!(argument.ty, ArgType::String | ArgType::Fd | ArgType::Array));

    let struct_elided_lifetime = has_lifetime.then(|| quote! { <'_> });
    let struct_lifetime = has_lifetime.then(|| quote! { <'s> });

    let struct_fields = request
        .arg
        .iter()
        .filter_map(|argument| {
            let field_name = Ident::new(&argument.name, Span::call_site());
            let field_type = match argument.ty {
                ArgType::Int => match &argument.enumeration {
                    Some(name) => enum_reference_to_path(name),
                    None => quote! { i32 },
                },
                ArgType::Uint => match &argument.enumeration {
                    Some(name) => enum_reference_to_path(name),
                    None => quote! { u32 },
                },
                ArgType::NewId => return None,
                ArgType::Object => {
                    if argument.allow_null {
                        quote! { ::std::option::Option<crate::object::WlObjectId> }
                    } else {
                        quote! { crate::object::WlObjectId }
                    }
                }
                ArgType::String => quote! { &'s ::std::ffi::CStr },
                ArgType::Fd => quote! { ::std::os::fd::BorrowedFd<'s> },
                ArgType::Fixed => quote! { wayland_sys::WlFixed },
                ArgType::Array => unimplemented!("array usage in reqeusts"),
            };

            let docs = format_doc_string(DocDescription::from_inner(argument.summary.as_deref()));

            Some(quote! {
                #[doc = #docs ]
                pub #field_name : #field_type
            })
        })
        .collect::<Vec<_>>();

    let struct_body = if struct_fields.is_empty() {
        quote! { ; }
    } else {
        quote! {
            { #( #struct_fields ),* }
        }
    };

    let request_builder_arguments = request.arg.iter().map(|argument| {
        let method = Ident::new(argument.ty.builder_str(), Span::call_site());
        let argument_name = Ident::new(&argument.name, Span::call_site());

        let method_arg = match argument.ty {
            ArgType::Object => {
                if !argument.allow_null {
                    quote! {
                        ::std::option::Option::Some(
                            storage.get_proxy(self. #argument_name ).unwrap()
                        )
                    }
                } else {
                    quote! {
                        self. #argument_name .map(|id| storage.get_proxy(id).unwrap())
                    }
                }
            }
            ArgType::NewId => quote! {},
            ArgType::Uint if argument.enumeration.is_some() => {
                quote! { self. #argument_name .into() }
            }
            ArgType::Int if argument.enumeration.is_some() => {
                quote! { u32::from(self. #argument_name ) as i32 }
            }
            ArgType::Int | ArgType::Uint | ArgType::String | ArgType::Fd | ArgType::Fixed => {
                quote! { self. #argument_name }
            }
            ArgType::Array => unimplemented!("array usage in requests"),
        };

        quote! { . #method ( #method_arg ) }
    });

    let outgoing_interface_names = request
        .arg
        .iter()
        .filter_map(|argument| {
            if let (ArgType::NewId, Some(name)) = (argument.ty, &argument.interface) {
                Some(strip_interface_name(name).to_case(Case::Pascal))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    assert!(outgoing_interface_names.len() <= 1);

    let outgoing_interface = outgoing_interface_names.first().map(String::deref);

    let object_parent_impl = outgoing_interface
        .map(|name| {
            let name = Ident::new(name, Span::call_site());

            quote! {
                impl crate::interface::ObjectParent for
                    #request_struct_name #struct_elided_lifetime
                {
                    const CHILD_TYPE: super::super::super::WlObjectType
                         = super::super::super::WlObjectType:: #name ;
                }
            }
        })
        .into_iter();

    let outgoing_interface_value = match outgoing_interface {
        Some(name) => {
            let name = Ident::new(name, Span::call_site());
            quote! {
                ::std::option::Option::Some(
                    super::super::super::WlObjectType:: #name
                )
            }
        }
        None => quote! { ::std::option::Option::None },
    };

    let derive_call = derive_call_from_args(&request.arg);

    quote! {
        #derive_call
        #[doc = #docs ]
        pub struct #request_struct_name #struct_lifetime #struct_body

        #( #object_parent_impl )*

        impl crate::object::HasObjectType for
            #request_struct_name #struct_elided_lifetime
        {
            const OBJECT_TYPE: super::super::super::WlObjectType
                 = super::super::super::WlObjectType:: #interface_name_ident;
        }

        impl<'s> crate::interface::Request<'s> for #request_struct_name #struct_lifetime {
            const CODE: crate::sys::wire::OpCode = #opcode_literal ;
            const CHILD_TYPE: ::std::option::Option<super::super::super::WlObjectType>
                = #outgoing_interface_value ;

            fn build_message<'m, S: crate::sys::object::dispatch::State>(
                self,
                buf: &'m mut impl crate::sys::wire::WlMessageBuffer,
                #[allow(dead_code)]
                storage: &'m crate::sys::object_storage::WlObjectStorage<S>,
            ) -> crate::sys::wire::WlMessage<'m>
            where
                's: 'm,
            {
                crate::sys::wire::WlMessage::builder(buf)
                    .opcode(Self::CODE)
                    #( #request_builder_arguments )*
                    .build()
            }
        }
    }
}

fn derive_call_from_args(args: &[Arg<'_>]) -> TokenStream {
    let has_fd_in_args = args.iter().any(|arg| matches!(arg.ty, ArgType::Fd));

    if has_fd_in_args {
        quote! { #[derive(Clone, Debug)] }
    } else {
        quote! { #[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)] }
    }
}

fn impl_composite_event(interface: &Interface, events: &[&Message<'_>]) -> TokenStream {
    if events.is_empty() {
        return TokenStream::new();
    }

    let docs = format!("All events for {} interface", interface.name);

    let event_names = events
        .iter()
        .map(|event| event.name.to_case(Case::Pascal))
        .collect::<Vec<_>>();

    let event_idents = event_names
        .iter()
        .map(|name| Ident::new(name, Span::call_site()))
        .collect::<Vec<_>>();

    let event_lifetimes = events
        .iter()
        .map(|event| event.has_lifetime().then(|| quote! { <'s> }))
        .collect::<Vec<_>>();

    let has_lifetime = events.iter().any(|event| event.has_lifetime());
    let lifetime = has_lifetime.then(|| quote! { <'s> });

    quote! {
        #[doc = #docs ]
        #[derive(Clone, Debug)]
        pub enum Event #lifetime {
            #(
                #event_idents ( #event_idents #event_lifetimes ) ,
            )*
        }

        impl #lifetime Event #lifetime {
            /// [`OpCode`](crate::sys::wire::OpCode) of this [`Event`]
            pub const fn code(&self) -> crate::sys::wire::OpCode {
                match self {
                    #(
                        Self:: #event_idents (..) => < #event_idents as crate::interface::Event>::CODE,
                    )*
                }
            }
        }

        #(
            impl #lifetime ::core::convert::From< #event_idents #event_lifetimes > for Event #lifetime {
                fn from(value: #event_idents #event_lifetimes) -> Self {
                    Self:: #event_idents (value)
                }
            }
        )*

        impl<'s> crate::interface::Event<'s> for Event #lifetime {
            const CODE: crate::sys::wire::OpCode = crate::sys::wire::OpCode::MAX ;

            fn from_message(message: crate::sys::wire::WlMessage<'s>)
                -> ::std::option::Option<Self>
            {
                Some(match message.opcode {
                    #(
                        < #event_idents as crate::interface::Event>::CODE => Self::from(
                            < #event_idents as crate::interface::Event>::from_message(message)?,
                        ),
                    )*
                    _ => return None,
                })
            }
        }
    }
}

fn event_to_impl(interface: &Interface, event: &Message, index: usize) -> TokenStream {
    let docs = format_doc_string(DocDescription::from_outer(event.description.as_ref()));

    let event_name_pascal = event.name.to_case(Case::Pascal);
    let event_ident = Ident::new(&event_name_pascal, Span::call_site());

    let event_lifetime = event.has_lifetime().then(|| quote! { <'s> });

    let argument_type = |argument: &Arg<'_>, add_lifetime: bool| -> Option<TokenStream> {
        let lifetime = add_lifetime.then(|| quote! { 's });
        let angle_braced_lifetime = add_lifetime.then(|| quote! { <'s> });

        Some(match argument.ty {
            ArgType::Int => match &argument.enumeration {
                Some(name) => enum_reference_to_path(name),
                None => quote! { i32 },
            },
            ArgType::Uint => match &argument.enumeration {
                Some(name) => enum_reference_to_path(name),
                None => quote! { u32 },
            },
            ArgType::NewId => return None,
            ArgType::Object => quote! { crate::sys::proxy::WlProxyQuery },
            ArgType::String => quote! { & #lifetime ::std::ffi::CStr },
            ArgType::Fd => quote! { ::std::os::fd::BorrowedFd #angle_braced_lifetime },
            ArgType::Fixed => quote! { ::wayland_sys::WlFixed },
            ArgType::Array => {
                let array_element_type = array_element_type_name(&interface.name, &event.name);
                quote! { & #lifetime [ #array_element_type ] }
            }
        })
    };

    let event_body_arguments = event
        .arg
        .iter()
        .filter_map(|argument| {
            let field_name = Ident::new(&argument.name, Span::call_site());
            let field_type = argument_type(argument, true)?;

            let docs = format_doc_string(DocDescription::from_inner(argument.summary.as_deref()));

            Some(quote! {
                #[doc = #docs ]
                pub #field_name : #field_type
            })
        })
        .collect::<Vec<_>>();

    let event_body = if event_body_arguments.is_empty() {
        quote! { ; }
    } else {
        quote! { { #( #event_body_arguments ),* } }
    };

    let opcode_string = index.to_string();
    let opcode_literal = LitInt::new(&opcode_string, Span::call_site());

    let read_statements = event.arg.iter().filter_map(|argument| {
        let argument_ident = Ident::new(&argument.name, Span::call_site());
        let argument_type = argument_type(argument, false)?;

        Some(if argument.enumeration.is_none() {
            quote! {
                let #argument_ident = unsafe { reader.read::< #argument_type >()? };
            }
        } else {
            quote! {
                let #argument_ident = unsafe {
                    #argument_type ::from_u32_unchecked(reader.read::<u32>()?)
                };
            }
        })
    });

    let field_names = event.arg.iter().filter_map(|argument| {
        if matches!(argument.ty, ArgType::NewId) {
            return None;
        }

        Some(Ident::new(&argument.name, Span::call_site()))
    });

    let derive_call = derive_call_from_args(&event.arg);

    quote! {
        #derive_call
        #[doc = #docs ]
        pub struct #event_ident #event_lifetime #event_body

        impl<'s> crate::interface::Event<'s> for #event_ident #event_lifetime {
            const CODE: crate::sys::wire::OpCode = #opcode_literal ;

            fn from_message(message: crate::sys::wire::WlMessage<'s>)
                -> ::std::option::Option<Self>
            {
                if message.opcode != Self::CODE {
                    return None;
                }

                let mut reader = message.reader();

                #( #read_statements )*

                Some(Self { #( #field_names ),* })
            }
        }
    }
}

fn enum_to_impl(enumeration: &Enum) -> TokenStream {
    if enumeration.is_bitfield {
        bitfield_enum_to_impl(enumeration)
    } else {
        regular_enum_to_impl(enumeration)
    }
}

fn bitfield_enum_to_impl(enumeration: &Enum) -> TokenStream {
    let docs = format_doc_string(DocDescription::from_outer(enumeration.description.as_ref()));

    let enum_name = enumeration.name.to_case(Case::Pascal);
    let enum_ident = Ident::new(&enum_name, Span::call_site());

    let from_u32_unchecked_docs =
        format!("Constructs [`{enum_name}`] from u32 as is\n\n# Safety\n\n`value` should be valid");

    let enum_entry_names = enumeration
        .entry
        .iter()
        .map(|entry| {
            let is_number = entry.name.chars().all(|c| c.is_ascii_digit());
            let screaming_snake = entry.name.to_case(Case::UpperSnake);

            if is_number {
                format!("_{screaming_snake}")
            } else {
                screaming_snake
            }
        })
        .collect::<Vec<_>>();

    let entries = enum_entry_names
        .iter()
        .zip(&enumeration.entry)
        .map(|(name, entry)| {
            let entry_ident = Ident::new(name, Span::call_site());
            let entry_value_string = entry.value.to_string();
            let entry_value_literal = LitInt::new(&entry_value_string, Span::call_site());

            let docs = format_doc_string(DocDescription::from_inner_and_outer(
                entry.summary.as_deref(),
                entry.description.as_ref(),
            ));

            quote! {
                #[doc = #docs ]
                const #entry_ident = #entry_value_literal;
            }
        });

    quote! {
        ::bitflags::bitflags! {
            #[doc = #docs ]
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
            pub struct #enum_ident : u32 {
                #( #entries )*
            }
        }

        impl #enum_ident {
            #[doc = #from_u32_unchecked_docs ]
            pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                unsafe { ::std::mem::transmute::<u32, Self>(value) }
            }
        }

        impl ::std::convert::From< #enum_ident > for u32 {
            fn from(value: #enum_ident ) -> Self {
                value.bits()
            }
        }
    }
}

fn regular_enum_to_impl(enumeration: &Enum) -> TokenStream {
    let docs = format_doc_string(DocDescription::from_outer(enumeration.description.as_ref()));

    let enum_name = enumeration.name.to_case(Case::Pascal);
    let enum_ident = Ident::new(&enum_name, Span::call_site());

    let from_u32_unchecked_docs =
        format!("Constructs [`{enum_name}`] from u32 as is\n\n# Safety\n\n`value` should be valid");

    let try_from_error_docs = format!("Error constructing [`{enum_name}`] from u32");

    let enum_entry_names = enumeration
        .entry
        .iter()
        .map(|entry| {
            let is_number = entry.name.chars().all(|c| c.is_ascii_digit());
            let pascal = entry.name.to_case(Case::Pascal);

            if is_number {
                format!("_{pascal}")
            } else {
                pascal
            }
        })
        .collect::<Vec<_>>();

    let entries = enum_entry_names
        .iter()
        .zip(&enumeration.entry)
        .map(|(name, entry)| {
            let entry_ident = Ident::new(name, Span::call_site());
            let entry_value_literal = u32::from(entry.value);

            let docs = format_doc_string(DocDescription::from_inner_and_outer(
                entry.summary.as_deref(),
                entry.description.as_ref(),
            ));

            quote! {
                #[doc = #docs ]
                #entry_ident = #entry_value_literal
            }
        });

    let try_from_error_ident = format_ident!("{}FromU32Error", enum_ident);

    let try_from_match_entries =
        enum_entry_names
            .iter()
            .zip(&enumeration.entry)
            .map(|(name, entry)| {
                let entry_ident = Ident::new(name, Span::call_site());
                let entry_value_literal = u32::from(entry.value);

                quote! {
                    #entry_value_literal => Self:: #entry_ident,
                }
            });

    let try_from_error_string = format!("failed to convert {{0}} to {enum_name}");

    quote! {
        #[doc = #docs ]
        #[repr(u32)]
        #[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
        pub enum #enum_ident {
            #( #entries ),*
        }

        impl #enum_ident {
            #[doc = #from_u32_unchecked_docs ]
            pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                unsafe { ::std::mem::transmute::<u32, Self>(value) }
            }
        }

        impl ::std::convert::From< #enum_ident > for u32 {
            fn from(value: #enum_ident ) -> Self {
                value as u32
            }
        }

        #[doc = #try_from_error_docs ]
        #[derive(Debug, ::thiserror::Error)]
        #[error( #try_from_error_string )]
        pub struct #try_from_error_ident (pub u32);

        impl ::std::convert::TryFrom<u32> for #enum_ident {
            type Error = #try_from_error_ident ;

            fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                Ok(match value {
                    #( #try_from_match_entries )*
                    _ => return Err( #try_from_error_ident (value)),
                })
            }
        }
    }
}
