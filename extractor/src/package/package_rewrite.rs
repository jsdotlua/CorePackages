use std::collections::HashMap;

use lazy_static::lazy_static;
use semver::Version;
use serde::Deserialize;

#[cfg(not(test))]
const RAW_PACKAGE_REWRITES: &'static str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/package_rewrites.json"
));

#[cfg(test)]
lazy_static! {
    /// Mocked rewrites for stable testing
    static ref RAW_PACKAGE_REWRITES: String = serde_json::json!({
        "Promise": {
            "newSource": "evaera/promise",
            "newVersion": "4.0.0",
            "originals": [
            {
                "name": "promise",
                "version": "8c520dea",
                "description": "Roblox's internal (unlicensed) Promise upgrade flag package, lives at https://github.com/roblox/promise-upgrade"
            }
            ]
        }
    }).to_string();
}

lazy_static! {
    static ref PACKAGE_REWRITES: HashMap<String, PackageRewrite> =
        serde_json::from_str(&RAW_PACKAGE_REWRITES).unwrap();
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageRewrite {
    pub new_source: String,
    pub new_version: Version,
    pub originals: Vec<OriginalPackage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalPackage {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Get back a package name and version that may or may not have been rewritten to something different.
pub fn resolve_package(name: &str, version: &str) -> (bool, String, String) {
    // Search through all package rewrites and see if one matches
    for (_, rewrite) in PACKAGE_REWRITES.iter() {
        let original = rewrite
            .originals
            .iter()
            .find(|i| i.name == name && i.version == version);

        if original.is_some() {
            // We found a rewrite!
            return (
                true,
                rewrite.new_source.to_owned(),
                rewrite.new_version.to_string(),
            );
        }
    }

    // No rewrite necessary
    (false, name.to_owned(), version.to_owned())
}

#[cfg(test)]
mod tests {
    use super::resolve_package;

    #[test]
    fn packages_are_rewritten() {
        let (rewritten, name, version) = resolve_package("promise", "8c520dea");

        assert_eq!(rewritten, true);
        assert_eq!(name, "evaera/promise");
        assert_eq!(version, "4.0.0");
    }

    #[test]
    fn untouched_packages_are_left_alone() {
        let (rewritten, name, version) = resolve_package("some-fake-package", "1.0.0");

        assert_eq!(rewritten, false);
        assert_eq!(name, "some-fake-package");
        assert_eq!(version, "1.0.0");
    }
}
