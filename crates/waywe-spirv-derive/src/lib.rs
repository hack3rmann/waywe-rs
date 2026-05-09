use proc_macro2::{Ident, Punct};
use quote::quote;
use std::{result::Result, str::FromStr};
use syn::{
    Attribute, DeriveInput, LitStr, Meta, Token,
    parse::{Result as ParseResult, *},
    punctuated::Punctuated,
    token::Comma,
};

#[derive(Debug, Clone, Copy)]
enum Stage {
    Vertex,
    Fragment,
    Compute,
}

impl FromStr for Stage {
    type Err = ParseFailed;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "vertex" => Self::Vertex,
            "fragment" => Self::Fragment,
            "compute" => Self::Compute,
            _ => return Err(ParseFailed),
        })
    }
}

#[derive(Debug)]
struct ParseFailed;

#[derive(Debug)]
struct ShaderAttribute {
    path: String,
    stage: Stage,
    label: Option<String>,
}

struct GenericGroupList {
    generic_groups: Punctuated<GenericGroup, Comma>,
}

impl Parse for GenericGroupList {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(Self {
            generic_groups: input.parse_terminated(<GenericGroup as Parse>::parse, Token![,])?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum AttrName {
    Stage,
    Path,
    Label,
}

impl FromStr for AttrName {
    type Err = ParseFailed;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "stage" => Self::Stage,
            "path" => Self::Path,
            "label" => Self::Label,
            _ => return Err(ParseFailed),
        })
    }
}

struct GenericGroup {
    ident: AttrName,
    literal: LitStr,
}

impl Parse for GenericGroup {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let ident = input.parse::<Ident>()?;
        let punct = input.parse::<Punct>()?;
        let literal = input.parse::<LitStr>()?;

        let ident = ident
            .to_string()
            .as_str()
            .parse::<AttrName>()
            .map_err(|_| Error::new(ident.span(), "expected 'stage' | 'path' | 'label'"))?;

        if punct.as_char() != '=' {
            return Err(Error::new(punct.span(), "expected '='"));
        }

        Ok(Self { ident, literal })
    }
}

fn parse_attributes(input: &[Attribute]) -> ShaderAttribute {
    let mut path = None;
    let mut stage = None;
    let mut label = None;

    for attr in input {
        if let Meta::List(list) = &attr.meta {
            let segments = &list.path.segments;

            if segments.len() != 1 || segments[0].ident != "shader" {
                continue;
            }

            let list = syn::parse2::<GenericGroupList>(list.tokens.clone()).unwrap();

            for group in list.generic_groups {
                match group.ident {
                    AttrName::Stage => {
                        stage = group.literal.value().as_str().parse().ok();
                    }
                    AttrName::Path => {
                        path = Some(group.literal.value());
                    }
                    AttrName::Label => {
                        label = Some(group.literal.value());
                    }
                }
            }
        } else {
            continue;
        }
    }

    let Some(path) = path else {
        panic!("no path provided")
    };

    let Some(stage) = stage else {
        panic!("no stage provided")
    };

    ShaderAttribute { path, stage, label }
}

#[proc_macro_derive(ShaderDescriptor, attributes(shader))]
pub fn spirv_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse2::<DeriveInput>(input.into()).unwrap();

    let struct_name = &ast.ident;
    let attr = parse_attributes(&ast.attrs);

    let shader_source = feature::shader_source(&attr);

    let label = match attr.label {
        Some(label) => quote! { Some(#label) },
        None => quote! { None },
    };

    quote! {
        impl ::waywe_runtime::shaders::ShaderDescriptor for #struct_name {
            fn shader_descriptor() -> ::wgpu::ShaderModuleDescriptor<'static> {
                ::wgpu::ShaderModuleDescriptor {
                    label: #label,
                    source: #shader_source,
                }
            }
        }
    }
    .into()
}

#[cfg(feature = "spirv")]
mod feature {
    use super::{ShaderAttribute, Stage};
    use proc_macro2::TokenStream;
    use quote::quote;
    use shaderc::{CompileOptions, Compiler, ShaderKind};
    use std::fs;

    fn compile_spirv(source: &str, kind: ShaderKind, file_name: &str) -> Vec<u32> {
        let compiler = Compiler::new().unwrap();
        let options = CompileOptions::new().unwrap();

        // TODO(Lorent1): add not main
        compiler
            .compile_into_spirv(source, kind, file_name, "main", Some(&options))
            .unwrap()
            .as_binary()
            .to_vec()
    }

    pub fn shader_source(attr: &ShaderAttribute) -> TokenStream {
        let glsl_source = fs::read_to_string(&attr.path).unwrap();
        let spirv_words = compile_spirv(&glsl_source, attr.stage.into(), &attr.path);

        quote! {
            ::wgpu::ShaderSource::SpirV(
                ::std::borrow::Cow::Borrowed(&[ #( #spirv_words ),* ]),
            )
        }
    }

    impl From<Stage> for ShaderKind {
        fn from(value: Stage) -> Self {
            match value {
                Stage::Vertex => Self::Vertex,
                Stage::Fragment => Self::Fragment,
                Stage::Compute => Self::Compute,
            }
        }
    }
}

#[cfg(not(feature = "spirv"))]
mod feature {
    use super::{ShaderAttribute, Stage};
    use proc_macro2::TokenStream;
    use quote::quote;
    use std::fs;

    pub fn shader_source(attr: &ShaderAttribute) -> TokenStream {
        let glsl_source = fs::read_to_string(&attr.path).unwrap();
        let stage = shader_stage(attr.stage);

        quote! {
            ::wgpu::ShaderSource::Glsl {
                shader: ::std::borrow::Cow::Borrowed(#glsl_source),
                stage: #stage,
                defines: ::std::default::Default::default(),
            }
        }
    }

    fn shader_stage(stage: Stage) -> TokenStream {
        match stage {
            Stage::Vertex => quote! { ::wgpu::naga::ShaderStage::Vertex },
            Stage::Fragment => quote! { ::wgpu::naga::ShaderStage::Fragment },
            Stage::Compute => quote! { ::wgpu::naga::ShaderStage::Compute },
        }
    }
}
