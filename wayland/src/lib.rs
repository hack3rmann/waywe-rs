pub mod init;
pub mod interface;
pub mod object;
pub mod sys;

wayland_scanner::include_wl_interfaces!("wayland-protocols/wayland.xml");
