use std::path::PathBuf;

use anyhow::{bail, Context};
use serde::Serialize;

#[cfg(feature = "check-licenses")]
use self::license_extractor::{
    PackageLicense, ScriptLicense, ScriptLicenses, UnlicensedPackageReason,
};
use self::{
    package_lock::PackageLock, package_name::PackageName, package_rewrite::resolve_package,
};
use crate::package_registry::PackageRegistry;

#[cfg(feature = "check-licenses")]
pub mod license_extractor;
pub mod package_lock;
pub mod package_name;
pub mod package_rewrite;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IncludeInEmit {
    Included,
    NotIncluded(NotIncludedReason),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotIncludedReason {
    OverwrittenPackage,
    #[cfg(feature = "check-licenses")]
    Unlicensed(UnlicensedPackageReason),
}

/// Represents a LuaPackage. Contains metadata about the package such as license info and dependencies.
#[derive(Debug, Serialize)]
pub struct Package {
    pub package_path: PathBuf,

    pub name: PackageName,
    pub lock: PackageLock,

    #[cfg(feature = "check-licenses")]
    #[serde(skip_serializing)]
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
            // If the package is rewritten then we'll just skip license checks.
            let (is_rewritten, _, _) =
                resolve_package(&name.registry_name, &lock.version.to_string());

            if is_rewritten {
                log::info!(
                    "Package {} is rewritten, skipping license check",
                    name.path_name
                );
            } else {
                log::info!("Computing licenses for package {}", name.path_name);
            }

            let src_path = get_package_src_path(&package_path, &name)?;
            license_extractor::compute_license_information(&src_path, is_rewritten)
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

    /// Returns whether a package is appropriately licensed.
    #[cfg(feature = "check-licenses")]
    pub fn is_package_licensed(
        &self,
        package_registry: &PackageRegistry,
    ) -> anyhow::Result<PackageLicense> {
        // First, check if *this* package is licensed. Look at dependencies later.
        if let Some(unlicensed_scripts) = self.licenses.get(&ScriptLicense::Unlicensed) {
            // This package isn't licensed, it contains unlicensed scripts!
            return Ok(PackageLicense::Unlicensed(
                UnlicensedPackageReason::UnlicensedScripts(unlicensed_scripts.to_owned()),
            ));
        }

        // This package doesn't directly contain unlicensed scripts. Check dependencies now.
        let mut unlicensed_dependencies = Vec::new();
        if let Ok(dependencies) = self.lock.parse_lock_dependencies() {
            for lock_dependency in dependencies {
                let dep_name = lock_dependency.registry_name.to_owned();
                let version = lock_dependency.version.to_owned();

                if lock_dependency.is_rewritten {
                    // We don't check license for rewritten packages because they should always be rewritten to a
                    // package that is licensed.
                    continue;
                }

                let (_, package) = package_registry
                    .resolve_lock_dependency_to_package(&lock_dependency)
                    .context(format!(
                        "Lock dependency {}@{} of package {}@{} does not exist in registry",
                        lock_dependency.registry_name,
                        lock_dependency.version.to_string(),
                        self.name.registry_name,
                        self.lock.version.to_string(),
                    ))?;

                let package_license =
                    package
                        .is_package_licensed(package_registry)
                        .context(format!(
                            "Failed to check if dependency {dep_name} is licensed"
                        ))?;

                if let PackageLicense::Unlicensed(reason) = package_license {
                    unlicensed_dependencies.push((dep_name, version, reason));
                }
            }
        }

        if !unlicensed_dependencies.is_empty() {
            // There's one or more unlicensed dependencies
            return Ok(PackageLicense::Unlicensed(
                UnlicensedPackageReason::UnlicensedDependencies(unlicensed_dependencies),
            ));
        }

        // Our package is appropriately licensed!
        // Work out which licenses are in use and return.
        let mut licenses = Vec::new();

        for (license, _) in &self.licenses {
            if let ScriptLicense::Licensed(license) = license {
                licenses.push(license.to_owned());
            }
        }

        Ok(PackageLicense::Licensed(licenses))
    }

    /// Returns if this package is rewritten as another package in dependencies. It shouldn't be included in the archive
    /// if so.
    pub fn is_package_rewritten(&self) -> bool {
        let (rewritten, _, _) =
            resolve_package(&self.name.registry_name, &self.lock.version.to_string());

        rewritten
    }

    /// Returns if it's OK to include this package in the extractors emit. This always runs, but some checks can be
    /// disabled via compiler feature flags.
    pub fn include_in_extractor_emit(
        &self,
        #[cfg(feature = "check-licenses")] package_registry: &PackageRegistry,
    ) -> IncludeInEmit {
        if self.is_package_rewritten() {
            return IncludeInEmit::NotIncluded(NotIncludedReason::OverwrittenPackage);
        }

        #[cfg(feature = "check-licenses")]
        if let Ok(PackageLicense::Unlicensed(unlicensed_reason)) =
            self.is_package_licensed(package_registry)
        {
            return IncludeInEmit::NotIncluded(NotIncludedReason::Unlicensed(unlicensed_reason));
        }

        // All checks passed
        IncludeInEmit::Included
    }
}

#[cfg(feature = "check-licenses")]
fn get_package_src_path(
    package_path: &std::path::Path,
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
        .unwrap_or_else(|| package_name.unprocessed_name.to_owned());

    let alt_src_path = package_path.join(&name);
    if alt_src_path.exists() {
        return Ok(alt_src_path);
    }

    bail!("Package doesn't contain a src/ directory");
}
