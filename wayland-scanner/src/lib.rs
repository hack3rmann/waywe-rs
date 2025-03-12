mod implementation;
mod xml;

use proc_macro::TokenStream;

/// Generates `wayland_sys::Interface`, `wayland_sys::InterfaceWlMessages`,
/// `wayland_sys::wl_interface` from protocols XML file.
///
/// # Example
///
/// ```rust
/// mod protocols {
///     wayland_scanner::include_wl_interfaces!("wayland-protocols/wayland.xml");
/// }
/// ```
#[proc_macro]
pub fn include_wl_interfaces(token_stream: TokenStream) -> TokenStream {
    implementation::include_wl_interfaces(token_stream.into()).into()
}
