mod xml;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::{fs, path::PathBuf};
use syn::{LitStr, parse2};

#[proc_macro]
pub fn include_wl_interfaces(token_stream: TokenStream) -> TokenStream {
    include_wl_interfaces_impl(token_stream.into()).into()
}

fn include_wl_interfaces_impl(token_stream: TokenStream2) -> TokenStream2 {
    let str_lit = parse2::<LitStr>(token_stream).expect("expecting string literal");
    let path = PathBuf::from(str_lit.value());
    let _file_contents = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read file '{}': {err}", path.display()));

    todo!()
}
