//! Collection of tests that ensure the correctness of the extractor.

use std::path::Path;

use extractor::{package::package_lock::LockDependency, package_registry::PackageRegistry};

const TEST_INDEX_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_index");

fn create_registry() -> PackageRegistry {
    let mut registry = PackageRegistry::new().unwrap();

    registry
        .discover_packages_at_index(&Path::new(TEST_INDEX_PATH))
        .unwrap();

    registry
}

#[test]
fn all_packages_discovered() {
    let registry = create_registry();
    assert_eq!(registry.package_count, 3);
}

#[test]
fn lock_dependencies_are_parsed() {
    let registry = create_registry();

    let (_, diff_sequences) = registry.find_by_registry_name("diff-sequences").unwrap();
    let deps = &diff_sequences.lock.parse_lock_dependencies().unwrap();

    assert_eq!(deps.len(), 1);

    let polyfill_dep = LockDependency {
        registry_name: "LuauPolyfill".into(),
        path_name: "LuauPolyfill".into(),
        version: "1.1.0".into(),
    };

    assert_eq!(*deps.get(0).unwrap(), polyfill_dep);
}

#[test]
fn lock_dependencies_exist_in_registry() {
    let registry = create_registry();

    let (_, diff_sequences) = registry.find_by_registry_name("diff-sequences").unwrap();
    let deps = &diff_sequences.lock.parse_lock_dependencies().unwrap();

    assert_eq!(deps.len(), 1);

    let polyfill_dep = deps.get(0).unwrap();

    let (_, luau_polyfill) = registry
        .find_by_registry_name(&polyfill_dep.registry_name)
        .unwrap();

    assert_eq!(polyfill_dep.registry_name, luau_polyfill.name.registry_name);
    assert_eq!(polyfill_dep.version, luau_polyfill.lock.version.to_string());
}
