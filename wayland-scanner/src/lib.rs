mod implementation;
mod xml;

use proc_macro::TokenStream;

#[proc_macro]
pub fn include_wl_interfaces(token_stream: TokenStream) -> TokenStream {
    implementation::include_wl_interfaces(token_stream.into()).into()
}
