use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Context;
use console::style;

use crate::domain::{PackageMeta, PackageName};

#[derive(Debug)]
pub struct PackageRegistry {
    packages: BTreeMap<PackageName, PackageMeta>,
}

#[allow(dead_code)]
impl PackageRegistry {
    pub fn new() -> Self {
        Self {
            packages: BTreeMap::new(),
        }
    }

    pub fn add_package(&mut self, package: PackageMeta) {
        self.packages.insert(package.thunk_name.clone(), package);
    }

    pub fn get_package(&self, package_name: &PackageName) -> Option<&PackageMeta> {
        self.packages.get(&package_name)
    }

    pub fn get_package_by_display_name(
        &self,
        display_name: &str,
    ) -> Option<(&PackageName, &PackageMeta)> {
        self.packages
            .iter()
            .find(|(_, meta)| meta.true_name == display_name)
    }

    pub fn debug_print_packages(&self) {
        println!("Packages in index:\n");

        for (thunk_name, meta) in &self.packages {
            let licensed = if !meta.contains_unlicensed_code() {
                style("Licensed").bold().green()
            } else {
                style("Unlicensed").bold().red()
            };

            println!("- {} ({}) {licensed}", thunk_name.0, meta.true_name);
        }

        println!(""); // Empty padding
    }

    /// Recursively checks a package and all of its dependencies for it is appropriately
    /// licensed.
    pub fn is_package_licensed(
        &self,
        package_name: &PackageName,
    ) -> anyhow::Result<(bool, Vec<PathBuf>)> {
        let package = self.get_package(&package_name).context(format!(
            "Package {package_name:?} does not exist in registry"
        ))?;

        let unlicensed = package.contains_unlicensed_code();
        let mut unlicensed_files = package.unlicensed_files.clone();

        if unlicensed {
            return Ok((false, unlicensed_files));
        }

        for dependency in &package.dependencies {
            let (licensed, unlicensed) = self
                .is_package_licensed(dependency)
                .context(format!("Failed to check if {dependency:?} is licensed"))?;

            unlicensed_files.extend(unlicensed);

            if !licensed {
                return Ok((false, unlicensed_files));
            }
        }

        Ok((true, unlicensed_files))
    }
}
