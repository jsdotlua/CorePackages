//! Handles generating documentation for core packages.

use anyhow::Context;
use lazy_static::lazy_static;
use tera::{Context as TeraContext, Tera};

use crate::{
    package::{
        license_extractor::{PackageLicense, ScriptLicense, UnlicensedPackageReason},
        IncludeInEmit, Package,
    },
    package_registry::PackageRegistry,
};

lazy_static! {
    pub static ref TEMPLATES: Tera =
        Tera::new("resources/templates/**/*").expect("valid template file");
}

#[derive(Debug)]
pub struct ReadmeContent<'a> {
    pub available_packages: Vec<&'a Package>,
    pub blocked_packages: Vec<&'a Package>,
    pub blocking_packages: Vec<&'a Package>,
    pub unlicensed_packages: Vec<&'a Package>,
}

impl<'a> ReadmeContent<'a> {
    pub fn new(registry: &'a PackageRegistry) -> anyhow::Result<Self> {
        let mut packages = Vec::from_iter(registry.packages.values());
        packages.sort_by_key(|k| k.name.path_name.to_owned());

        // First, find all of our fully licensed packages
        let available_packages = packages
            .into_iter()
            .filter(|package| {
                package.include_in_extractor_emit(
                    #[cfg(feature = "check-licenses")]
                    registry,
                ) == IncludeInEmit::Included
            })
            .collect::<Vec<&Package>>();

        Ok(Self {
            available_packages,
            blocked_packages: vec![],
            blocking_packages: vec![],
            unlicensed_packages: vec![],
        })
    }
}

pub fn generate_readme(registry: &PackageRegistry) -> anyhow::Result<String> {
    let content = ReadmeContent::new(registry).context("Failed to generate readme content")?;

    let mut context = TeraContext::new();
    context.insert("available_packages", &content.available_packages);
    context.insert("blocked_packages", &content.blocked_packages);
    context.insert("blocking_packages", &content.blocking_packages);
    context.insert("unlicensed_packages", &content.unlicensed_packages);

    let readme_str = TEMPLATES
        .render("README.md", &context)
        .context("Failed to render template")?;

    Ok(readme_str)
}

#[derive(Debug)]
pub struct DebugContent<'a> {
    pub package: &'a Package,
    pub licensed_scripts: Vec<String>,
    pub unlicensed_scripts: Vec<String>,
    pub is_blocked: bool,
    pub blocking_tree: String,
}

impl<'a> DebugContent<'a> {
    pub fn new(registry: &'a PackageRegistry, package_name: &str) -> anyhow::Result<Self> {
        let (_, package) = registry.find_by_path_name(package_name).context(format!(
            "Package name does not exist in registry: {package_name}"
        ))?;

        let mut licensed_scripts = Vec::new();
        let mut unlicensed_scripts = Vec::new();

        #[cfg(feature = "check-licenses")]
        for (license, paths) in &package.licenses {
            if *license == ScriptLicense::Unlicensed {
                for path in paths {
                    unlicensed_scripts.push(path.to_str().unwrap().to_owned());
                }
            } else {
                for path in paths {
                    licensed_scripts.push(path.to_str().unwrap().to_owned());
                }
            }
        }

        let mut is_blocked = false;
        let mut blocking_tree = String::new();

        #[cfg(feature = "check-licenses")]
        if let PackageLicense::Unlicensed(reason) = package.is_package_licensed(&registry)? {
            if let UnlicensedPackageReason::UnlicensedDependencies(deps) = reason {
                is_blocked = true;

                // TODO: Recursively search the tree instead of only doing one-level
                for (dependency, version, _) in deps {
                    blocking_tree
                        .push_str(&format!("- `{dependency}` (`{}`)\n", version.to_string()));
                }
            }
        }

        Ok(Self {
            package,
            licensed_scripts,
            unlicensed_scripts,
            is_blocked,
            blocking_tree,
        })
    }
}

pub fn generate_package_debug(
    registry: &PackageRegistry,
    package_name: &str,
) -> anyhow::Result<String> {
    let content =
        DebugContent::new(registry, package_name).context("Failed to generate debug content")?;

    let mut context = TeraContext::new();
    context.insert("package", content.package);
    context.insert("licensed_scripts", &content.licensed_scripts);
    context.insert("unlicensed_scripts", &content.unlicensed_scripts);
    context.insert("is_blocked", &content.is_blocked);
    context.insert("blocking_tree", &content.blocking_tree);

    let debug_str = TEMPLATES
        .render("PACKAGE_DEBUG.md", &context)
        .context("Failed to render template")?;

    Ok(debug_str)
}
