use flate2::bufread::GzDecoder;
use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::{self, Debug},
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex, Weak},
};
use tap::Pipe;
use tar::Archive;
use uuid::Uuid;
use waywe_runtime_dir::runtime_dir;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WallpaperPackage {
    pub source_path: PathBuf,
    pub package_path: PathBuf,
    pub unpacked_path: PathBuf,
    pub wallpaper_path: PathBuf,
}

impl WallpaperPackage {
    pub fn inflate(path: impl Into<PathBuf>) -> Self {
        let source_path = path.into();
        let mut unpacked_path = PathBuf::new();

        loop {
            let uuid = Uuid::now_v7();

            unpacked_path.clear();
            unpacked_path.push(&*PACKAGES_DIR);
            unpacked_path.push(uuid.to_string());

            if !unpacked_path.exists() {
                break;
            }
        }

        fs::create_dir_all(unpacked_path.parent().unwrap()).unwrap();

        let mut archive = File::open(&source_path)
            .unwrap()
            .pipe(BufReader::new)
            .pipe(GzDecoder::new)
            .pipe(Archive::new);

        archive.unpack(&unpacked_path).unwrap();

        let package_path = fs::read_dir(&unpacked_path)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();

        Self {
            wallpaper_path: package_path.join("wallpaper.so"),
            package_path,
            source_path,
            unpacked_path,
        }
    }
}

impl Drop for WallpaperPackage {
    fn drop(&mut self) {
        _ = fs::remove_dir_all(&self.unpacked_path);
    }
}

pub static PACKAGES_DIR: LazyLock<PathBuf> = LazyLock::new(|| runtime_dir().join("packages"));

#[derive(Clone, Default, Debug)]
pub struct PackageRegistryInner {
    pub packages: HashMap<PathBuf, Weak<WallpaperPackage>>,
}

#[derive(Clone, Default)]
pub struct PackageRegistry(Arc<Mutex<PackageRegistryInner>>);

impl PackageRegistry {
    pub fn inflate(&self, package_source: impl Into<PathBuf>) -> Arc<WallpaperPackage> {
        let package_source = package_source.into();
        let mut this = self.0.lock().unwrap();

        // Remove dropped packages
        this.packages
            .retain(|_, package| package.strong_count() != 0);

        let inflate_package = {
            let package_source = package_source.clone();
            // TODO(hack3rmann): return error on inflate fail
            move || Arc::new(WallpaperPackage::inflate(package_source))
        };

        match this.packages.entry(package_source) {
            Entry::Occupied(mut entry) => match entry.get_mut().upgrade() {
                Some(package) => package,
                None => {
                    let package = inflate_package();
                    entry.insert(Arc::downgrade(&package));
                    package
                }
            },
            Entry::Vacant(entry) => {
                let package = inflate_package();
                entry.insert(Arc::downgrade(&package));
                package
            }
        }
    }
}

impl Debug for PackageRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.0.lock().unwrap();
        inner.packages.fmt(f)
    }
}

impl Drop for PackageRegistryInner {
    fn drop(&mut self) {
        _ = fs::remove_dir_all(&*PACKAGES_DIR);
    }
}
