use std::io;

#[derive(thiserror::Error, Debug)]
pub enum LocateError {
    #[error("failed to locate home directory")]
    HomeDirError,

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("wallpaper engine or steam itself is not installed")]
    NotFound,

    #[error("error parsign libraryfolders.vdf file")]
    ParseError,
}

pub type LocateResult<T> = Result<T, LocateError>;
