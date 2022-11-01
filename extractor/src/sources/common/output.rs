use std::{collections::BTreeMap, fs, path::PathBuf};

use anyhow::Context;
use console::style;
use serde_json::json;

use crate::{
    constants::{BANNED_PACKAGE_NAMES, DEPENDENCY_ALIASES},
    domain::{PackageMeta, PackageName, WallyConfig, WallyConfigPackage},
};

use super::{source_utils::get_file_source, PackageRegistry};

pub fn output_packages_to_path(
    packages: &BTreeMap<&PackageName, &PackageMeta>,
    package_registry: &PackageRegistry,
    output_path: &PathBuf,
) -> anyhow::Result<()> {
    // Some terminal padding
    println!("");

    for (package_name, package_meta) in packages {
        if BANNED_PACKAGE_NAMES.contains(&package_name.as_str()) {
            continue;
        }

        let root_folder = output_path.join(&package_name.to_string());

        fs::create_dir(&root_folder).context(format!(
            "Failed to create directory for package at path {root_folder:?}"
        ))?;

        write_wally_file(&root_folder, package_meta, package_registry).context(format!(
            "Failed to write Wally file for package {package_name:?}"
        ))?;

        write_project_file(&root_folder, package_meta).context(format!(
            "Failed to write project file for package {package_name:?}"
        ))?;

        write_source_files(&root_folder, package_meta).context(format!(
            "Failed to write source files for package {package_name:?}"
        ))?;

        println!(
            "Successfully outputted package {}",
            style(&package_name.0).bold().cyan()
        );
    }

    Ok(())
}

fn write_wally_file(
    path: &PathBuf,
    package_meta: &PackageMeta,
    package_registry: &PackageRegistry,
) -> anyhow::Result<()> {
    let package_license = package_meta
        .licenses
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(" + ");

    let package_dependencies = package_meta
        .dependency_thunk_names
        .iter()
        .map(|(package_name, thunk_name)| {
            if DEPENDENCY_ALIASES.contains_key(package_name) {
                (package_name.to_string(), thunk_name.to_owned())
            } else {
                let package = package_registry
                    .get_package(package_name)
                    // SAFETY: At this point we know `thunk_name` is valid. Unwrapping simplifies this a lot.
                    .expect("Valid dependency name");

                (
                    thunk_name.to_owned(),
                    format!(
                        "core-packages/{}@{}",
                        package.wally_complaint_name,
                        package.version.to_string()
                    ),
                )
            }
        })
        .collect::<BTreeMap<String, String>>();

    let wally_file = WallyConfig {
        dependencies: package_dependencies,
        package: WallyConfigPackage {
            name: format!("core-packages/{}", package_meta.wally_complaint_name),
            description: "https://github.com/grilme99/CorePackages".into(),
            version: package_meta.version.to_string(),
            authors: vec!["Roblox Corporation".into()],
            license: package_license,
            registry: "https://github.com/UpliftGames/wally-index".into(),
            realm: "shared".into(),
        },
    };

    let source = toml::to_string_pretty(&wally_file)?;

    let path = path.join("wally.toml");
    fs::write(path, source).context("Failed to write wally.toml")?;

    Ok(())
}

fn write_project_file(path: &PathBuf, package_meta: &PackageMeta) -> anyhow::Result<()> {
    let project = json!({
        "name": package_meta.wally_complaint_name,
        "tree": {
            "$path": "src/"
        }
    });

    let source = serde_json::to_string_pretty(&project)?;

    let path = path.join("default.project.json");
    fs::write(path, source).context("Failed to write default.project.json")?;

    Ok(())
}

fn write_source_files(path: &PathBuf, package_meta: &PackageMeta) -> anyhow::Result<()> {
    let root_path = path.join("src/");
    fs::create_dir(&root_path).context("Failed to create src/ directory")?;

    // Recursively step through the package's original source files and write them back here
    // We need to do this instead of just copying the source directory because we need to
    // manually replace the source of some files.
    let package_path = &package_meta.package_path;
    write_back_directory(&root_path, package_path)
        .context("Failed to write back source directory")?;

    Ok(())
}

fn write_back_directory(write_to: &PathBuf, current_path: &PathBuf) -> anyhow::Result<()> {
    let entires =
        fs::read_dir(current_path).context(format!("Failed to read directory {current_path:?}"))?;

    for entry in entires {
        let entry = entry?;

        let path = entry.path();
        let file_name = path
            .file_name()
            .context(format!("Failed to get file name of path {path:?}"))?;

        let new_path = write_to.join(file_name);

        if path.is_dir() {
            fs::create_dir(&new_path)
                .context(format!("Failed to create directory at path {new_path:?}"))?;

            write_back_directory(&new_path, &path)
                .context(format!("Failed to write back directory {path:?}"))?;
        } else {
            let content =
                get_file_source(&path).context(format!("Failed to read path {path:?}"))?;

            fs::write(&new_path, content)
                .context(format!("Failed to write to path {new_path:?}"))?;
        }
    }

    Ok(())
}
