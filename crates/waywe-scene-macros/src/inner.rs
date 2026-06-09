use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse2};

pub fn derive_scene(input: TokenStream) -> TokenStream {
    let input = parse2::<DeriveInput>(input).unwrap();
    let scene_type = &input.ident;

    let waywe_scene = quote! { ::waywe_scene };
    let render_api = quote! { ::waywe_rendering_api };

    quote! {
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn waywe_ffi_create_opaque_renderer(
            __desc: & #render_api ::api::OpaqueRendererDesc,
            __out_renderer: &mut ::std::mem::MaybeUninit<#render_api ::api::OpaqueRenderer>,
        ) -> #render_api ::ffi::PanicPayload {
            let __result = ::std::panic::catch_unwind(move || {
                #waywe_scene ::ffi::create_opaque_renderer::< #scene_type >(__desc)
            });

            #render_api ::ffi::PanicPayload::map(__result, |__renderer| {
                __out_renderer.write(__renderer);
            })
        }
    }
}
