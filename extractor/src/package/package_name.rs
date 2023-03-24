use std::path::Path;

use anyhow::Context;
use convert_case::{Case, Casing};
use serde::Serialize;

/// Formats any arbitrary string into a Wally registry-compatible string. Some Rotriever packages are scoped and others
/// aren't. Wally only supports one scope, and we're using that for `core-packages`. Convert the Rotriever scope into a
/// prefix. For example, `roblox/lumberyak` becomes `core-packages/roblox-lumberyak`.
pub fn format_registry_name(name: &str) -> String {
    let name = name.to_case(Case::Kebab);
    name.replace("/", "-")
}

/// There are various different ways a package is named with Roblox's Rotriever. This stores all of them.
#[derive(Debug, Serialize)]
pub struct PackageName {
    /// The name of the package as it appears on the path. For example, `ChalkLua-198f600a-0.1.3`.
    pub path_name: String,
    /// The 'proper' name of the package as it exists in the Rotriever/Wally registry. For example, `ChalkLua`.
    pub registry_name: String,

    /// Some Rotriever packages are scoped. If this package is scoped then the scope name will be here.
    pub scope: Option<String>,
    /// Some Rotriever packages are scoped. If this package is scoped then the package name will be here.
    pub scoped_name: Option<String>,

    /// The original name of the package before we did any processing to it.
    pub unprocessed_name: String,
}

impl PackageName {
    pub fn new(package_path: &Path, lock_name: &str) -> anyhow::Result<Self> {
        let path_name = package_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .context("Failed to get file name from package path")?;

        // Wally requires that all packages are kebab-case
        let name = lock_name.to_case(Case::Kebab);

        let registry_name = format_registry_name(&name);

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

            unprocessed_name: lock_name.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::PackageName;

    fn create_package_name() -> PackageName {
        PackageName::new(Path::new("Emittery-edcba0e9-2.4.1/"), "roblox/Emittery").unwrap()
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
