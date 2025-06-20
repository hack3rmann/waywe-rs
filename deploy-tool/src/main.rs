use anyhow::{Context, Error};
use clap::Parser;
use cmd_lib::{run_cmd, run_fun};
use lazy_regex::regex_find;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the cli tool executable
    #[arg(long)]
    cli: PathBuf,
    /// Path to the daemon executable
    #[arg(long)]
    daemon: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let version = run_fun!(cat "Cargo.toml" | rg -A 3 "workspace.package" | rg "version")?;
    assert!(!version.is_empty());

    let version =
        regex_find!(r"\d*\.\d*\.\d*", &version).context("package version does not match")?;
    let v_version = format!("v{version}");

    let package_tool_name = run_fun!(cat "deploy-tool/Cargo.toml" | rg "name = ")?;
    assert_eq!(package_tool_name, "name = \"deploy-tool\"");

    let build_name = format!("waywe-{v_version}-linux-wayland-x86_64");

    let releases_path = Path::new("deploy-tool/releases/").join(&build_name);
    fs::create_dir_all(&releases_path).await?;

    let package = Path::new("/tmp/waywe/releases/").join(&build_name);
    fs::create_dir_all(&package).await?;

    let bin = package.join("bin");
    fs::create_dir_all(&bin).await?;

    tokio::try_join!(
        fs::copy(&args.cli, bin.join(args.cli.file_name().unwrap())),
        fs::copy(&args.daemon, bin.join(args.daemon.file_name().unwrap())),
        fs::copy("README.md", package.join("README.md")),
        fs::copy("LICENSE", package.join("LICENSE")),
        fs::copy("deploy-tool/INSTALL.md", package.join("INSTALL.md")),
        fs::copy("deploy-tool/install.sh", package.join("install.sh")),
        fs::copy("deploy-tool/uninstall.sh", package.join("uninstall.sh")),
    )?;

    run_cmd!(ouch compress $package $package.tar.gz)?;
    run_cmd!(mv $package.tar.gz $releases_path)?;

    let tmp = package.parent().unwrap();
    run_cmd!(rm -rf $tmp)?;

    Ok(())
}
