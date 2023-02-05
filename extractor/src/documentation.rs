//! Handles generating documentation for core packages.

use anyhow::Context;
use lazy_static::lazy_static;
use tera::{Context as TeraContext, Tera};

use crate::{
    package::Package,
    package_registry::{PackageRef, PackageRegistry},
};

lazy_static! {
    pub static ref TEMPLATES: Tera = Tera::new("resources/templates/**/*").unwrap();
}

#[derive(Debug)]
pub struct ReadmeContent {
    pub available_packages: Vec<PackageRef>,
    pub blocked_packages: Vec<PackageRef>,
    pub blocking_packages: Vec<PackageRef>,
    pub unlicensed_packages: Vec<PackageRef>,
}

pub fn generate_readme(
    registry: &PackageRegistry,
    content: &ReadmeContent,
) -> anyhow::Result<String> {
    let available_packages = content
        .available_packages
        .iter()
        .map(|i| registry.packages.get(&i).unwrap())
        .collect::<Vec<&Package>>();

    let blocked_packages = content
        .blocked_packages
        .iter()
        .map(|i| registry.packages.get(&i).unwrap())
        .collect::<Vec<&Package>>();

    let blocking_packages = content
        .blocking_packages
        .iter()
        .map(|i| registry.packages.get(&i).unwrap())
        .collect::<Vec<&Package>>();

    let unlicensed_packages = content
        .unlicensed_packages
        .iter()
        .map(|i| registry.packages.get(&i).unwrap())
        .collect::<Vec<&Package>>();

    let mut context = TeraContext::new();
    context.insert("available_packages", &available_packages);
    context.insert("blocked_packages", &blocked_packages);
    context.insert("blocking_packages", &blocking_packages);
    context.insert("unlicensed_packages", &unlicensed_packages);

    let readme_str = TEMPLATES
        .render("README.md", &context)
        .context("Failed to render template")?;

    Ok(readme_str)
}
