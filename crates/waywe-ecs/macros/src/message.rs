use crate::uuid::{generate_uuid, quote_uuid};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericParam, Path, parse_macro_input, parse_quote};

pub fn derive_message(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let waywe_ecs_path: Path = crate::waywe_ecs_path();

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let type_parameters = ast
        .generics
        .params
        .iter()
        .flat_map(|param| match param {
            GenericParam::Type(type_param) => Some(type_param.ident.clone()),
            GenericParam::Lifetime(_) | GenericParam::Const(_) => None,
        })
        .collect::<Vec<_>>();

    let uuid = generate_uuid();
    let uuid_bytes = quote_uuid(&uuid);

    TokenStream::from(quote! {
        impl #impl_generics #waywe_ecs_path::uuid::TypeUuid
            for #struct_name #type_generics #where_clause
        {
            fn uuid() -> [u8; 16] {
                #waywe_ecs_path::uuid::UuidBuilder::new(
                    #waywe_ecs_path::uuid::Uuid::from_bytes(
                        #uuid_bytes
                    )
                )
                    #(
                        .add::<#type_parameters>()
                    )*
                    .build()
                    .into_bytes()
            }
        }

        impl #impl_generics #waywe_ecs_path::message::Message
            for #struct_name #type_generics #where_clause
        {}
    })
}
