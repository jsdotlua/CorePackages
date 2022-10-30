use std::collections::BTreeMap;

use serde::Serialize;

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
