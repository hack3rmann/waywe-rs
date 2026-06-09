mod inner;

use proc_macro::TokenStream;

#[proc_macro_derive(Scene)]
pub fn derive_scene(input: TokenStream) -> TokenStream {
    inner::derive_scene(input.into()).into()
}
