use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Context};
use roblox_install::RobloxStudio;

use crate::domain::{PackageMeta, PackageName};
use crate::sources::common::output::output_packages_to_path;

use super::common::{package_resolution::populate_package_registry, PackageRegistry};
use super::CorePackageSource;

/// Detects a local studio installation and extracts CorePackages from it.
#[derive(Debug)]
pub struct LocalPackageSource;

impl CorePackageSource for LocalPackageSource {
    fn extract_packages(
        extract_to: &PathBuf,
        root_packages: &Vec<PackageName>,
    ) -> anyhow::Result<()> {
        let mut package_registry = PackageRegistry::new();

        let packages_path =
            Self::get_studio_packages_path().context("Failed to find path to Packages")?;

        println!("Found path to Packages at: {packages_path:?}");

        // First, build a list of all packages and collect meta information like their line count
        // and license information.
        populate_package_registry(&mut package_registry, &packages_path)
            .context("Failed to collect CorePackages")?;

        package_registry.debug_print_packages();

        // Next, go through provided root packages and work out if each package can be included
        // (using license information of all dependencies). If any package can't be included,
        // error out early (just to be safe).
        #[cfg(not(feature = "bypass_license_check"))]
        for thunk_name in root_packages {
            println!("Checking root package {thunk_name:?} license");

            let (licensed, unlicensed_files) = package_registry
                .is_package_licensed(&thunk_name)
                .context("Failed to check if package is licensed")?;

            if !licensed {
                let mut message = format!("Package {thunk_name:?} contains unlicensed code:");
                message.push_str("\n\n");
                message.push_str(
                    &unlicensed_files
                        .iter()
                        .map(|i| i.to_str().unwrap())
                        .collect::<Vec<&str>>()
                        .join("\n"),
                );

                bail!(message);
            }
        }

        // Next, collect all the packages we want to write back out to the modules folder
        let mut packages_to_write = BTreeMap::new();
        for thunk_name in root_packages {
            Self::write_dependencies_recursive(
                &mut packages_to_write,
                &package_registry,
                thunk_name,
            )?;
        }

        // Finally, output the modules to the file system
        output_packages_to_path(&packages_to_write, &package_registry, extract_to)
            .context("Failed to write packages to output path")?;

        Ok(())
    }
}

impl LocalPackageSource {
    fn get_studio_packages_path() -> anyhow::Result<PathBuf> {
        let studio =
            RobloxStudio::locate().context("Failed to locate a Roblox Studio installation")?;

        let mut packages_path = studio.content_path().to_owned();
        packages_path.pop();
        packages_path.push("ExtraContent/LuaPackages/Packages");

        if !packages_path.exists() {
            bail!("Found a valid Studio installation, but no Packages directory");
        }

        Ok(packages_path)
    }

    fn write_dependencies_recursive<'a>(
        packages_to_write: &mut BTreeMap<&'a PackageName, &'a PackageMeta>,
        package_registry: &'a PackageRegistry,
        thunk_name: &'a PackageName,
    ) -> anyhow::Result<()> {
        let package = package_registry
            .get_package(thunk_name)
            .context("Package does not exist in registry")?;

        for thunk_name in &package.dependencies {
            if packages_to_write.contains_key(thunk_name) {
                continue;
            }

            Self::write_dependencies_recursive(packages_to_write, package_registry, thunk_name)?;
        }

        packages_to_write.insert(thunk_name, package);

        Ok(())
    }
}
