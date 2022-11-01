use std::fs;
use std::path::{Path, PathBuf};

use crate::constants::{
    ALLOWED_MODULES, APACHE_LICENSE_PHRASES, MIT_LICENSE_PHRASES, SOURCE_REPLACEMENTS,
};
use crate::domain::License;

pub fn infer_script_license(source: &str, path: &PathBuf) -> Option<License> {
    if source_matches_license_list(source, MIT_LICENSE_PHRASES.to_vec()) {
        return Some(License::MIT);
    } else {
        if is_script_whitelisted(path) {
            return Some(License::MIT);
        }
    }

    if source_matches_license_list(source, APACHE_LICENSE_PHRASES.to_vec()) {
        return Some(License::Apache2);
    }

    None
}

/// Returns a files source, supporting manual overrides for file rewrites
pub fn get_file_source(path: &Path) -> anyhow::Result<String> {
    for (replacement_path, content) in SOURCE_REPLACEMENTS.into_iter() {
        let path = path.to_str().unwrap();
        let path = path.replace("\\", "/");

        if path.contains(replacement_path) {
            return Ok((*content).to_owned());
        }
    }

    let mut source = fs::read_to_string(&path)?;
    if is_script_whitelisted(&path.to_owned()) {
        source.insert_str(
            0,
            "-- NOTE: This file is too small and/or simple to be sufficiently rewritten under a new license. Assume MIT.\n",
        );
    }

    Ok(source)
}

fn source_matches_license_list(source: &str, license_list: Vec<&str>) -> bool {
    for phrase in license_list {
        if source.to_lowercase().contains(&phrase.to_lowercase()) {
            return true;
        }
    }

    false
}

fn is_script_whitelisted(path: &PathBuf) -> bool {
    let path = path.to_str().unwrap();
    let path = path.replace("\\", "/");

    for module_path in ALLOWED_MODULES {
        if path.contains(module_path) {
            return true;
        }
    }

    false
}
