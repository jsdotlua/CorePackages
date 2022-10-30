use std::{collections::VecDeque, fs, path::PathBuf};

use convert_case::{Case, Casing};
use full_moon::{
    ast::{Call, Expression, FunctionArgs, Index, Suffix, Value, Var},
    tokenizer::TokenType,
};
use phf::phf_map;
use rbx_dom_weak::{
    types::{Ref, Variant},
    Instance, WeakDom,
};
use serde_json::json;

use crate::wally_types::{WallyConfig, WallyConfigPackage, WallyDependencies};

// Don't ask me why but, while the vast-majority of modules are MIT, there's a small handful
// which are just arbitrarily Apache 2.0, for some reason.
//
// Also a couple of scripts are from Node.js and have their own bespoke license.
const LICENSE_PHRASES: [&str; 3] = [
    "licensed under the MIT license",
    "licensed under the Apache License, Version 2.0",
    "Copyright Node.js contributors. All rights reserved",
];

// Some modules are so small that it's impossible to rewrite them enough to be considered unique.
// Explicitly allow those modules here.
const ALLOWED_MODULES: [&str; 3] = [
    "Packages._Index.Collections.Collections.Map",
    "Packages._Index.Math.Math.clz32",
    "Packages._Index.ReactRoblox-9c8468d8-8a7220fd.ReactRoblox.ReactReconciler.roblox",
];

// We want to manually rename some packages for better discovery
static PACKAGE_NAME_OVERRIDES: phf::Map<&'static str, &'static str> = phf_map! {
    // "RoactCompat" => "Roact17",
};

// Any module that needs to be rewritten should be included here
static SOURCE_REPLACEMENTS: phf::Map<&'static str, &'static str> = phf_map! {
    "Packages._Index.Scheduler-9c8468d8-8a7220fd.Scheduler.getJestMatchers.roblox" =>
        include_str!("../resources/sourceReplacements/getJestMatchers.roblox.lua"),
    "Packages._Index.RoactProxy.RoactProxy" =>
        include_str!("../resources/sourceReplacements/RoactProxy.lua"),
};

pub fn find_first_child<'a>(
    dom: &'a WeakDom,
    parent: &'a Instance,
    name: &str,
) -> Option<&'a Instance> {
    let children = parent.children();
    let len = children.len();

    if len == 0 {
        None
    } else {
        let child_ref = children.iter().find(|child_ref| {
            let instance = resolve_ref(dom, *child_ref);
            instance.name == name
        });

        if let Some(child_ref) = child_ref {
            Some(resolve_ref(dom, child_ref))
        } else {
            None
        }
    }
}

// https://github.com/rojo-rbx/remodel/blob/master/src/roblox_api/instance.rs#L137
pub fn get_full_name(dom: &WeakDom, instance: &Instance) -> String {
    let mut names = vec![instance.name.as_str()];
    let mut current = instance.parent();

    while let Some(parent_instance) = dom.get_by_ref(current) {
        if current != dom.root_ref() && parent_instance.class != "DataModel" {
            names.push(parent_instance.name.as_str());
        }
        current = parent_instance.parent();
    }

    names.reverse();

    names.join(".")
}

pub fn get_script_source<'a>(dom: &'a WeakDom, instance: &'a Instance) -> &str {
    if let Some(source) = SOURCE_REPLACEMENTS.get(&get_full_name(dom, instance)) {
        source
    } else {
        let source = instance
            .properties
            .get("Source")
            .expect("ModuleScript to have Source property");

        match source {
            Variant::String(string) => string,
            _ => unreachable!("ModuleScript should have String Source property"),
        }
    }
}

pub fn resolve_ref<'a>(dom: &'a WeakDom, instance_ref: &'a Ref) -> &'a Instance {
    dom.get_by_ref(*instance_ref).expect("valid ref")
}

// https://github.com/JohnnyMorganz/wally-package-types/blob/master/src/command.rs#L50
pub fn expression_to_components(expression: &Expression) -> Vec<String> {
    let mut components = Vec::new();

    match expression {
        Expression::Value { value, .. } => match &**value {
            Value::Var(Var::Expression(var_expression)) => {
                components.push(var_expression.prefix().to_string().trim().to_string());

                for suffix in var_expression.suffixes() {
                    match suffix {
                        Suffix::Index(index) => match index {
                            Index::Dot { name, .. } => {
                                components.push(name.to_string().trim().to_string());
                            }
                            Index::Brackets { expression, .. } => match expression {
                                Expression::Value { value, .. } => match &**value {
                                    Value::String(name) => match name.token_type() {
                                        TokenType::StringLiteral { literal, .. } => {
                                            components.push(literal.trim().to_string());
                                        }
                                        _ => panic!("non-string brackets index"),
                                    },
                                    _ => panic!("non-string brackets index"),
                                },
                                _ => panic!("non-string brackets index"),
                            },
                            _ => panic!("unknown index"),
                        },
                        _ => panic!("incorrect suffix"),
                    }
                }
            }
            _ => panic!("unknown require expression"),
        },
        _ => panic!("unknown require expression"),
    };

    components
}

// https://github.com/JohnnyMorganz/wally-package-types/blob/master/src/command.rs#L90
pub fn match_require(expression: &Expression) -> Option<Vec<String>> {
    match expression {
        Expression::Value { value, .. } => match &**value {
            Value::FunctionCall(call) => {
                if call.prefix().to_string().trim() == "require" && call.suffixes().count() == 1 {
                    if let Suffix::Call(Call::AnonymousCall(FunctionArgs::Parentheses {
                        arguments,
                        ..
                    })) = call.suffixes().next().unwrap()
                    {
                        if arguments.len() == 1 {
                            return Some(expression_to_components(
                                arguments.iter().next().unwrap(),
                            ));
                        }
                    }
                } else {
                    panic!("unknown require expression");
                }
            }
            _ => panic!("unknown require expression"),
        },
        _ => panic!("unknown require expression"),
    }

    None
}

#[derive(Debug)]
pub struct PackageDetails {
    pub bad_scripts: Vec<String>,
    pub all_licensed: bool,
    pub loc: usize,
}

pub fn get_dep_details(dom: &WeakDom, instance: &Instance) -> PackageDetails {
    let mut all_licensed = true;
    let mut bad_scripts: Vec<String> = Vec::new();

    let (root_loc, _) = get_script_details(dom, instance);
    let mut total_loc: usize = root_loc;

    let mut stack = VecDeque::from_iter(instance.children().into_iter());
    while let Some(current) = stack.pop_front() {
        let current_instance = resolve_ref(dom, current);

        if current_instance.class == "ModuleScript" {
            let (current_loc, current_licensed) = get_script_details(dom, current_instance);

            total_loc += current_loc;
            if current_licensed == false {
                bad_scripts.push(get_full_name(dom, current_instance));
                all_licensed = false
            }
        }

        for child in current_instance.children().iter().rev() {
            stack.push_front(child);
        }
    }

    PackageDetails {
        all_licensed,
        bad_scripts,
        loc: total_loc,
    }
}

pub fn get_script_details(dom: &WeakDom, instance: &Instance) -> (usize, bool) {
    let full_name = get_full_name(dom, instance);

    let source = get_script_source(dom, instance);

    let loc = source.lines().count();

    let licensed = if ALLOWED_MODULES.contains(&full_name.as_str()) {
        true
    } else {
        let mut licensed = false;
        for phrase in LICENSE_PHRASES {
            if source.to_lowercase().contains(&phrase.to_lowercase()) {
                licensed = true;
                continue;
            }
        }
        licensed
    };

    (loc, licensed)
}

pub fn build_project_file(package_name: &str) -> String {
    let package_name = fix_package_name(package_name);
    let package_name = package_name.to_case(Case::Kebab);

    let project = json!({
        "name": package_name,
        "tree": {
            "$path": "src/"
        }
    });

    serde_json::to_string_pretty(&project).unwrap()
}

pub fn build_wally_manifest(
    package_name: &str,
    package_version: &str,
    package_deps: &WallyDependencies,
) -> String {
    let package_name = fix_package_name(package_name);
    let package_name = package_name.to_case(Case::Kebab);

    let package = WallyConfig {
        package: WallyConfigPackage {
            name: format!("core-packages/{package_name}"),
            description: "https://github.com/grilme99/CorePackages".into(),
            version: package_version.into(),
            license: "MIT + Apache 2.0".into(),
            authors: vec!["Roblox".into(), "Brooke Rhodes <brooke@gril.me>".into()],
            registry: "https://github.com/UpliftGames/wally-index".into(),
            realm: "shared".into(),
        },
        dependencies: package_deps.to_owned(),
    };

    toml::to_string_pretty(&package).unwrap()
}

const IGNORED_INSTANCE_NAMES: [&str; 1] = [".robloxrc"];

/// Recursively write an instance and all of its descendants to the file system
pub fn write_instance_to_path(
    is_start: bool,
    root: &PathBuf,
    dom: &WeakDom,
    instance: &Instance,
) -> anyhow::Result<()> {
    // There is always a root ModuleScript of a package, throw its contents into a child `init.lua` file
    if is_start {
        let mut path = root.clone();
        path.push("init.lua");

        let source = get_script_source(dom, instance);
        fs::write(path, source)?;
    }

    let children = instance.children();
    for child in children {
        let child = resolve_ref(dom, child);

        if IGNORED_INSTANCE_NAMES.contains(&child.name.as_str()) {
            continue;
        }

        match child.class.as_str() {
            "Script" | "LocalScript" | "ModuleScript" => {
                let source = get_script_source(dom, child);
                let file_name = format!(
                    "{}{}",
                    child.name,
                    get_script_extension_from_class(&child.class)
                );

                // Scripts can also include children!
                // If there are children then treat it like a folder and also insert a `init.lua` file
                let path = if child.children().len() > 0 {
                    let mut path = handle_folder_case(root, dom, child)?;
                    path.push("init.lua");
                    path
                } else {
                    // Not sure how to do this cleaner
                    let mut path = root.clone();
                    path.push(file_name);
                    path
                };

                fs::write(path, source)?;
            }
            "Folder" => {
                handle_folder_case(root, dom, child)?;
            }
            _ => panic!(
                "Got unexpected instance type '{}' at {}",
                child.class,
                get_full_name(dom, child)
            ),
        }
    }

    Ok(())
}

fn handle_folder_case(root: &PathBuf, dom: &WeakDom, child: &Instance) -> anyhow::Result<PathBuf> {
    // Not sure how to do this cleaner
    let mut path = root.clone();
    path.push(&child.name);

    fs::create_dir(&path)?;

    write_instance_to_path(false, &path, dom, child)?;

    Ok(path)
}

fn get_script_extension_from_class(class: &str) -> &str {
    match class {
        "Script" => ".server.lua",
        "LocalScript" => ".client.lua",
        "ModuleScript" => ".lua",
        _ => unreachable!(),
    }
}

pub fn fix_package_name(name: &str) -> &str {
    let name = PACKAGE_NAME_OVERRIDES.get(name).unwrap_or(&name);

    // Anything after a `-` in the package name is a version hash, which we don't want
    name.split("-").next().unwrap()
}
