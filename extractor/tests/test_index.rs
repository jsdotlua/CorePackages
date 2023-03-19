//! Collection of tests that ensure the correctness of the extractor.

use std::path::Path;

#[cfg(feature = "check-licenses")]
use extractor::package::license_extractor::{
    PackageLicense, ScriptLicense, UnlicensedPackageReason,
};
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
    assert_eq!(registry.package_count, 4);
}

#[test]
fn lock_dependencies_are_parsed() {
    let registry = create_registry();

    let (_, diff_sequences) = registry.find_by_registry_name("diff-sequences").unwrap();
    let deps = &diff_sequences.lock.parse_lock_dependencies().unwrap();

    assert_eq!(deps.len(), 1);

    let polyfill_dep = LockDependency {
        registry_name: "luau-polyfill".into(),
        path_name: "LuauPolyfill".into(),
        version: "1.1.0".into(),
        is_semver_version: true,
        is_rewritten: false,
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

#[test]
#[cfg(feature = "check-licenses")]
fn mit_license_variations_parsed_correctly() {
    let registry = create_registry();

    let (_, chalk) = registry.find_by_registry_name("chalk-lua").unwrap();
    let licenses = &chalk.licenses;

    // Only one license should exist for Chalk
    assert_eq!(licenses.keys().len(), 1);
    assert_eq!(
        licenses.contains_key(&ScriptLicense::Licensed("MIT".into())),
        true
    );

    let (_, polyfill) = registry.find_by_registry_name("luau-polyfill").unwrap();
    let licenses = &polyfill.licenses;

    // Only one license should exist for LuauPolyfill
    assert_eq!(licenses.keys().len(), 1);
    assert_eq!(
        licenses.contains_key(&ScriptLicense::Licensed("MIT".into())),
        true
    );
}

#[test]
#[cfg(feature = "check-licenses")]
fn unlicensed_packages_discovered_correctly() {
    let registry = create_registry();

    let (_, diff_sequences) = registry.find_by_registry_name("diff-sequences").unwrap();
    let licenses = &diff_sequences.licenses;

    // Only one license should exist for DiffSequences
    assert_eq!(licenses.keys().len(), 1);
    assert_eq!(licenses.contains_key(&ScriptLicense::Unlicensed), true);
}

#[test]
#[cfg(feature = "check-licenses")]
fn licensed_package_with_no_deps_is_licensed() {
    let registry = create_registry();

    let (_, polyfill) = registry.find_by_registry_name("luau-polyfill").unwrap();

    assert_eq!(
        polyfill.is_package_licensed(&registry).unwrap(),
        PackageLicense::Licensed(vec!["MIT".into()])
    );
}

#[test]
#[cfg(feature = "check-licenses")]
fn unlicensed_package_not_licensed() {
    let registry = create_registry();

    let (_, diff_sequences) = registry.find_by_registry_name("diff-sequences").unwrap();

    assert_eq!(
        diff_sequences.is_package_licensed(&registry).unwrap(),
        PackageLicense::Unlicensed(UnlicensedPackageReason::UnlicensedScripts(vec![
            "DiffSequences-edcba0e9-2.4.5/src/init.lua".into()
        ]))
    );
}

#[test]
#[cfg(feature = "check-licenses")]
fn licensed_package_with_direct_unlicensed_dependency_is_unlicensed() {
    let registry = create_registry();

    let (_, jest) = registry.find_by_registry_name("jest-circus").unwrap();

    assert_eq!(
        jest.is_package_licensed(&registry).unwrap(),
        PackageLicense::Unlicensed(UnlicensedPackageReason::UnlicensedDependencies(vec![(
            "diff-sequences".into(),
            "2.4.5".into(),
            UnlicensedPackageReason::UnlicensedScripts(vec![
                "DiffSequences-edcba0e9-2.4.5/src/init.lua".into()
            ])
        )]))
    );
}

#[test]
#[cfg(feature = "check-licenses")]
fn licensed_package_with_transient_unlicensed_dependency_is_unlicensed() {
    let registry = create_registry();

    let (_, chalk) = registry.find_by_registry_name("chalk-lua").unwrap();

    assert_eq!(
        chalk.is_package_licensed(&registry).unwrap(),
        PackageLicense::Unlicensed(UnlicensedPackageReason::UnlicensedDependencies(vec![(
            "jest-circus".into(),
            "3.2.1".into(),
            UnlicensedPackageReason::UnlicensedDependencies(vec![(
                "diff-sequences".into(),
                "2.4.5".into(),
                UnlicensedPackageReason::UnlicensedScripts(vec![
                    "DiffSequences-edcba0e9-2.4.5/src/init.lua".into()
                ])
            )])
        )]))
    );
}
