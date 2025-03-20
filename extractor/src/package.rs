//! Functions and structs for extracting `scene.pkg` files.
//!
//! For the module entry point reference [`PackageReader`]

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use safe_transmute::to_bytes::transmute_one_to_bytes_mut;

/// Error that may occur while working with `scene.pkg` file
#[derive(thiserror::Error, Debug)]
pub enum PackageExtractError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("failed to parse string in scene.pkg")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    /// Failed to create a path read from scene.pkg
    #[error("failed to parse file path from scene.pkg")]
    Parse,
}

/// Info about a file contained in scene.pkg
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileMeta {
    /// Name of the file
    pub name: String,
    /// Offset in scene.pkg where this file data starts
    pub offset: u32,
    /// Size of the file
    pub size: u32,
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

/// Info about `scene.pkg` file
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageMeta {
    pub files: Vec<FileMeta>,
    pub version: String,
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
struct Reader<'a, T: Read>(&'a mut T);

impl<T: Read> Reader<'_, T> {
    fn read_int(&mut self) -> Result<u32, PackageExtractError> {
        let mut res = 0;

        self.0.read_exact(transmute_one_to_bytes_mut(&mut res))?;

        Ok(res)
    }

    fn read_str(&mut self) -> Result<String, PackageExtractError> {
        let size = self.read_int()?;

        let mut buf = vec![0_u8; size as usize];

        self.0.read_exact(&mut buf)?;

        Ok(String::from_utf8(buf)?)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), PackageExtractError> {
        self.0.read_exact(buf)?;

        Ok(())
    }
}

/// Entry point of the module. Reader for the `scene.pkg` files
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageReader<'a, T: Read> {
    meta: PackageMeta,
    reader: Reader<'a, T>,
}

impl<'a, T: Read> PackageReader<'a, T> {
    /// Reads meta info from the file and constructs Self
    pub fn new(fd: &'a mut T) -> Result<Self, PackageExtractError> {
        let mut reader = Reader(fd);
        let version = reader.read_str()?;
        let filecount = reader.read_int()?;

        let mut files = Vec::new();
        for _ in 0..filecount {
            files.push(FileMeta {
                name: reader.read_str()?,
                offset: reader.read_int()?,
                size: reader.read_int()?,
            })
        }

        Ok(Self {
            meta: PackageMeta { files, version },
            reader,
        })
    }

    pub fn meta(&self) -> &PackageMeta {
        &self.meta
    }

    /// Reads the rest of the file to get the actual encoded files data and stores them on disk
    /// in the `output_dir`
    pub fn store_files(&mut self, output_dir: &Path) -> Result<(), PackageExtractError> {
        let mut path = PathBuf::new();

        for file in self.meta.files.iter() {
            let mut buf = vec![0; file.size as usize];
            self.reader.read_exact(&mut buf)?;

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
    // #[ignore = "requires scene.pkg file to be present in the crate directory"]
    fn test_pkg_extract() {
        let mut fd = File::open("scene.pkg").unwrap();
        let mut reader = PackageReader::new(&mut fd).unwrap();

        let mut path = PathBuf::new();
        path.push("assets");
        reader.store_files(&path).unwrap();
    }
}
