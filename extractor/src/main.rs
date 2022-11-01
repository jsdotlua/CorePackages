use std::{env, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use libextractor::{
    domain::PackageName,
    sources::{CorePackageSource, LocalPackageSource},
};

/// Extracts Roblox CorePackages into the specified directory, structured specifically for Wally.
#[derive(Debug, Parser)]
struct Args {
    /// List of packages names to search for first.
    #[arg(short, long, required = true)]
    root_packages: Vec<String>,

    #[arg(short, long, required = true)]
    /// Directory to write all extracted packages.
    export_to: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let path = resolve_path(&args.export_to);

    let root_packages = &args
        .root_packages
        .iter()
        .map(|i| PackageName(i.to_owned()))
        .collect();

    LocalPackageSource::extract_packages(&path, root_packages)
        .context("Failed to extract CorePackages")?;

    Ok(())
}

// https://github.com/rojo-rbx/rojo/blob/b88d34c639b7d7bdd4171b7846a64c2b13f0c2d5/src/cli/mod.rs#L124
fn resolve_path(path: &PathBuf) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        env::current_dir().unwrap().join(path)
    }
}
