pub mod display;
pub mod object;
pub mod object_storage;
pub mod proxy;
pub mod wire;

pub mod protocol {
    use wayland_scanner::include_wl_interfaces;

    include_wl_interfaces!("wayland-protocols/wayland.xml");

    include_wl_interfaces!("wayland-protocols/stable/xdg-shell/xdg-shell.xml");

    include_wl_interfaces!("wayland-protocols/stable/viewporter/viewporter.xml");

    include_wl_interfaces!(
        "wayland-protocols/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml"
    );
}
