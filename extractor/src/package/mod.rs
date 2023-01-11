use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
#[cfg(feature = "check-licenses")]
use askalono::Store;

use self::{package_lock::PackageLock, package_name::PackageName};

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
    pub fn new(
        package_path: PathBuf,
        #[cfg(feature = "check-licenses")] license_store: &Store,
    ) -> anyhow::Result<Self> {
        let lock_path = package_path.join("lock.toml");
        if !lock_path.exists() {
            bail!("Lock does not exist at path {lock_path:?}");
        }

        let lock =
            PackageLock::new(&lock_path).context("Failed to parse Rotriever lock.toml file")?;

        let name =
            PackageName::new(&package_path, &lock).context("Failed to parse package name")?;

        #[cfg(feature = "check-licenses")]
        let licenses = {
            log::info!("Computing licenses for package {}", name.path_name);

            let src_path = get_package_src_path(&package_path, &name)?;
            compute_license_information(&src_path, license_store)
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

/// Walks through all source files in the directory and computes license information.
#[cfg(feature = "check-licenses")]
fn compute_license_information(
    src_path: &Path,
    license_store: &Store,
) -> anyhow::Result<ScriptLicenses> {
    use std::{collections::BTreeMap, fs};

    let mut licenses: BTreeMap<ScriptLicense, Vec<PathBuf>> = BTreeMap::new();

    for entry in walkdir::WalkDir::new(src_path) {
        if let Ok(entry) = entry {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // We only care about Lua and Luau files right now
            if let Some(extension) = path.extension() {
                if !(extension == "lua" || extension == "luau") {
                    continue;
                }
            }

            let script_source = fs::read_to_string(path)
                .context(format!("Failed to read script to string: {path:?}"))?;

            let matched = license_store.analyze(&script_source.into());
            let license = if matched.score > 0.8 {
                ScriptLicense::Licensed(matched.name.to_owned())
            } else {
                ScriptLicense::Unlicensed
            };

            if let Some(license_record) = licenses.get_mut(&license) {
                license_record.push(path.to_owned());
            } else {
                licenses.insert(license, vec![path.to_owned()]);
            }
        }
    }

    Ok(licenses)
}
