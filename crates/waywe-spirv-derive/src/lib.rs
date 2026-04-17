use proc_macro2::{Ident, Punct};
use quote::quote;
use shaderc::{CompileOptions, Compiler, ShaderKind};
use std::fs;
use syn::{
    Attribute, DeriveInput, LitStr, Meta, Token, parse::*, punctuated::Punctuated, token::Comma,
};

fn compile_spirv(attr: &ShaderAttribute) -> Vec<u32> {
    let source = fs::read_to_string(&attr.path).unwrap();

    let compiler = Compiler::new().unwrap();
    let options = CompileOptions::new().unwrap();

    // TODO(Lorent1): add not main
    compiler
        .compile_into_spirv(
            &source,
            attr.stage.to_shader_kind(),
            &attr.path,
            "main",
            Some(&options),
        )
        .unwrap()
        .as_binary()
        .to_vec()
}

#[derive(Debug, Clone, Copy)]
enum Stage {
    Vertex,
    Fragment,
    Compute,
}

impl Stage {
    pub fn to_shader_kind(self) -> ShaderKind {
        match self {
            Self::Vertex => ShaderKind::Vertex,
            Self::Fragment => ShaderKind::Fragment,
            Self::Compute => ShaderKind::Compute,
        }
    }
}

#[derive(Debug)]
struct ShaderAttribute {
    path: String,
    stage: Stage,
}

struct GenericGroupList {
    generic_groups: Punctuated<GenericGroup, Comma>,
}

impl Parse for GenericGroupList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            generic_groups: input.parse_terminated(<GenericGroup as Parse>::parse, Token![,])?,
        })
    }
}

enum AttrName {
    Stage,
    Path,
}

struct GenericGroup {
    ident: AttrName,
    literal: LitStr,
}

impl Parse for GenericGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let punct = input.parse::<Punct>()?;
        let literal = input.parse::<LitStr>()?;

        let ident = match ident.to_string().as_str() {
            "stage" => AttrName::Stage,
            "path" => AttrName::Path,
            _ => return Err(Error::new(ident.span(), "expected 'stage' or 'path'")),
        };

        if punct.as_char() != '=' {
            return Err(Error::new(punct.span(), "expected '='"));
        }

        Ok(Self { ident, literal })
    }
}

fn parse_attributes(input: &[Attribute]) -> ShaderAttribute {
    let mut path = None;
    let mut stage = None;

    for attr in input {
        match &attr.meta {
            Meta::List(list) => {
                // panic!("{list:#?}");
                let segments = &list.path.segments;

                if segments.len() != 1 {
                    continue;
                }

                if segments[0].ident != "shader" {
                    continue;
                }

                let list = syn::parse2::<GenericGroupList>(list.tokens.clone()).unwrap();

                for group in list.generic_groups {
                    match group.ident {
                        AttrName::Stage => {
                            stage = match group.literal.value().as_str() {
                                "vertex" => Some(Stage::Vertex),
                                "fragment" => Some(Stage::Fragment),
                                "compute" => Some(Stage::Compute),
                                _ => None,
                            };
                        }
                        AttrName::Path => {
                            path = Some(group.literal.value());
                        }
                    }
                }
            }
            _ => continue,
        }
    }

    let Some(path) = path else {
        panic!("no path provided")
    };

    let Some(stage) = stage else {
        panic!("no stage provided")
    };

    ShaderAttribute { path, stage }
}

#[proc_macro_derive(ShaderDescriptor, attributes(shader))]
pub fn spirv_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse2::<DeriveInput>(input.into()).unwrap();

    let struct_name = &ast.ident;
    let attr = parse_attributes(&ast.attrs);
    let u32_vec = compile_spirv(&attr);

    quote! {
        impl #struct_name {
            const __SPRIV_SOURCE: &[u32] = &[ #( #u32_vec ),* ];
        }
    }
    .into()
}
