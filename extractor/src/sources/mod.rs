mod common;
mod local;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use local::LocalPackageSource;

use crate::domain::PackageName;

#[derive(Debug, Deserialize, Serialize)]
pub enum Source {
    /// Fetch CorePackages from the local studio installation.
    Local,
    // /// Fetch CorePackages from the online Client Tracker.
    // ClientTracker,
}

pub trait CorePackageSource {
    fn extract_packages(
        extract_to: &PathBuf,
        root_packages: &Vec<PackageName>,
    ) -> anyhow::Result<()>;
}
