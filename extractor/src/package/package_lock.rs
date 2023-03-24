use std::{fs, path::Path, str::FromStr};

use anyhow::{bail, Context};
use derive_more::Deref;
use semver::Version;
use serde::{Deserialize, Serialize};

use super::{package_name::format_registry_name, package_rewrite::resolve_package};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Deref)]
pub struct CommitHash(String);

/// Rotriever stores package versions as either a commit hash or a SemVer version.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageVersion {
    SemVer(Version),
    Commit(CommitHash),
}

impl PackageVersion {
    pub fn new(raw_version: &str) -> Self {
        if let Ok(version) = Version::from_str(raw_version) {
            PackageVersion::SemVer(version)
        } else {
            PackageVersion::Commit(CommitHash(raw_version.to_owned()))
        }
    }
}

impl ToString for PackageVersion {
    fn to_string(&self) -> String {
        match self {
            PackageVersion::SemVer(version) => version.to_string(),
            PackageVersion::Commit(commit) => commit.0.to_owned(),
        }
    }
}

/// An easy to consume representation of dependencies in a lock.toml file.
#[derive(Debug, PartialEq, Eq)]
pub struct LockDependency {
    /// Name of the dependency as it exists in the Wally and Rotriever registry.
    pub registry_name: String,

    /// Name of the dependency as it appears on path. **This is important** because it's how the dependencies are
    /// referenced from Lua. For example, this will be used in the `wally.toml` like so:
    ///
    /// ```toml
    /// PATH_NAME = "SCOPE/REGISTRY_NAME@VERSION"
    /// ```
    pub path_name: String,

    /// The package version of this dependency.
    pub version: PackageVersion,

    /// Indicates that the original package has been rewritten with another.
    pub is_rewritten: bool,
}

/// Raw Rotriever lock file as it appears on disk. Has associated utility for parsing dependencies.
#[derive(Debug, Deserialize, Serialize)]
pub struct PackageLock {
    pub name: String,
    pub version: Version,
    pub commit: String,
    pub source: String,
    pub dependencies: Option<Vec<String>>,
}

impl PackageLock {
    pub fn new(lock_path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(&lock_path)?;
        let lock = toml::from_str(&content).context("Failed to parse lock to TOML")?;

        Ok(lock)
    }

    /// Each dependency in a Rotriever lock file are represented as one string with lots of information. This method
    /// parses each string into an easily consumable struct.
    pub fn parse_lock_dependencies(&self) -> anyhow::Result<Vec<LockDependency>> {
        if let Some(dependencies) = &self.dependencies {
            let mut dependency_list = Vec::with_capacity(dependencies.len());

            for dep_string in dependencies {
                let mut parts = dep_string.split(" ");

                let path_name = parts.next().context("Expected path name")?;
                let mut registry_name = parts.next().context("Expected registry name")?;

                if registry_name == "<patched>" {
                    registry_name = parts
                        .next()
                        .context("Expected registry name after <patched>")?;
                }

                let raw_version = parts.next().context("Expected version")?;
                let version = PackageVersion::new(&raw_version);

                // We have special behaviour for parsing registry names
                let registry_name = format_registry_name(&registry_name);

                // Resolve package overwrites
                let (rewritten, registry_name, version) =
                    resolve_package(&registry_name, &version.to_string());

                let version = PackageVersion::new(&version);

                let dependency = LockDependency {
                    registry_name,
                    path_name: path_name.to_owned(),
                    version,
                    is_rewritten: rewritten,
                };

                dependency_list.push(dependency);
            }

            Ok(dependency_list)
        } else {
            bail!("Lock has no dependencies defined");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use semver::Version;

    use crate::package::package_lock::{LockDependency, PackageVersion};

    use super::PackageLock;

    fn make_lock() -> PackageLock {
        PackageLock {
            name: "Emittery".into(),
            version: Version::from_str("3.2.1").unwrap(),
            commit: "792ffec6ca98a6d725d25d678d693f486c1d2c75".into(),
            source: "url+https://github.com/roblox/jest-roblox".into(),
            dependencies: Some(vec![
                "LuauPolyfill LuauPolyfill 1.1.0 url+https://github.com/roblox/luau-polyfill".into(),
                "Promise <patched> Promise 8c520dea git+https://github.com/roblox/promise-upgrade#v0.1.0".into(),
            ])
        }
    }

    #[test]
    fn parses_lock_dependencies() {
        let lock = make_lock();
        let deps = lock.parse_lock_dependencies().unwrap();
        assert_eq!(deps.len(), 2);

        let dep = LockDependency {
            registry_name: "luau-polyfill".into(),
            path_name: "LuauPolyfill".into(),
            version: PackageVersion::new("1.1.0"),
            is_rewritten: false,
        };

        assert_eq!(*deps.get(0).unwrap(), dep);
    }

    #[test]
    fn rewrites_package_dependencies() {
        let lock = make_lock();
        let deps = lock.parse_lock_dependencies().unwrap();
        assert_eq!(deps.len(), 2);

        let dep = LockDependency {
            registry_name: "evaera/promise".into(),
            path_name: "Promise".into(),
            version: PackageVersion::new("4.0.0"),
            is_rewritten: true,
        };

        assert_eq!(*deps.get(1).unwrap(), dep);
    }
}
