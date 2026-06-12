pub mod command;
pub mod config;
pub mod detach;
pub mod epoll;
pub mod ipc;
pub mod profile;
pub mod signals;
pub mod wallpaper;

pub use command::DaemonCommand;
pub use detach::{DaemonSetupError, DaemonSetupResult};
pub use epoll::Epoll;
pub use ipc::{
    client::{IpcClient, SendError},
    server::{IpcServer, RecvError},
};
pub use wallpaper::WallpaperType;
