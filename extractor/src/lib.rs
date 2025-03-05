use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use safe_transmute::to_bytes::transmute_one_to_bytes_mut;

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
    pub fn new(fd: &'a mut T) -> Self {
        let version = Self::read_str(fd);
        let filecount = Self::read_int(fd);

        let mut files = Vec::new();
        for _ in 0..filecount {
            files.push(FileMeta {
                name: Self::read_str(fd),
                offset: Self::read_int(fd),
                size: Self::read_int(fd),
            })
        }

        Self {
            meta: PackageMeta { files, version },
            fd,
        }
    }

    fn read_int(fd: &mut impl Read) -> u32 {
        let mut res = 0;

        fd.read_exact(transmute_one_to_bytes_mut(&mut res))
            .expect("failed to read file");

        res
    }

    fn read_str(fd: &mut impl Read) -> String {
        let size = Self::read_int(fd);
        let mut buf = vec![0_u8; size as usize];

        fd.read_exact(&mut buf).expect("failed to read from file");

        String::from_utf8(buf).expect("failed to parse string in scene.pkg file")
    }

    pub fn store_files(&mut self, output_dir: &Path) {
        for file in self.meta.files.iter() {
            let mut buf = vec![0; file.size as usize];
            self.fd.read_exact(&mut buf).expect("failed to read file");

            let path = output_dir.join(&file.name);

            fs::create_dir_all(path.parent().expect("invalid file path"))
                .expect("failed to create ouput directory");

            let mut out = File::create(path).expect("failed to create output file");
            out.write_all(&buf).expect("failed to store file");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test() {
        let mut fd = File::open("scene.pkg").unwrap();

        let mut reader = PackageReader::new(&mut fd);

        reader = dbg!(reader);

        let mut path = PathBuf::new();
        path.push("assets");
        reader.store_files(&path);
    }
}
