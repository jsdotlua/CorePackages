use std::{collections::BTreeMap, path::PathBuf};

use derive_more::Deref;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::constants::DEPENDENCY_VERSION_ALIASES;

#[derive(Debug, Deref, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum License {
    MIT,
    Apache2,
    NoLicense,
}

impl ToString for License {
    fn to_string(&self) -> String {
        match self {
            License::MIT => "MIT".into(),
            License::Apache2 => "Apache 2.0".into(),
            License::NoLicense => "Unlicensed".into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageMeta {
    /// Name of the package thunk that is required by other packages.
    pub thunk_name: PackageName,
    /// True name of the package, found inside the `lock.toml`.
    pub true_name: String,
    /// Version of `true_name` but Wally compliant.
    pub wally_complaint_name: String,
    /// Version of the package, as found in the `lock.toml` file.
    pub version: Version,
    /// Reference to all dependencies this package defines.
    pub dependencies: Vec<PackageName>,
    /// A map of all dependencies of this package and how they are referred to.
    pub dependency_thunk_names: BTreeMap<PackageName, String>,
    /// Total lines of code in the package, used for statistics.
    pub lines_of_code: usize,
    /// List of all licenses present in package source code, including NoLicense.
    pub licenses: Vec<License>,
    /// List of all source files that do not contain a license header.
    pub unlicensed_files: Vec<PathBuf>,
    /// System path to the original source files.
    pub package_path: PathBuf,
}

impl PackageMeta {
    pub fn contains_unlicensed_code(&self) -> bool {
        // Bypass license check for overwritten dependencies ONLY.
        if DEPENDENCY_VERSION_ALIASES.contains_key(self.thunk_name.as_str()) {
            return false;
        }

        self.licenses.contains(&License::NoLicense)
    }
}

#[derive(Debug, Deserialize)]
pub struct WallyLock {
    pub name: String,
    pub version: Version,
}

pub type WallyDependencies = BTreeMap<String, String>;

#[derive(Debug, Serialize)]
pub struct WallyConfig {
    pub package: WallyConfigPackage,
    pub dependencies: WallyDependencies,
}

#[derive(Debug, Serialize)]
pub struct WallyConfigPackage {
    pub name: String,
    pub description: String,
    pub version: String,
    pub license: String,
    pub authors: Vec<String>,
    pub registry: String,
    pub realm: String,
}
