use std::path::Path;

use anyhow::Context;
use convert_case::{Case, Casing};

use super::package_lock::PackageLock;

/// There are various different ways a package is named with Roblox's Rotriever. This stores all of them.
#[derive(Debug)]
pub struct PackageName {
    /// The name of the package as it appears on the path. For example, `ChalkLua-198f600a-0.1.3`.
    pub path_name: String,
    /// The 'proper' name of the package as it exists in the Rotriever/Wally registry. For example, `ChalkLua`.
    pub registry_name: String,

    /// Some Rotriever packages are scoped. If this package is scoped then the scope name will be here.
    pub scope: Option<String>,
    /// Some Rotriever packages are scoped. If this package is scoped then the package name will be here.
    pub scoped_name: Option<String>,
}

impl PackageName {
    pub fn new(package_path: &Path, lock: &PackageLock) -> anyhow::Result<Self> {
        let path_name = package_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .context("Failed to get file name from package path")?;

        // Wally requires that all packages are kebab-case
        let name = lock.name.to_case(Case::Kebab);

        // Some Rotriever packages are scoped and others aren't. Wally only supports one scope, and we're using that for
        // `core-packages`. Convert the Rotriever scope into a prefix. For example, `roblox/lumberyak` becomes
        // `roblox-lumberyak`.
        let registry_name = name.replace("/", "-");

        let (scope, scoped_name) = if name.contains("/") {
            let mut segments = name.split("/");

            let scope = segments.next().context("Expected scope")?;
            let scoped_name = segments.next().context("Expected scoped name")?;

            (Some(scope.to_owned()), Some(scoped_name.to_owned()))
        } else {
            (None, None)
        };

        Ok(Self {
            path_name,
            registry_name,

            scope,
            scoped_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, str::FromStr};

    use semver::Version;

    use crate::package::package_lock::PackageLock;

    use super::PackageName;

    fn create_lock_file() -> PackageLock {
        PackageLock {
            name: "roblox/Emittery".into(),
            version: Version::from_str("3.2.1").unwrap(),
            commit: "792ffec6ca98a6d725d25d678d693f486c1d2c75".into(),
            source: "url+https://github.com/roblox/jest-roblox".into(),
            dependencies: Some(vec![
                "LuauPolyfill LuauPolyfill 1.1.0 url+https://github.com/roblox/luau-polyfill".into(),
                "Promise <patched> Promise 8c520dea git+https://github.com/roblox/promise-upgrade#v0.1.0".into(),
            ])
        }
    }

    fn create_package_name() -> PackageName {
        let lock = create_lock_file();
        PackageName::new(Path::new("Emittery-edcba0e9-2.4.1/"), &lock).unwrap()
    }

    #[test]
    fn path_name_correct() {
        let package_name = create_package_name();
        assert_eq!(package_name.path_name, "Emittery-edcba0e9-2.4.1");
    }

    #[test]
    fn registry_name_correct() {
        let package_name = create_package_name();
        assert_eq!(package_name.registry_name, "roblox-emittery");
    }

    #[test]
    fn scope_name_correct() {
        let package_name = create_package_name();
        assert_eq!(package_name.scope.unwrap(), "roblox");
    }

    #[test]
    fn scoped_name_correct() {
        let package_name = create_package_name();
        assert_eq!(package_name.scoped_name.unwrap(), "emittery");
    }
}
