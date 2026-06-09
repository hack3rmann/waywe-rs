#![allow(unused)]

use proc_macro2::{Ident, Punct, Span};
use quote::quote;
use std::{env, path::Path, result::Result, str::FromStr};
use syn::{
    Attribute, DeriveInput, Error, LitStr, Meta, Token,
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
    path_span: Span,
    stage: Stage,
    stage_span: Span,
    label: Option<String>,
    label_span: Option<Span>,
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
        let Meta::List(list) = &attr.meta else {
            continue;
        };
        let segments = &list.path.segments;

        if segments.len() != 1 || segments[0].ident != "shader" {
            continue;
        }

        let list = syn::parse2::<GenericGroupList>(list.tokens.clone()).unwrap();

        for group in list.generic_groups {
            match group.ident {
                AttrName::Stage => {
                    stage = group
                        .literal
                        .value()
                        .as_str()
                        .parse::<Stage>()
                        .ok()
                        .map(|stage| (stage, group.literal.span()));
                }
                AttrName::Path => {
                    path = Some((group.literal.value(), group.literal.span()));
                }
                AttrName::Label => {
                    label = Some((group.literal.value(), group.literal.span()));
                }
            }
        }
    }

    let Some((path, path_span)) = path else {
        panic!("no path provided")
    };

    let Some((stage, stage_span)) = stage else {
        panic!("no stage provided")
    };

    let (label, label_span) = match label {
        Some((label, label_span)) => (Some(label), Some(label_span)),
        None => (None, None),
    };

    ShaderAttribute {
        path,
        path_span,
        stage,
        stage_span,
        label,
        label_span,
    }
}

#[proc_macro_derive(ShaderDescriptor, attributes(shader))]
pub fn spirv_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse2::<DeriveInput>(input.into()).unwrap();

    let struct_name = &ast.ident;
    let attr = parse_attributes(&ast.attrs);

    if !Path::new(&attr.path).exists() {
        let working_dir = env::current_dir().unwrap();

        return Error::new(
            attr.path_span,
            format!(
                "'{}' does not exist relative to '{}'",
                attr.path,
                working_dir.display()
            ),
        )
        .into_compile_error()
        .into();
    }

    let absolute_shader_path = {
        let path = Path::new(&attr.path).canonicalize().unwrap();
        path.to_string_lossy().into_owned()
    };

    let shader_source_const = quote! {
        // NOTE(hack3rmann): Rust will rebuild the current file when the GLSL source is changed
        const _: &str = include_str!( #absolute_shader_path )
    };

    let shader_source = match feature::shader_source(&attr) {
        Ok(source) => source,
        Err(err) => {
            let error = err.into_compile_error();

            return quote! {
                #shader_source_const ;
                #error
            }
            .into();
        }
    };

    let label = match attr.label {
        Some(label) => quote! { Some(#label) },
        None => quote! { None },
    };

    quote! {
        #shader_source_const ;

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
    use super::{Error, ShaderAttribute, Stage};
    use proc_macro2::{Span, TokenStream};
    use quote::quote;
    use shaderc::{CompileOptions, Compiler, ShaderKind};
    use std::fs;

    fn spirv_compile_error(span: Span, error: shaderc::Error) -> Error {
        let message = match error {
            shaderc::Error::CompilationError(count, info) => {
                format!("{count} compile errors:\n{info}")
            }
            shaderc::Error::InternalError(info) => format!("internal error: {info}"),
            shaderc::Error::InvalidStage(info) => format!("invalid stage: {info}"),
            shaderc::Error::InvalidAssembly(info) => format!("invalid assembly: {info}"),
            shaderc::Error::NullResultObject(info) => format!("null result object: {info}"),
            shaderc::Error::InitializationError(info) => format!("initialization error: {info}"),
            shaderc::Error::ParseError(info) => format!("parse error: {info}"),
        };

        Error::new(span, message)
    }

    fn compile_spirv(
        span: Span,
        source: &str,
        kind: ShaderKind,
        file_name: &str,
    ) -> Result<Vec<u32>, Error> {
        let compiler = Compiler::new().unwrap();
        let options = CompileOptions::new().unwrap();

        // TODO(Lorent1): add not main
        let artifact =
            match compiler.compile_into_spirv(source, kind, file_name, "main", Some(&options)) {
                Ok(artifact) => artifact,
                Err(error) => return Err(spirv_compile_error(span, error)),
            };

        Ok(artifact.as_binary().to_vec())
    }

    pub fn shader_source(attr: &ShaderAttribute) -> Result<TokenStream, Error> {
        let glsl_source = fs::read_to_string(&attr.path).unwrap();
        let spirv_words =
            compile_spirv(attr.path_span, &glsl_source, attr.stage.into(), &attr.path)?;

        Ok(quote! {
            ::wgpu::ShaderSource::SpirV(
                ::std::borrow::Cow::Borrowed(&[ #( #spirv_words ),* ]),
            )
        })
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
    use super::{Error, ShaderAttribute, Stage};
    use proc_macro2::TokenStream;
    use quote::quote;
    use std::fs;

    pub fn shader_source(attr: &ShaderAttribute) -> Result<TokenStream, Error> {
        let glsl_source = fs::read_to_string(&attr.path).unwrap();
        let stage = shader_stage(attr.stage);

        Ok(quote! {
            ::wgpu::ShaderSource::Glsl {
                shader: ::std::borrow::Cow::Borrowed(#glsl_source),
                stage: #stage,
                defines: ::std::default::Default::default(),
            }
        })
    }

    fn shader_stage(stage: Stage) -> TokenStream {
        match stage {
            Stage::Vertex => quote! { ::wgpu::naga::ShaderStage::Vertex },
            Stage::Fragment => quote! { ::wgpu::naga::ShaderStage::Fragment },
            Stage::Compute => quote! { ::wgpu::naga::ShaderStage::Compute },
        }
    }
}
