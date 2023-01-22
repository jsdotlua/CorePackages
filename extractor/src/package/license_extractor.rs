use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use askalono::TextData;
use lazy_static::lazy_static;

/// Minimum match score for the script license to be considered valid
const LICENSE_SCORE_THRESHOLD: f32 = 0.95;

const RAW_LICENSE_DATASET: &'static str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/datasets/license_headers.json"
));

type LicenseDataset = HashMap<String, Vec<String>>;
type LicenseTextData = HashMap<String, Vec<TextData>>;

lazy_static! {
    static ref LICENSE_DATASET: LicenseDataset =
        serde_json::from_str(&RAW_LICENSE_DATASET).unwrap();
}

lazy_static! {
    static ref LICENSE_TEXTS: LicenseTextData = {
        LICENSE_DATASET
            .to_owned()
            .into_iter()
            .map(|(license_name, data)| {
                let data = data
                    .into_iter()
                    .map(|string| TextData::from(string))
                    .collect();

                (license_name, data)
            })
            .collect()
    };
}

/// Described the license status of one script.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScriptLicense {
    Licensed(String),
    Unlicensed,
}

/// Describes the license status of an entire package.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageLicense {
    /// This entire package is appropriately licensed. Enum contains a vector of all licenses found.
    Licensed(Vec<String>),
    /// For some reason the package is unlicensed. Enum contains a reason why.
    Unlicensed(UnlicensedPackageReason),
}

/// Describes why exactly a package is unlicensed.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnlicensedPackageReason {
    /// One or more scripts in the package is unlicensed. Enum contains a vector of all script paths that are unlicensed.
    UnlicensedScripts(Vec<PathBuf>),
    /// One or more dependencies are unlicensed. Enum contains a vector of all dependencies, their version, and its package
    /// license that are not licensed.
    UnlicensedDependencies(Vec<(String, String, UnlicensedPackageReason)>),
}

pub type ScriptLicenses = std::collections::BTreeMap<ScriptLicense, Vec<PathBuf>>;

/// Walks through all source files in the directory and computes license information.
pub fn compute_license_information(src_path: &Path) -> anyhow::Result<ScriptLicenses> {
    let mut licenses: BTreeMap<ScriptLicense, Vec<PathBuf>> = BTreeMap::new();

    let package_path = src_path.parent().context("Src path contains no parent")?;

    for entry in walkdir::WalkDir::new(src_path) {
        if let Ok(entry) = entry {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // We only care about Lua and Luau files right now
            if let Some(extension) = path.extension() {
                if !(extension == "lua" || extension == "luau") {
                    continue;
                }
            }

            let script_source = fs::read_to_string(path)
                .context(format!("Failed to read script to string: {path:?}"))?;

            // Make a best-effort to find the license header in the script and only match against that when detecting
            // the license.
            let license_header = extract_license_header(&script_source);

            let license = if license_header.is_empty() {
                // No license header, this script is probably unlicensed
                ScriptLicense::Unlicensed
            } else {
                // Script has a license header
                compute_header_license(&license_header)
            };

            // Slice off the first part of the path so that it only includes beyond the package root
            let component_count = package_path.components().count();
            let path = path
                .components()
                .skip(component_count - 1)
                .map(|i| i.as_os_str())
                .collect::<PathBuf>();

            if let Some(license_record) = licenses.get_mut(&license) {
                license_record.push(path.to_owned());
            } else {
                licenses.insert(license, vec![path.to_owned()]);
            }
        }
    }

    Ok(licenses)
}

fn compute_header_license(license_header: &str) -> ScriptLicense {
    let header_text_data = TextData::from(license_header);

    let mut top_license = ScriptLicense::Unlicensed;
    let mut highest_score = LICENSE_SCORE_THRESHOLD;

    for (license_name, texts) in LICENSE_TEXTS.iter() {
        for text in texts {
            let (_, score) = header_text_data.optimize_bounds(text);

            if score > highest_score {
                top_license = ScriptLicense::Licensed(license_name.to_owned());
                highest_score = score;
            }
        }
    }

    top_license
}

/// Attempts to best extract a scripts license header
fn extract_license_header(source: &str) -> String {
    let mut license_parts = Vec::new();
    let mut lines = source.lines();

    // Find the start of the license definition
    let mut found_start = false;
    while let Some(next_line) = lines.next() {
        let next_line = trim_comment_padding(next_line);

        // Pretty much all scripts starts with `local` and `type` definitions in LuaPackages. This isn't very
        // bulletproof, but it does the job.
        let is_code_line = next_line.starts_with("local") || next_line.starts_with("type");

        if !found_start {
            if is_code_line {
                // Code lines before we've even found a license header means the code is probably unlicensed.
                // Stop searching.
                break;
            }

            // It's a little hard to 100% determine if a comment is the start of a script license, but we can make a
            // best-guess. In most LuaPackages, the script will either start with the license header or a `-- ROBLOX
            // upstream` comment. If we check the comment *isn't* an upstream comment, then it's *probably* a license.

            let is_upstream = next_line.to_lowercase().contains("upstream");
            let is_empty = next_line.is_empty();

            let is_license_start = !is_upstream && !is_empty;
            if is_license_start {
                found_start = true;
                license_parts.push(next_line);
            }
        } else {
            // Just keep adding lines to the list until we reach the end of the comment
            let is_end = next_line.contains("]]") || is_code_line;

            if !is_end {
                license_parts.push(next_line);
            } else {
                // We reached the end of the license header
                break;
            }
        }
    }

    license_parts.join("\n").trim().to_owned()
}

fn trim_comment_padding(comment: &str) -> String {
    comment
        .replace("*", "")
        .replace("/", "")
        .replace("\\", "")
        .replace("--[[", "")
        .replace("--", "")
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::extract_license_header;

    #[test]
    fn extracts_multiline_license_header() {
        let source = "--[[
* Copyright (c) GraphQL Contributors
*
* This source code is licensed under the MIT license found in the
* LICENSE file in the root directory of this source tree.
]]
-- ROBLOX upstream: https://github.com/graphql/graphql-js/blob/00d4efea7f5b44088356798afff0317880605f4d/src/execution/index.js

local executeModule = require(script.execute)
local valuesModule = require(script.values)";

        let header = extract_license_header(source);

        assert_eq!(
            header,
            "Copyright (c) GraphQL Contributors

This source code is licensed under the MIT license found in the
LICENSE file in the root directory of this source tree."
        );
    }

    #[test]
    fn extracts_js_comment_license_header() {
        let source = "-- ROBLOX upstream: https://github.com/facebook/jest/blob/v27.4.7/packages/jest-diff/src/diffStrings.ts
-- /**
--  * Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved.
--  *
--  * This source code is licensed under the MIT license found in the
--  * LICENSE file in the root directory of this source tree.
--  */

local CurrentModule = script.Parent
local Packages = CurrentModule.Parent";

        let header = extract_license_header(source);

        assert_eq!(
            header,
            "Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved.

This source code is licensed under the MIT license found in the
LICENSE file in the root directory of this source tree."
        )
    }

    #[test]
    fn unlicensed_code_has_empty_header() {
        let source = "local CurrentModule = script.Parent
local Packages = CurrentModule.Parent
local LuauPolyfill = require(Packages.LuauPolyfill)
type Array<T> = LuauPolyfill.Array<T>

local diffSequences = require(Packages.DiffSequences)

local CleanupSemantic = require(CurrentModule.CleanupSemantic)
local DIFF_DELETE = CleanupSemantic.DIFF_DELETE
local DIFF_EQUAL = CleanupSemantic.DIFF_EQUAL
local DIFF_INSERT = CleanupSemantic.DIFF_INSERT
local Diff = CleanupSemantic.Diff
type Diff = CleanupSemantic.Diff";

        let header = extract_license_header(source);
        assert_eq!(header.is_empty(), true)
    }
}
