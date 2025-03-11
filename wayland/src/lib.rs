pub mod init;
pub mod interface;
pub mod object;
pub mod sys;

use wayland_scanner::include_wl_interfaces;

include_wl_interfaces!("wayland-protocols/wayland.xml");
