//! Handles generating documentation for core packages.

use anyhow::Context;
use lazy_static::lazy_static;
use tera::{Context as TeraContext, Tera};

use crate::{
    package::{IncludeInEmit, Package},
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
        // First, find all of our fully licensed packages
        let available_packages = registry
            .packages
            .values()
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
