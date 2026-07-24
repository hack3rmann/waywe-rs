pub mod command;
pub mod detach;
pub mod ipc;
pub mod profile;
pub mod wallpaper;

pub use command::DaemonCommand;
pub use detach::{DaemonSetupError, DaemonSetupResult};
pub use ipc::{
    client::{ClientError, IpcClient},
    server::{IpcServer, RecvError},
};
pub use wallpaper::WallpaperType;
