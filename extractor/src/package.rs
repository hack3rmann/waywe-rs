use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use safe_transmute::to_bytes::transmute_one_to_bytes_mut;

#[derive(thiserror::Error, Debug)]
pub enum PackageExtractError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("failed to parse string in scene.pkg")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("failed to parse file path from scene.pkg")]
    Parse,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileMeta {
    name: String,
    offset: u32,
    size: u32,
}

impl FileMeta {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageMeta {
    files: Vec<FileMeta>,
    version: String,
}

impl PackageMeta {
    pub fn files(&self) -> &[FileMeta] {
        &self.files
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageReader<'a, T: Read> {
    pub meta: PackageMeta,
    fd: &'a mut T,
}

impl<'a, T: Read> PackageReader<'a, T> {
    pub fn new(fd: &'a mut T) -> Result<Self, PackageExtractError> {
        let version = Self::read_str(fd)?;
        let filecount = Self::read_int(fd)?;

        let mut files = Vec::new();
        for _ in 0..filecount {
            files.push(FileMeta {
                name: Self::read_str(fd)?,
                offset: Self::read_int(fd)?,
                size: Self::read_int(fd)?,
            })
        }

        Ok(Self {
            meta: PackageMeta { files, version },
            fd,
        })
    }

    fn read_int(fd: &mut impl Read) -> Result<u32, PackageExtractError> {
        let mut res = 0;

        fd.read_exact(transmute_one_to_bytes_mut(&mut res))?;

        Ok(res)
    }

    fn read_str(fd: &mut impl Read) -> Result<String, PackageExtractError> {
        let size = Self::read_int(fd)?;
        let mut buf = vec![0_u8; size as usize];

        fd.read_exact(&mut buf)?;

        Ok(String::from_utf8(buf)?)
    }

    pub fn store_files(&mut self, output_dir: &Path) -> Result<(), PackageExtractError> {
        let mut path = PathBuf::new();

        for file in self.meta.files.iter() {
            let mut buf = vec![0; file.size as usize];
            self.fd.read_exact(&mut buf)?;

            path.clear();
            path.push(output_dir);
            path.push(&file.name);

            fs::create_dir_all(path.parent().ok_or(PackageExtractError::Parse)?)?;

            let mut out = File::create(&path)?;
            out.write_all(&buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires scene.pkg file to be present in the crate directory"]
    fn test_pkg_extract() {
        let mut fd = File::open("scene.pkg").unwrap();
        let mut reader = PackageReader::new(&mut fd).unwrap();

        let mut path = PathBuf::new();
        path.push("assets");
        reader.store_files(&path).unwrap();
    }
}
