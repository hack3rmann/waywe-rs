extern crate proc_macro;

use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream as TokenStream2};
use quote::quote;
use std::time::SystemTime;
use syn::{parse::*, punctuated::Punctuated, token::Comma, Expr, ExprLit, Lit, *};
use uuid::{Builder as UuidBuilder, Uuid};

#[derive(Debug, Clone)]
struct UuidAttribute {
    uuid: Option<String>,
    bounds: Vec<UuidGenericBound>,
}

#[derive(Clone, Copy, Default, Debug)]
enum UuidReboundType {
    #[default]
    TypeUuid,
    TypeId,
}

#[derive(Debug, Clone)]
struct UuidGenericBound {
    ident: Ident,
    rebound_type: UuidReboundType,
}

impl Parse for UuidGenericBound {
    fn parse(input: ParseStream) -> Result<Self> {
        let rebound_ident = input.parse::<Ident>()?;

        if rebound_ident != "rebound" {
            return Err(Error::new(
                rebound_ident.span(),
                "expected 'rebound' identifier",
            ));
        }

        let group = input.parse::<Group>()?;

        if group.delimiter() != Delimiter::Parenthesis {
            return Err(Error::new(group.span(), "expected parenthesis"));
        }

        let inner = syn::parse2::<UuidGenericBoundInnerGroup>(group.stream())?;

        Ok(inner.into())
    }
}

struct UuidGenericBoundInnerGroup {
    ident: Ident,
    rebound_type: UuidReboundType,
}

impl Parse for UuidGenericBoundInnerGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let colon = input.parse::<Punct>()?;

        if colon.as_char() != ':' || colon.spacing() != Spacing::Alone {
            return Err(Error::new(colon.span(), "expected ':' alone"));
        }

        let rebound_type_ident = input.parse::<Ident>()?;

        let rebound_type = match rebound_type_ident.to_string().as_str() {
            "TypeId" => UuidReboundType::TypeId,
            "TypeUuid" => UuidReboundType::TypeUuid,
            _ => {
                return Err(Error::new(
                    rebound_type_ident.span(),
                    "expected 'TypeId' or 'TypeUuid'",
                ));
            }
        };

        Ok(Self {
            ident,
            rebound_type,
        })
    }
}

impl From<UuidGenericBoundInnerGroup> for UuidGenericBound {
    fn from(value: UuidGenericBoundInnerGroup) -> Self {
        Self {
            ident: value.ident,
            rebound_type: value.rebound_type,
        }
    }
}

fn parse_uuid_attributes<'a>(attributes: impl IntoIterator<Item = &'a Attribute>) -> UuidAttribute {
    let uuid_attributes = attributes
        .into_iter()
        .filter(|attr| attr.meta.path().is_ident("uuid"))
        .collect::<Vec<_>>();

    let uuid = uuid_attributes.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            }),
            ..
        }) => Some(lit.value()),
        _ => None,
    });

    let bounds = uuid_attributes
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::List(MetaList {
                delimiter: MacroDelimiter::Paren(_),
                tokens,
                ..
            }) => syn::parse2::<UuidGenericBound>(tokens.clone()).ok(),
            _ => None,
        })
        .collect::<Vec<_>>();

    UuidAttribute { uuid, bounds }
}

fn make_type_params(
    uuid_info: &UuidAttribute,
    params: &Punctuated<GenericParam, Comma>,
) -> Vec<TokenStream2> {
    params
        .iter()
        .flat_map(|param| match param {
            GenericParam::Type(type_param) => Some(type_param.ident.clone()),
            GenericParam::Lifetime(_) | GenericParam::Const(_) => None,
        })
        .map(
            |ident| match uuid_info.bounds.iter().find(|bound| bound.ident == ident) {
                Some(UuidGenericBound {
                    rebound_type: UuidReboundType::TypeId,
                    ..
                }) => quote! { .add_from_type_id::<#ident>() },
                None
                | Some(UuidGenericBound {
                    rebound_type: UuidReboundType::TypeUuid,
                    ..
                }) => quote! { .add::<#ident>() },
            },
        )
        .collect::<Vec<_>>()
}

fn generate_uuid() -> Uuid {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("unix epoch was a long time ago");

    let millis = ts.as_millis() as u64;
    let mut random_bytes = [0; 10];
    rand::fill(&mut random_bytes);

    UuidBuilder::from_unix_timestamp_millis(millis, &random_bytes).into_uuid()
}

fn quote_uuid(uuid: &Uuid) -> TokenStream2 {
    let bytes = uuid.as_bytes();

    quote! { [ #( #bytes ),* ] }
}

#[proc_macro_derive(TypeUuid, attributes(uuid))]
pub fn waywe_uuid_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse::<DeriveInput>(input).unwrap();

    let mut uuid_info = parse_uuid_attributes(&ast.attrs);

    let uuid = uuid_info
        .uuid
        .take()
        .and_then(|uuid_str| Uuid::parse_str(&uuid_str).ok())
        .unwrap_or_else(generate_uuid);

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();
    let uuid_bytes = quote_uuid(&uuid);
    let type_parameters = make_type_params(&uuid_info, &ast.generics.params);

    let gen = quote! {
        impl #impl_generics waywe_uuid::TypeUuid
            for #struct_name #type_generics #where_clause
        {
            fn uuid() -> [u8; 16] {
                waywe_uuid::UuidBuilder::new(
                    waywe_uuid::Uuid::from_bytes(
                        #uuid_bytes
                    )
                )
                    #( #type_parameters )*
                    .build()
                    .into_bytes()
            }
        }
    };
    gen.into()
}

struct ExternalDeriveInput {
    path: ExprPath,
    uuid_str: LitStr,
}

impl Parse for ExternalDeriveInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        input.parse::<Token![,]>()?;
        let uuid_str = input.parse()?;
        Ok(Self { path, uuid_str })
    }
}

#[proc_macro]
pub fn external_type_uuid(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ExternalDeriveInput { path, uuid_str } = parse_macro_input!(tokens as ExternalDeriveInput);

    let uuid = Uuid::parse_str(&uuid_str.value()).expect("Value was not a valid UUID");

    let bytes = uuid
        .as_bytes()
        .iter()
        .map(|byte| format!("{:#X}", byte))
        .map(|byte_str| syn::parse_str::<LitInt>(&byte_str).unwrap());

    let gen = quote! {
        impl crate::ConstTypeUuid for #path {
            const UUID: crate::Bytes = [
                #( #bytes ),*
            ];
        }
    };
    gen.into()
}
