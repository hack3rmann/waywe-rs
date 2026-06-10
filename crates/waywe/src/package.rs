use crate::{
    args::PackageCommand,
    command::ExecuteError,
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

    let mut archive = File::create(&output_path)?
        .pipe(|file| GzEncoder::new(file, Compression::default()))
        .pipe(|encoder| {
            let mut builder = Builder::new(encoder);
            // Large .so files are detected as sparse on Linux; GNU sparse entries
            // (typeflag 'S') are not extracted by tools like ouch.
            builder.sparse(false);
            builder
        });

    let wallpaper_entry = format!("{}/wallpaper.so", package.name);
    status("Adding", &wallpaper_entry);
    archive.append_path_with_name(&dylib_path, &wallpaper_entry)?;

    let assets_dir = path.join("assets");
    if assets_dir.is_dir() {
        append_assets_dir(&mut archive, &assets_dir, &package.name)?;
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

fn append_assets_dir(
    archive: &mut Builder<GzEncoder<File>>,
    assets_dir: &Path,
    package_name: &str,
) -> Result<(), ExecuteError> {
    let mut entries = vec![assets_dir.to_path_buf()];

    while let Some(dir) = entries.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_path = entry.path();

            if file_type.is_dir() {
                entries.push(file_path);
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let relative = file_path
                .strip_prefix(assets_dir)
                .expect("asset path must be under assets directory");
            status("Adding", relative.display());

            let archive_path = format!(
                "{package_name}/assets/{relative}",
                relative = relative.display()
            );

            archive.append_path_with_name(&file_path, &archive_path)?;
        }
    }

    Ok(())
}
