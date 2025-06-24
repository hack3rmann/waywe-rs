use crate::WallpaperType;
use bincode::{
    Decode, Encode, config,
    error::{DecodeError, EncodeError},
};
use glam::UVec2;
use std::{
    borrow::Cow,
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Clone, PartialEq, PartialOrd, Debug, Hash, Eq, Ord, Encode, Decode)]
pub struct SetupProfile<'s> {
    pub wallpaper_type: WallpaperType,
    pub path: Cow<'s, Path>,
    pub monitor_size: (u32, u32),
}

impl<'s> SetupProfile<'s> {
    pub fn new(path: impl Into<Cow<'s, Path>>, wallpaper_type: WallpaperType, monitor_size: UVec2) -> Self {
        Self {
            path: path.into(),
            wallpaper_type,
            monitor_size: monitor_size.into(),
        }
    }

    pub const fn size(&self) -> UVec2 {
        let (width, height) = self.monitor_size;
        UVec2::new(width, height)
    }

    pub fn read() -> Result<Self, SetupProfileError> {
        let profile_path = {
            let mut path = cache_dir().ok_or(SetupProfileError::NoHomeDirectory)?;
            path.push("profile.bin");
            path
        };

        let mut file = File::open(&profile_path)?;

        Ok(bincode::decode_from_std_read(
            &mut file,
            config::standard(),
        )?)
    }

    pub fn store(&self) -> Result<(), SetupProfileError> {
        let cache_directory = cache_dir().ok_or(SetupProfileError::NoHomeDirectory)?;

        fs::create_dir_all(&cache_directory)?;

        let profile_path = {
            let mut path = cache_directory;
            path.push("profile.bin");
            path
        };

        let mut file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&profile_path)?;

        bincode::encode_into_std_write(self, &mut file, config::standard())?;

        Ok(())
    }
}

pub fn cache_dir() -> Option<PathBuf> {
    env::home_dir().map(|mut home| {
        home.push(".cache/waywe");
        home
    })
}

#[derive(Debug, Error)]
pub enum SetupProfileError {
    #[error("failed to find user's home directory")]
    NoHomeDirectory,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
}
