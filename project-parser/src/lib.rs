use std::io::{self, Read};
use std::{env::home_dir, fs::File, path::PathBuf};

pub mod library_folders;

#[derive(thiserror::Error, Debug)]
pub enum LocateError {
    #[error("failed to locate home directory")]
    HomeDirError,

    #[error(transparent)]
    IoError(#[from] io::Error),
    // #[error(transparent)]
    // VdfError(#[from] vdf_serde::Error),
}

pub type LocateResult<T> = Result<T, LocateError>;

pub fn locate_we_assets() -> LocateResult<PathBuf> {
    let library_folders = home_dir()
        .ok_or(LocateError::HomeDirError)?
        .join(".steam/steam/steamapps/libraryfolders.vdf");

    let library_folders = File::open(&library_folders)?;
    let mut library_folders = io::BufReader::new(library_folders);

    let mut buf = String::new();
    library_folders.read_to_string(&mut buf)?;

    // TODO(ArnoDarkrose): implement the rest of the function

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn library_folders_parse() {
        locate_we_assets().unwrap();
    }
}
