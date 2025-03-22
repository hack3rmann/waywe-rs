mod fmt;
mod implementation;
mod xml;

use proc_macro::TokenStream;

/// Generates `wayland_sys::Interface`, `wayland_sys::InterfaceWlMessages`,
/// `wayland_sys::wl_interface` from protocols XML file.
///
/// # Example
///
/// ```rust
/// extern crate wayland_sys;
///
/// mod protocols {
///     wayland_scanner::include_wl_interfaces!("wayland-protocols/wayland.xml");
/// }
///
/// # fn _asserts() {
/// assert_eq!(protocols::wl_display::INTERFACE.name, c"wl_display");
/// assert_eq!(protocols::wl_display::INTERFACE.version.get(), 1);
///
/// assert!(protocols::wl_surface::INTERFACE.version.get() >= 1);
/// assert_eq!(protocols::wl_surface::INTERFACE.methods[1].name, c"attach");
/// assert_eq!(protocols::wl_surface::INTERFACE.methods[1].signature, c"?oii");
/// # }
/// ```
#[proc_macro]
pub fn include_wl_interfaces(token_stream: TokenStream) -> TokenStream {
    implementation::wl::include_wl_interfaces(token_stream.into()).into()
}

#[proc_macro]
pub fn include_interfaces(token_stream: TokenStream) -> TokenStream {
    implementation::our::include_interfaces(token_stream.into()).into()
}
