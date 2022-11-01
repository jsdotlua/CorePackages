use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use convert_case::{Case, Casing};
use semver::Version;
use walkdir::WalkDir;

use crate::constants::{BANNED_PACKAGE_NAMES, DEPENDENCY_ALIASES, PACKAGE_VERSION_OVERRIDES};
use crate::domain::{License, PackageMeta, PackageName, WallyLock};

use super::source_utils::{get_file_source, infer_script_license};
use super::thunk_parser::resolve_index_path;
use super::PackageRegistry;

/// Collects all packages in the specified path and adds them to the PackageRegistry.
pub fn populate_package_registry(
    package_registry: &mut PackageRegistry,
    packages_path: &PathBuf,
) -> anyhow::Result<()> {
    let files = get_lua_files_in_path(packages_path)
        .context("Failed to get Lua files in Packages directory")?;

    let mut index_paths = BTreeMap::new();

    for file in files {
        let path = file.path();

        // Resolve where this module is pointing to in the package index
        if let Ok((thunk_name, index_path)) = resolve_index_path(&path, packages_path) {
            let dependencies = resolve_package_dependencies(&index_path, packages_path)
                .context("Failed to parse dependencies")?;

            for (thunk_name, thunk_path) in dependencies {
                if index_paths.contains_key(&thunk_name as &String) {
                    continue;
                }

                populate_index_paths(&mut index_paths, packages_path, thunk_path)?;
            }

            index_paths.insert(thunk_name, index_path);
        }
    }

    for (package_name, index_path) in index_paths {
        if BANNED_PACKAGE_NAMES.contains(&&*package_name)
            && !DEPENDENCY_ALIASES.contains_key(&&*package_name)
        {
            println!("WARN: Found blocked package {package_name}. Skipping.");
            continue;
        }

        let package_lock = parse_package_lock(&index_path).context("Failed to parse lock.toml")?;
        let true_name = package_lock.name.split("/").last().unwrap();

        // Next, work out meta information about the package (LOC, license info).
        let source_path = index_path.join(true_name);
        let (loc, licenses, unlicensed_files) =
            get_package_source_info(&source_path).context("Failed to get package source info")?;

        let dependencies = resolve_package_dependencies(&index_path, packages_path)
            .context("Failed to parse dependencies")?;

        let dependency_thunk_names = dependencies
            .iter()
            .map(|(package_name, path)| {
                if let Some(alias) = DEPENDENCY_ALIASES.get(package_name) {
                    (package_name.to_owned(), alias.to_string())
                } else {
                    let thunk_name = path.file_name().expect("file name").to_str().unwrap();
                    let thunk_name = thunk_name.split(".").next().unwrap();

                    (package_name.to_owned(), thunk_name.to_owned())
                }
            })
            .collect::<BTreeMap<PackageName, String>>();

        let version = if let Some(version) = PACKAGE_VERSION_OVERRIDES.get(&package_name) {
            Version::from_str(version).unwrap()
        } else {
            package_lock.version
        };
        
        let package_meta = PackageMeta {
            thunk_name: PackageName(package_name.clone()),
            true_name: true_name.to_owned(),
            wally_complaint_name: true_name.to_case(Case::Kebab),
            version,
            dependencies: dependencies.into_keys().collect::<Vec<PackageName>>(),
            dependency_thunk_names,
            lines_of_code: loc,
            licenses,
            unlicensed_files,
            package_path: source_path,
        };

        package_registry.add_package(package_meta);
    }

    Ok(())
}

pub fn resolve_package_dependencies(
    package_path: &PathBuf,
    packages_path: &PathBuf,
) -> anyhow::Result<BTreeMap<PackageName, PathBuf>> {
    let mut dependencies = BTreeMap::new();

    let files = get_lua_files_in_path(package_path).context(format!(
        "Failed to get Lua files in package directory: {package_path:?}"
    ))?;

    for file in files {
        let path = file.path();
        if let Ok((package_name, _)) = resolve_index_path(&path, packages_path).context(format!(
            "Failed to resolve _Index path for package {path:?}"
        )) {
            if BANNED_PACKAGE_NAMES.contains(&&*package_name) {
                dependencies.insert(
                    PackageName(package_name),
                    PathBuf::from_str("NO_PATH").unwrap(),
                );

                continue;
            }

            dependencies.insert(PackageName(package_name), path);
        }
    }

    Ok(dependencies)
}

/// Some packages aren't exposed publicly, so iterate through dependencies and add dependency
/// packages recursively
fn populate_index_paths(
    index_paths: &mut BTreeMap<String, PathBuf>,
    packages_path: &PathBuf,
    thunk_path: PathBuf,
) -> anyhow::Result<()> {
    if let Ok((package_name, index_path)) = resolve_index_path(&thunk_path, packages_path) {
        let dependencies = resolve_package_dependencies(&index_path, packages_path)
            .context("Failed to parse dependencies")?;

        for (package_name, thunk_path) in dependencies {
            if index_paths.contains_key(&package_name as &String) {
                continue;
            }

            populate_index_paths(index_paths, packages_path, thunk_path)?;
        }

        index_paths.insert(package_name, index_path);
    }

    Ok(())
}

fn parse_package_lock(package_path: &PathBuf) -> anyhow::Result<WallyLock> {
    let lock_path = package_path.join("lock.toml");
    let lock_content = get_file_source(&lock_path).context("Failed to read lock.toml")?;

    let lock_file =
        toml::from_str::<WallyLock>(&lock_content).context("Failed to parse lock.toml")?;

    Ok(lock_file)
}

fn get_package_source_info(
    source_path: &PathBuf,
) -> anyhow::Result<(usize, Vec<License>, Vec<PathBuf>)> {
    let mut loc = 0;
    let mut licenses = Vec::new();
    let mut unlicensed_files = Vec::new();

    let dir = WalkDir::new(source_path).into_iter().filter_map(|e| e.ok());
    for file in dir {
        let path = file.path();
        let extension = path.extension();

        // We only want to operate on Lua files
        if extension.and_then(OsStr::to_str) != Some("lua") {
            continue;
        }

        let source = get_file_source(path)?;
        loc += source.lines().count();

        let license = infer_script_license(&source, &path.to_owned()).unwrap_or(License::NoLicense);
        if !licenses.contains(&license) {
            licenses.push(license.clone());
        }

        if license == License::NoLicense {
            unlicensed_files.push(path.to_owned());
        }
    }

    Ok((loc, licenses, unlicensed_files))
}

fn get_lua_files_in_path(path: &PathBuf) -> anyhow::Result<Vec<DirEntry>> {
    let mut files = Vec::new();
    for file in fs::read_dir(path)? {
        if let Ok(file) = file {
            let path = file.path();
            let extension = path.extension();

            // We only want to operate on Lua files
            if extension.and_then(OsStr::to_str) != Some("lua") {
                continue;
            }

            files.push(file);
        }
    }

    Ok(files)
}
