use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

use self::{package_lock::PackageLock, package_name::PackageName};

#[cfg(feature = "check-licenses")]
pub mod license_extractor;
pub mod package_lock;
pub mod package_name;

#[cfg(feature = "check-licenses")]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScriptLicense {
    Licensed(String),
    Unlicensed,
}

#[cfg(feature = "check-licenses")]
pub type ScriptLicenses = std::collections::BTreeMap<ScriptLicense, Vec<PathBuf>>;

/// Represents a LuaPackage. Contains metadata about the package such as license info and dependencies.
#[derive(Debug)]
pub struct Package {
    pub package_path: PathBuf,

    pub name: PackageName,
    pub lock: PackageLock,

    #[cfg(feature = "check-licenses")]
    pub licenses: ScriptLicenses,
}

impl Package {
    pub fn new(package_path: PathBuf) -> anyhow::Result<Self> {
        let lock_path = package_path.join("lock.toml");
        if !lock_path.exists() {
            bail!("Lock does not exist at path {lock_path:?}");
        }

        let lock =
            PackageLock::new(&lock_path).context("Failed to parse Rotriever lock.toml file")?;

        let name =
            PackageName::new(&package_path, &lock.name).context("Failed to parse package name")?;

        #[cfg(feature = "check-licenses")]
        let licenses = {
            log::info!("Computing licenses for package {}", name.path_name);

            let src_path = get_package_src_path(&package_path, &name)?;
            license_extractor::compute_license_information(&src_path)
                .context("Failed to compute license information")
        }?;

        Ok(Self {
            package_path,

            name,
            lock,

            #[cfg(feature = "check-licenses")]
            licenses,
        })
    }
}

fn get_package_src_path(
    package_path: &Path,
    package_name: &PackageName,
) -> anyhow::Result<PathBuf> {
    let src_path = package_path.join("src");
    if src_path.exists() {
        return Ok(src_path);
    }

    // Some packages contain their source in a `PackageName/PackageName`, rather than `PackageName/src`.
    let name = package_name
        .scoped_name
        .to_owned()
        .unwrap_or_else(|| package_name.registry_name.to_owned());

    let alt_src_path = package_path.join(&name);
    if alt_src_path.exists() {
        return Ok(alt_src_path);
    }

    bail!("Package doesn't contain a src/ directory");
}
