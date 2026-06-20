pub mod command;
pub mod config;
pub mod detach;
pub mod ipc;
pub mod profile;
pub mod wallpaper;

pub use command::DaemonCommand;
pub use detach::{DaemonSetupError, DaemonSetupResult};
pub use ipc::{
    client::{IpcClient, SendError},
    server::{IpcServer, RecvError},
};
pub use wallpaper::WallpaperType;
