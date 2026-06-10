use crate::{
    args::PackageCommand,
    command::ExecuteError,
    progress::Progress,
    status::{format_elapsed, status},
};
use flate2::{Compression, write::GzEncoder};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};
use tap::Pipe;
use tar::Builder;

struct ArchiveEntry {
    source: PathBuf,
    archive_path: String,
    log_name: String,
}

pub fn execute_package(command: PackageCommand) -> Result<(), ExecuteError> {
    match command {
        PackageCommand::Build { debug, path } => {
            let kind = if debug {
                BuildKind::Debug
            } else {
                BuildKind::Release
            };

            execute_package_build(kind, path.into())?;
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuildKind {
    #[default]
    Release,
    Debug,
}

impl BuildKind {
    pub const fn flag(self) -> Option<&'static str> {
        match self {
            BuildKind::Release => Some("--release"),
            BuildKind::Debug => None,
        }
    }

    pub const fn profile_dir(self) -> &'static str {
        match self {
            BuildKind::Release => "release",
            BuildKind::Debug => "debug",
        }
    }
}

pub fn execute_package_build(kind: BuildKind, path: PathBuf) -> Result<(), ExecuteError> {
    let manifest_path = path.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Err(ExecuteError::InvalidPackageRoot { path });
    }

    let manifest_path = fs::canonicalize(&manifest_path)?;

    let build_status = Command::new("cargo")
        .current_dir(&path)
        .arg("build")
        .args(kind.flag())
        .status()?;

    if !build_status.success() {
        return Err(ExecuteError::CargoBuild {
            status: build_status,
        });
    }

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()?;

    let package = metadata
        .packages
        .iter()
        .find(|package| package.manifest_path.as_std_path() == manifest_path)
        .ok_or_else(|| ExecuteError::InvalidPackageRoot { path: path.clone() })?;

    let has_dylib = package.targets.iter().any(|target| {
        target.kind.iter().any(|kind| {
            matches!(
                kind,
                cargo_metadata::TargetKind::DyLib | cargo_metadata::TargetKind::CDyLib
            )
        })
    });

    if !has_dylib {
        return Err(ExecuteError::NotADylibCrate {
            manifest: manifest_path,
        });
    }

    let lib_name = package.name.replace('-', "_");
    let dylib_path = metadata
        .target_directory
        .join(kind.profile_dir())
        .join(format!("lib{lib_name}.so"));

    if !dylib_path.is_file() {
        return Err(ExecuteError::DylibNotFound {
            path: dylib_path.into(),
        });
    }

    let output_path = metadata
        .target_directory
        .join(kind.profile_dir())
        .join(format!("{}.ww", package.name));

    let crate_path = manifest_path.parent().expect("manifest path has a parent");
    let started = Instant::now();

    status(
        "Packaging",
        format_args!(
            "{} v{} ({})",
            package.name,
            package.version,
            crate_path.display()
        ),
    );

    let wallpaper_entry = format!("{}/wallpaper.so", package.name);
    let mut entries = vec![ArchiveEntry {
        source: dylib_path.into(),
        archive_path: wallpaper_entry.clone(),
        log_name: wallpaper_entry,
    }];

    let assets_dir = path.join("assets");
    if assets_dir.is_dir() {
        collect_asset_entries(&assets_dir, &package.name, &mut entries)?;
    }

    let mut archive = File::create(&output_path)?
        .pipe(|file| GzEncoder::new(file, Compression::default()))
        .pipe(|encoder| {
            let mut builder = Builder::new(encoder);
            // Large .so files are detected as sparse on Linux; GNU sparse entries
            // (typeflag 'S') are not extracted by tools like ouch.
            builder.sparse(false);
            builder
        });

    {
        let total = entries.len();
        let mut progress = Progress::new("Compressing");

        for (index, entry) in entries.iter().enumerate() {
            status("Adding", &entry.log_name);
            progress.tick(index + 1, total, &entry.log_name);
            archive.append_path_with_name(&entry.source, &entry.archive_path)?;
        }
    }

    let encoder = archive.into_inner()?;
    encoder.finish()?;

    status(
        "Finished",
        format_args!(
            "package [{}] at `{}` in {}",
            kind.profile_dir(),
            output_path.as_str(),
            format_elapsed(started.elapsed())
        ),
    );

    Ok(())
}

fn collect_asset_entries(
    assets_dir: &Path,
    package_name: &str,
    entries: &mut Vec<ArchiveEntry>,
) -> Result<(), ExecuteError> {
    let mut dirs = vec![assets_dir.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_path = entry.path();

            if file_type.is_dir() {
                dirs.push(file_path);
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let relative = file_path
                .strip_prefix(assets_dir)
                .expect("asset path must be under assets directory");
            let log_name = relative
                .to_str()
                .map(str::to_owned)
                .unwrap_or_else(|| relative.display().to_string());

            entries.push(ArchiveEntry {
                source: file_path,
                archive_path: format!("{package_name}/assets/{log_name}"),
                log_name,
            });
        }
    }

    Ok(())
}
