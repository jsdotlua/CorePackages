use std::env;

use anyhow::Context;
use libextractor::{
    domain::PackageName,
    sources::{CorePackageSource, LocalPackageSource},
};

fn main() -> anyhow::Result<()> {
    let path = env::current_dir()?.join("../modules/");
    LocalPackageSource::extract_packages(
        &path,
        &vec![PackageName("RoactCompat-9c8468d8-8a7220fd".into())],
    )
    .context("Failed to extract CorePackages")?;

    Ok(())
}
