//! Contains common logic between all CorePackage sources.

pub mod output;
mod package_registry;
pub mod package_resolution;
pub mod source_utils;
pub mod thunk_parser;

pub use package_registry::PackageRegistry;
