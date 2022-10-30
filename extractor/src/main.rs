mod util;
mod wally_types;

use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use console::style;
use convert_case::{Case, Casing};
use full_moon::ast::Stmt;
use rbx_dom_weak::{Instance, WeakDom};

use util::{
    build_project_file, build_wally_manifest, find_first_child, fix_package_name, get_dep_details,
    get_script_source, match_require, resolve_ref, write_instance_to_path,
};
use wally_types::WallyDependencies;

// TODO: Come up with a better way to resolve the version of individual packages
const CORE_PACKAGES_VERSION: &str = "0.0.1";

const ROOT_PACKAGES: [&str; 1] = ["RoactProxy"];
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

    // Recursively collect all packages that our root packages depend on
    let mut relevant_packages: BTreeMap<String, &Instance> = BTreeMap::new();
    let mut package_dependencies: HashMap<String, WallyDependencies> = HashMap::new();

    for package in entry_packages {
        resolve_package_deps(
            &dom,
            package,
            index,
            &mut relevant_packages,
            &mut package_dependencies,
        );

        let root_module = find_first_child(&dom, package, &package.name).unwrap();
        relevant_packages.insert(package.name.clone(), root_module);
    }
    output_extracted_deps(&dom, &relevant_packages);

    let modules = env::current_dir()?.join(Path::new("../modules/"));
    if modules.exists() {
        fs::remove_dir(&modules)?;
    }

    fs::create_dir(&modules)?;

    // Write packages to the file system
    for (name, package) in relevant_packages {
        let mut package_root = modules.clone();

        package_root.push(fix_package_name(&name));
        fs::create_dir(&package_root)?;

        let dependencies = package_dependencies.get(&name).unwrap();
        write_package_meta_files(&name, dependencies, &package_root)?;

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
    relevant_packages: &mut BTreeMap<String, &'a Instance>,
    package_deps: &mut HashMap<String, WallyDependencies>,
) {
    let root_name = &root.name;
    let children = root.children();

    if package_deps.get(root_name).is_none() {
        package_deps.insert(root_name.to_owned(), BTreeMap::new());
    }

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

        let source = get_script_source(dom, child);

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

            // Add this dependency to the list of deps for the root package
            let deps = package_deps.get_mut(root_name).unwrap();

            let wally_name = fix_package_name(dep_name).to_string();
            let version = format!(
                "core-packages/{}@{}",
                wally_name.to_case(Case::Kebab),
                CORE_PACKAGES_VERSION
            );

            deps.insert(wally_name, version);

            if relevant_packages.contains_key(dep_name) {
                continue;
            }

            if let Some(dep) = find_first_child(dom, index, dep_name) {
                let dep_entry =
                    find_first_child(dom, dep, path_components.last().unwrap()).unwrap();

                relevant_packages.insert(dep_name.to_owned(), dep_entry);
                resolve_package_deps(dom, dep, index, relevant_packages, package_deps);
            }
        };
    }
}

fn write_package_meta_files(
    package_name: &str,
    package_deps: &WallyDependencies,
    package_root: &PathBuf,
) -> anyhow::Result<()> {
    let mut package_root = package_root.clone();
    package_root.push("default.project.json");

    let project_file = build_project_file(package_name);
    fs::write(&package_root, project_file)?;

    package_root.pop();
    package_root.push("wally.toml");

    let wally_file = build_wally_manifest(package_name, CORE_PACKAGES_VERSION, package_deps);
    fs::write(&package_root, wally_file)?;

    Ok(())
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
