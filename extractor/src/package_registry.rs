use std::{collections::HashMap, fs, path::Path};

use anyhow::Context;
use derive_more::Deref;
use petgraph::{
    dot::{Config, Dot},
    stable_graph::{NodeIndex, StableGraph},
};

use crate::package::{
    package_lock::{LockDependency, PackageVersion},
    Package,
};

/// Numeric reference to a specific package in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref)]
pub struct PackageRef(pub u32);

/// Source of truth for LuaPackages. Maintains a list of all discovered Packages.
pub struct PackageRegistry {
    pub packages: HashMap<PackageRef, Package>,
    pub package_count: u32,
    pub package_graph: StableGraph<PackageRef, ()>,
    pub node_indexes: HashMap<PackageRef, NodeIndex<u32>>,
}

impl PackageRegistry {
    pub fn new() -> anyhow::Result<Self> {
        let package_graph = StableGraph::new();

        Ok(Self {
            packages: HashMap::new(),
            package_count: 0,
            package_graph,
            node_indexes: HashMap::new(),
        })
    }

    /// Find a package in the registry by its path name
    pub fn find_by_path_name(&self, path_name: &str) -> Option<(&PackageRef, &Package)> {
        self.packages
            .iter()
            .find(|(_, package)| package.name.path_name == path_name)
    }

    /// Find a package in the registry by its registry name
    pub fn find_by_registry_name(&self, registry_name: &str) -> Option<(&PackageRef, &Package)> {
        self.packages
            .iter()
            .find(|(_, package)| package.name.registry_name == registry_name)
    }

    /// Find a package in the registry by its registry name and a **specific** version (doesn't do Semver checks)
    pub fn find_by_registry_name_and_version(
        &self,
        registry_name: &str,
        version: &str,
    ) -> Option<(&PackageRef, &Package)> {
        self.packages.iter().find(|(_, package)| {
            package.name.registry_name == registry_name
                && package.lock.version.to_string() == version
        })
    }

    pub fn find_by_commit_hash(&self, commit_hash: &str) -> Option<(&PackageRef, &Package)> {
        self.packages
            .iter()
            .find(|(_, package)| package.lock.commit.starts_with(commit_hash))
    }

    // A best-guess attempt to convert a lock dependency to a package defined in the registry
    pub fn resolve_lock_dependency_to_package(
        &self,
        dependency: &LockDependency,
    ) -> Option<(&PackageRef, &Package)> {
        let first_guess = match &dependency.version {
            PackageVersion::SemVer(version) => self
                .find_by_registry_name_and_version(&dependency.registry_name, &version.to_string()),
            PackageVersion::Commit(version) => self.find_by_commit_hash(&version),
        };

        if let Some(first_guess) = first_guess {
            Some(first_guess)
        } else {
            self.find_by_registry_name(&dependency.registry_name)
        }
    }

    /// Search through an `_Index` directory for packages.
    pub fn discover_packages_at_index(&mut self, index_path: &Path) -> anyhow::Result<()> {
        let entries = fs::read_dir(index_path)
            .context(format!("Failed to read index path: {index_path:?}"))?;

        let start_package_count = self.package_count;

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if !path.is_dir() {
                    // All packages are a directory
                    continue;
                }

                let lock_path = path.join("lock.toml");
                if !lock_path.exists() {
                    log::warn!(
                        "Found directory in _Index that doesn't contain a lock file: {path:?}"
                    );

                    continue;
                }

                // Package constructor can take it from here
                let package = Package::new(path.to_owned())
                    .context(format!("Failed to create package from path {path:?}"))?;

                log::debug!("Discovered package: {}", package.name.path_name);

                self.package_count += 1;

                let package_ref = PackageRef(self.package_count);
                self.packages.insert(package_ref.clone(), package);

                let index = self.package_graph.add_node(package_ref.clone());
                self.node_indexes.insert(package_ref, index);
            }
        }

        log::info!(
            "Discovered {} new packages",
            self.package_count - start_package_count
        );

        Ok(())
    }

    pub fn debug_log_packages(&self) {
        for (_, package) in &self.packages {
            println!(
                "- {}@{}",
                package.name.registry_name,
                package.lock.version.to_string()
            )
        }
    }

    pub fn construct_graph_edges(&mut self) -> anyhow::Result<()> {
        for (package_ref, package) in &self.packages {
            let package_index = self.node_indexes.get(package_ref).unwrap();

            if let Ok(lock_dependencies) = package.lock.parse_lock_dependencies() {
                for dependency in lock_dependencies {
                    let (dependency_ref, _) = self
                        .resolve_lock_dependency_to_package(&dependency)
                        .context(format!(
                            "Dependency {}@{} of package {} does not exist in the registry",
                            dependency.registry_name,
                            dependency.version.to_string(),
                            package.name.path_name
                        ))?;

                    let dependency_index = self.node_indexes.get(dependency_ref).unwrap();
                    self.package_graph
                        .add_edge(*package_index, *dependency_index, ());
                }
            } else {
                log::warn!(
                    "Failed to parse lock dependencies for package {} while building graph edges",
                    package.name.path_name
                );
            }
        }

        Ok(())
    }

    pub fn generate_graphviz_output(&self) -> Dot<'_, &StableGraph<PackageRef, ()>> {
        Dot::with_config(&self.package_graph, &[Config::EdgeNoLabel])
    }
}
