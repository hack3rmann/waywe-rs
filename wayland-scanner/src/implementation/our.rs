use super::protocol_from_str;
use crate::xml::{ArgType, Enum, Interface, InterfaceEntry, Message};
use convert_case::{Case, Casing as _};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::{fs, ops::Deref, path::PathBuf};
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
            let path = PathBuf::from(str_lit.value());

            fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read file '{}': {err}", path.display()))
        })
        .collect::<Vec<_>>();

    let protocols = file_contents
        .iter()
        .map(|contents| protocol_from_str(contents).expect("failed to parse protocol"))
        .collect::<Vec<_>>();

    let object_type_enum_variants_names = protocols
        .iter()
        .flat_map(|protocol| {
            protocol
                .interface
                .iter()
                .map(|interface| interface.name.as_ref().to_case(Case::Pascal))
        })
        .collect::<Vec<_>>();

    let object_type_enum_variants = object_type_enum_variants_names
        .iter()
        .map(|name| Ident::new(name, Span::call_site()));

    let protocol_modules = protocols.iter().map(|protocol| {
        let protocol_module_name = Ident::new(&protocol.name, Span::call_site());
        let modules = protocol.interface.iter().map(interface_to_module);

        quote! {
            pub mod #protocol_module_name {
                #( #modules )*
            }
        }
    });

    quote! {
        #[derive(Debug, PartialEq, Default, Clone, Copy, Eq, PartialOrd, Ord, Hash)]
        pub enum WlObjectType {
            #[default]
            #( #object_type_enum_variants ),*
        }

        #( #protocol_modules )*
    }
}

fn interface_to_module(interface: &Interface) -> TokenStream {
    let module_name = Ident::new(&interface.name, Span::call_site());

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
        .map(event_to_impl);

    let enums = interface
        .entries
        .iter()
        .filter_map(|e| match e {
            InterfaceEntry::Enum(r) => Some(r),
            _ => None,
        })
        .map(enum_to_impl);

    quote! {
        pub mod #module_name {
            pub mod request {
                #( #requests )*
            }

            pub mod event {
                #( #events )*
            }

            pub mod wl_enum {
                #( #enums )*
            }
        }
    }
}

fn request_to_impl(interface: &Interface, request: &Message, index: usize) -> TokenStream {
    let interface_name_pascal_case = interface.name.to_case(Case::Pascal);
    let interface_name_ident = Ident::new(&interface_name_pascal_case, Span::call_site());

    let request_name_pascal_case = request.name.to_case(Case::Pascal);
    let request_struct_name = Ident::new(&request_name_pascal_case, Span::call_site());

    let index_string = index.to_string();
    let opcode_literal = LitInt::new(&index_string, Span::call_site());

    let has_lifetime = request
        .arg
        .iter()
        .any(|argument| matches!(argument.ty, ArgType::String | ArgType::Fd | ArgType::Array));

    let struct_lifetime_decl = has_lifetime.then(|| quote! { <'s> }).into_iter();
    let struct_elided_lifetime_has_type = has_lifetime.then(|| quote! { <'_> }).into_iter();
    let struct_elided_lifetime_object_parent = has_lifetime.then(|| quote! { <'_> }).into_iter();
    let struct_lifetime_impl = struct_lifetime_decl.clone();

    let struct_fields = request.arg.iter().filter_map(|argument| {
        let field_name = Ident::new(&argument.name, Span::call_site());
        let field_type = match argument.ty {
            ArgType::Int => quote! { i32 },
            ArgType::Uint => quote! { u32 },
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
            ArgType::Array => unimplemented!(),
        };

        Some(quote! { #field_name : #field_type })
    });

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
            ArgType::Int | ArgType::Uint | ArgType::String | ArgType::Fd | ArgType::Fixed => {
                quote! { self. #argument_name }
            }
            ArgType::Array => unimplemented!(),
        };

        quote! { . #method ( #method_arg ) }
    });

    let outgoing_interface_names = request
        .arg
        .iter()
        .filter_map(|argument| {
            if let (ArgType::NewId, Some(name)) = (argument.ty, &argument.interface) {
                Some(name.to_case(Case::Pascal))
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
                    #request_struct_name #( #struct_elided_lifetime_object_parent )*
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

    quote! {
        // TODO: derive anything
        pub struct #request_struct_name #( #struct_lifetime_decl )* {
            #( #struct_fields ),*
        }

        #( #object_parent_impl )*

        impl crate::object::HasObjectType for
            #request_struct_name #( #struct_elided_lifetime_has_type )*
        {
            const OBJECT_TYPE: super::super::super::WlObjectType
                 = super::super::super::WlObjectType:: #interface_name_ident;
        }

        impl<'s> crate::interface::Request<'s> for #request_struct_name #( #struct_lifetime_impl )* {
            const CODE: crate::sys::wire::OpCode = #opcode_literal ;
            const OUTGOING_INTERFACE: ::std::option::Option<super::super::super::WlObjectType>
                = #outgoing_interface_value ;

            fn build_message<'m, S: crate::sys::object::dispatch::State>(
                self,
                buf: &'m mut impl crate::sys::wire::MessageBuffer,
                storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
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

fn event_to_impl(_event: &Message) -> TokenStream {
    quote! {}
}

fn enum_to_impl(_enum: &Enum) -> TokenStream {
    quote! {}
}
