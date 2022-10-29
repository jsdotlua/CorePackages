mod util;

use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use console::style;
use full_moon::ast::Stmt;
use rbx_dom_weak::{Instance, WeakDom};

use util::{
    find_first_child, get_dep_details, get_script_source, match_require, resolve_ref,
    write_instance_to_path,
};

const ROOT_PACKAGES: [&str; 1] = ["ReactRobloxProxy"];
const IGNORED_PACKAGES: [&str; 4] = ["Promise", "roblox_cryo", "Cryo", "Roact17UpgradeFlag"];

// Some core packages are provided as multiple versions, but older versions are not MIT
// Ignore any which aren't MIT to simplify maintenance here
const BANNED_PACKAGE_VERSIONS: [&str; 4] = [
    "LuauPolyfill-12e911c4-90b08185",
    "LuauPolyfill-2fca3173-0.4.2",
    "LuauPolyfill-2fca3173-0.3.4",
    "0fbbfa70", // Old React-related deps
];

fn main() -> anyhow::Result<()> {
    let path = env::current_dir()?.join(Path::new("resources/Packages.rbxm"));
    let packages = BufReader::new(File::open(path)?);

    let dom = rbx_binary::from_reader(packages)?;

    let source_root = dom.root();
    let packages_root = find_first_child(&dom, source_root, "Packages").expect("Packages folder");

    let index = find_first_child(&dom, packages_root, "_Index").expect("_Index folder");

    let entry_packages: Vec<&Instance> = ROOT_PACKAGES
        .into_iter()
        .map(|package_name| find_first_child(&dom, index, package_name).expect("root package"))
        .collect();

    // Recursively collect all package
    let mut package_deps: BTreeMap<String, &Instance> = BTreeMap::new();

    for package in entry_packages {
        resolve_package_deps(&dom, package, index, &mut package_deps);

        let root_module = find_first_child(&dom, package, &package.name).unwrap();
        package_deps.insert(package.name.clone(), root_module);
    }

    output_extracted_deps(&dom, &package_deps);

    let modules = env::current_dir()?.join(Path::new("../modules/"));
    if modules.exists() {
        fs::remove_dir(&modules)?;
    }

    fs::create_dir(&modules)?;

    // Write packages to the file system
    for (name, package) in package_deps {
        let mut package_root = modules.clone();

        package_root.push(name);
        fs::create_dir(&package_root)?;

        package_root.push("src");
        fs::create_dir(&package_root)?;

        write_instance_to_path(true, &package_root, &dom, package)?;
    }

    Ok(())
}

fn resolve_package_deps<'a>(
    dom: &'a WeakDom,
    root: &Instance,
    index: &'a Instance,
    package_deps: &mut BTreeMap<String, &'a Instance>,
) {
    let children = root.children();

    for child in children {
        let child = resolve_ref(dom, child);

        // Package folders should only ever contain modules but check just in case
        if child.class != "ModuleScript" {
            continue;
        }

        // We only want to collect dependencies currently
        if child.name == root.name {
            continue;
        }

        let source = get_script_source(child);

        let ast = full_moon::parse(source).unwrap();
        assert!(ast.nodes().last_stmt().is_some());

        if let Some(Stmt::LocalAssignment(assignment)) = ast.nodes().stmts().nth(1) {
            let require = assignment.expressions().iter().next().unwrap();
            let path_components =
                match_require(require).expect("could not resolve path for require");

            let dep_name: &str = path_components.iter().nth(1).unwrap();

            if IGNORED_PACKAGES.contains(&dep_name) {
                continue;
            }

            // Work out if this dependency is banned
            let mut break_out = false;
            for hash in BANNED_PACKAGE_VERSIONS {
                if dep_name.contains(hash) {
                    break_out = true;
                    continue;
                }
            }

            if break_out {
                continue;
            }

            if package_deps.contains_key(dep_name) {
                continue;
            }

            if let Some(dep) = find_first_child(dom, index, dep_name) {
                let dep_entry =
                    find_first_child(dom, dep, path_components.last().unwrap()).unwrap();

                package_deps.insert(dep_name.to_owned(), dep_entry);
                resolve_package_deps(dom, dep, index, package_deps);
            }
        };
    }
}

fn output_extracted_deps(dom: &WeakDom, deps: &BTreeMap<String, &Instance>) {
    println!("Found {} dependencies:", style(deps.len()).bold().italic());

    let mut bad_deps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total_loc = 0;

    for (key, dep) in deps {
        let details = get_dep_details(dom, dep);

        total_loc += details.loc;

        let loc_text = style(format!("({} LOC)", details.loc)).bold();
        let message = format!("{key} {loc_text}");

        if details.all_licensed {
            println!("\t- {message}");
        } else {
            println!(
                "\t- {} (contains {} unlicensed scripts)",
                style(message).dim(),
                details.bad_scripts.len()
            );
            bad_deps.insert(key.into(), details.bad_scripts);
        }
    }

    println!(
        "\nTotal Lines of Code: {}",
        style(total_loc).bold().italic()
    );

    println!("\nPackages with unlicensed scripts:");
    for (dep, scripts) in bad_deps {
        println!("{}", style(dep).bold().underlined());

        for script in scripts {
            println!("\t- {script}");
        }
    }
}
