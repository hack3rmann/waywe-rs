use clap::{Parser, Subcommand};
use runtime::{ipc::Client, DaemonCommand, IpcSocket};
use std::{ffi::CString, path::Path};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Video {
        /// Path/URL to the video
        path: CString,
    },
}

fn main() {
    let args = Args::parse();
    let socket = IpcSocket::<Client, DaemonCommand>::connect().unwrap();

    let Command::Video { mut path } = args.command;

    let path_str = std::str::from_utf8(path.to_bytes()).unwrap();
    let path_value = Path::new(path_str);

    if path_value.is_relative() {
        let full_path = path_value.canonicalize().unwrap();
        let mut full_path_bytes = full_path.into_os_string().into_encoded_bytes();
        full_path_bytes.push(0);
        path = CString::from_vec_with_nul(full_path_bytes).unwrap();
    }

    socket.send(DaemonCommand::SetVideo { path }).unwrap();
}
