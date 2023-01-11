use std::{fs, path::Path};

use anyhow::{bail, Context};
use semver::Version;
use serde::{Deserialize, Serialize};

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

    /// Stored as a string rather than a Semver `Version` struct because this can either by a standard `Version` or a
    /// Git commit hash. In the case of a commit hash, an error will be thrown later because they are not supported by
    /// Wally. Any dependency with a commit hash need to be overwritten manually.
    pub version: String,
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

                let version = parts.next().context("Expected version")?;

                let dependency = LockDependency {
                    registry_name: registry_name.to_owned(),
                    path_name: path_name.to_owned(),
                    version: version.to_owned(),
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

    use crate::package::package_lock::LockDependency;

    use super::PackageLock;

    #[test]
    fn parses_lock_dependencies() {
        let lock = PackageLock {
            name: "Emittery".into(),
            version: Version::from_str("3.2.1").unwrap(),
            commit: "792ffec6ca98a6d725d25d678d693f486c1d2c75".into(),
            source: "url+https://github.com/roblox/jest-roblox".into(),
            dependencies: Some(vec![
                "LuauPolyfill LuauPolyfill 1.1.0 url+https://github.com/roblox/luau-polyfill".into(),
                "Promise <patched> Promise 8c520dea git+https://github.com/roblox/promise-upgrade#v0.1.0".into(),
            ])
        };

        let deps = lock.parse_lock_dependencies().unwrap();
        assert_eq!(deps.len(), 2);

        let dep_1 = LockDependency {
            registry_name: "LuauPolyfill".into(),
            path_name: "LuauPolyfill".into(),
            version: "1.1.0".into(),
        };

        assert_eq!(*deps.get(0).unwrap(), dep_1);

        let dep_2 = LockDependency {
            registry_name: "Promise".into(),
            path_name: "Promise".into(),
            version: "8c520dea".into(),
        };

        assert_eq!(*deps.get(1).unwrap(), dep_2);
    }
}
