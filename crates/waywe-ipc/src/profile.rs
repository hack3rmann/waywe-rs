use crate::WallpaperType;
use bincode::{
    Decode, Encode, config,
    error::{DecodeError, EncodeError},
};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io,
    path::PathBuf,
};
use thiserror::Error;

#[derive(Clone, PartialEq, PartialOrd, Debug, Hash, Eq, Ord, Encode, Decode)]
pub struct ProfileInfo {
    pub ty: WallpaperType,
    pub path: PathBuf,
}

#[derive(Clone, Default, PartialEq, Debug, Eq, Encode, Decode)]
pub struct SetupProfile {
    for_all: Option<ProfileInfo>,
    per_monitor: HashMap<String, ProfileInfo>,
}

impl SetupProfile {
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

    pub fn all(&mut self, info: ProfileInfo) -> &mut Self {
        self.for_all = Some(info);
        self.per_monitor.clear();
        self
    }

    pub fn with(&mut self, name: String, info: ProfileInfo) -> &mut Self {
        self.per_monitor.insert(name, info);
        self
    }

    pub fn take(&mut self, name: &str) -> Option<ProfileInfo> {
        self.per_monitor
            .remove(name)
            .or_else(|| self.for_all.take())
    }

    pub fn save(&self) -> Result<(), SetupProfileError> {
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
