use anyhow::Context;

use crate::{
    package::{stream::Stream, Package},
    package_registry::{PackageRef, PackageRegistry},
};

use super::stream::Indent;

/// Returns a tree of packages and their dependencies. Includes information about what is licensed or not.
pub fn compute_package_tree(
    package: &Package,
    registry: &PackageRegistry,
) -> anyhow::Result<String> {
    let mut stream = Stream::new();
    stream.indents.push(Indent::Some);

    compute_package_tree_internal(&mut stream, package, registry, &mut Vec::new(), false)?;

    Ok(stream.stream)
}

fn compute_package_tree_internal(
    stream: &mut Stream,
    package: &Package,
    registry: &PackageRegistry,
    previous_packages: &mut Vec<PackageRef>,
    final_package: bool,
) -> anyhow::Result<()> {
    let current_indent = stream.indents.len();

    // {SCOPE}/{PACKAGE_NAME} (v{VERSION})
    let mut line = format!(
        "core-packages/{} (v{})",
        package.name.registry_name,
        package.lock.version.to_string()
    );

    if package.is_package_rewritten() {
        line.push_str(" (rewritten)");
    }

    stream.write_line(&line, final_package);

    if let Ok(dependencies) = package.lock.parse_lock_dependencies() {
        let indent = match final_package {
            true => Indent::None,
            false => Indent::Some,
        };

        *stream.indents.get_mut(current_indent - 1).unwrap() = indent;
        stream.indents.push(Indent::Some);

        let mut dependencies = dependencies.iter().peekable();
        while let Some(dependency) = dependencies.next() {
            if !dependency.is_rewritten {
                // This dependency isn't a core package (probably rewritten), so we can't compute its dependencies.
                // Insert the dependency into the stream and continue
                stream.write_line(
                    &format!(
                        "{} (v{}) (external)",
                        dependency.registry_name, dependency.version,
                    ),
                    false,
                );

                continue;
            }

            let (package_ref, package) = registry
                .find_by_registry_name_and_version(&dependency.registry_name, &dependency.version)
                .context(format!(
                    "Dependency ({} {}) of package {} does not exist in registry",
                    dependency.registry_name, dependency.version, package.name.registry_name
                ))?;

            // If this package has already appeared up the tree then break here because we don't want to end up in a
            // recursive loop.
            if previous_packages.contains(package_ref) {
                break;
            }

            previous_packages.push(package_ref.to_owned());

            // This is the last dependency if there is no next dependency, *or* the next dependency has already appeared
            // up the tree
            let is_last_dependency = if let Some(next_dep) = dependencies.peek() {
                let search_result = registry
                    .find_by_registry_name_and_version(&next_dep.registry_name, &next_dep.version);

                if let Some((dep_ref, _)) = search_result {
                    previous_packages.contains(dep_ref)
                } else {
                    false
                }
            } else {
                true
            };

            compute_package_tree_internal(
                stream,
                &package,
                registry,
                previous_packages,
                is_last_dependency,
            )
            .context(format!(
                "Failed to compute package tree for dependency {}",
                dependency.registry_name
            ))?;
        }

        stream.indents.pop();
    }

    Ok(())
}
