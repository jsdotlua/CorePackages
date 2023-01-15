use std::{collections::HashMap, fs, path::Path};

use anyhow::Context;
#[cfg(feature = "check-licenses")]
use askalono::Store;
use derive_more::Deref;

use crate::package::Package;

/// Numeric reference to a specific package in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref)]
pub struct PackageRef(u32);

/// Source of truth for LuaPackages. Maintains a list of all discovered Packages.
pub struct PackageRegistry {
    pub packages: HashMap<PackageRef, Package>,
    pub package_count: u32,

    #[cfg(feature = "check-licenses")]
    license_store: Store,
}

impl PackageRegistry {
    pub fn new() -> anyhow::Result<Self> {
        #[cfg(feature = "check-licenses")]
        let license_store = {
            let mut store = Store::new();

            log::info!("Loading SPDX data, this may take a while...");
            store
                .load_spdx(
                    Path::new(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/datasets/modules/license-list-data/json/details"
                    )),
                    false,
                )
                .context("Failed to load SPDX data")?;

            store
        };

        Ok(Self {
            packages: HashMap::new(),
            package_count: 0,

            #[cfg(feature = "check-licenses")]
            license_store,
        })
    }

    /// Find a package in the registry by its path name
    pub fn find_by_path_name(&self, path_name: &str) -> Option<(&PackageRef, &Package)> {
        self.packages
            .iter()
            .find(|(_, package)| package.name.path_name == path_name)
    }

    /// Find a package in the registry by its registry name
    pub fn find_by_registry_name(&self, registry_name: &str) -> Option<(&PackageRef, &Package)> {
        self.packages
            .iter()
            .find(|(_, package)| package.name.registry_name == registry_name)
    }

    /// Search through an `_Index` directory for packages.
    pub fn discover_packages_at_index(&mut self, index_path: &Path) -> anyhow::Result<()> {
        let entries = fs::read_dir(index_path)
            .context(format!("Failed to read index path: {index_path:?}"))?;

        let start_package_count = self.package_count;

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if !path.is_dir() {
                    // All packages are a directory
                    continue;
                }

                let lock_path = path.join("lock.toml");
                if !lock_path.exists() {
                    log::warn!(
                        "Found directory in _Index that doesn't contain a lock file: {path:?}"
                    );

                    continue;
                }

                // Package constructor can take it from here
                let package = Package::new(
                    path.to_owned(),
                    #[cfg(feature = "check-licenses")]
                    &self.license_store,
                )
                .context(format!("Failed to create package from path {path:?}"))?;

                log::debug!("Discovered package: {}", package.name.path_name);

                self.package_count += 1;

                let package_ref = PackageRef(self.package_count);
                self.packages.insert(package_ref, package);
            }
        }

        log::info!(
            "Discovered {} new packages",
            self.package_count - start_package_count
        );

        Ok(())
    }
}