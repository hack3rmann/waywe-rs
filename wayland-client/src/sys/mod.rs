//! Safe wrappers on libwayland types

pub mod display;
pub(crate) mod log;
pub mod object;
pub mod object_storage;
pub mod proxy;
pub(crate) mod thin;
pub mod wire;

/// Wayland interfaces and their signatures
pub mod protocol {
    use wayland_scanner::include_wl_interfaces;

    include_wl_interfaces!("wayland-protocols/wayland.xml");

    include_wl_interfaces!("wayland-protocols/stable/xdg-shell/xdg-shell.xml");

    include_wl_interfaces!("wayland-protocols/stable/viewporter/viewporter.xml");

    include_wl_interfaces!(
        "wayland-protocols/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml"
    );
}
