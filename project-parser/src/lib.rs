use std::io::{self, Read};
use std::str::FromStr;
use std::{env::home_dir, fs::File, path::PathBuf};

use chumsky::Parser;

pub mod library_folders;

const WALPAPER_ENGINE_STEAM_ID: usize = 431960;

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

pub fn locate_we_assets() -> LocateResult<PathBuf> {
    let library_folders = home_dir()
        .ok_or(LocateError::HomeDirError)?
        .join(".steam/steam/steamapps/libraryfolders.vdf");

    let library_folders = File::open(&library_folders).map_err(|_| LocateError::NotFound)?;
    let mut library_folders = io::BufReader::new(library_folders);

    let mut buf = String::new();
    library_folders.read_to_string(&mut buf)?;

    let library_folders = library_folders::library_folders()
        .parse(&buf)
        .into_result()
        .map_err(|_| LocateError::ParseError)?;

    let mut we_installation_path = None;

    for library_folder in library_folders.iter() {
        for app_id in library_folder.apps_ids.iter() {
            if app_id == &WALPAPER_ENGINE_STEAM_ID {
                we_installation_path = Some(&library_folder.path);
            }
        }
    }

    let Some(we_installation_path) = we_installation_path else {
        return Err(LocateError::NotFound);
    };

    Ok(PathBuf::from_str(&we_installation_path)
        .expect("infallible")
        .join("steamapps/workshop/content/431960"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires wallpaper engine to be installed in /home/arno/home-ext/Games/steamapps/workshop/content/431960"]
    fn library_folders_parse() {
        let we_assets_location = locate_we_assets().unwrap();

        assert_eq!(
            we_assets_location,
            PathBuf::from_str("/home/arno/home-ext/Games/steamapps/workshop/content/431960")
                .unwrap()
        )
    }
}
