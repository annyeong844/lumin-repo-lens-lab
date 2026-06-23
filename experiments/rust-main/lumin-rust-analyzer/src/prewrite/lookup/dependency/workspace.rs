use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use toml::Value as TomlValue;

mod glob;

use glob::{member_manifest_paths_for_pattern, workspace_member_root_matches};

pub(super) fn workspace_member_manifest_paths(
    root: &Path,
    value: &TomlValue,
) -> Result<Vec<PathBuf>> {
    let Some(members) = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(TomlValue::as_array)
    else {
        return Ok(Vec::new());
    };
    let excludes = workspace_exclude_patterns(value);
    let mut paths = BTreeSet::new();
    for member in members.iter().filter_map(TomlValue::as_str) {
        if member.starts_with('!') {
            bail!(
                "blocked-prewrite-dependency-manifest: workspace.members does not support negated member '{member}'; use workspace.exclude"
            );
        }
        for manifest in member_manifest_paths_for_pattern(root, member)? {
            if !is_excluded_workspace_member(root, &manifest, &excludes) {
                paths.insert(manifest);
            }
        }
    }
    Ok(paths.into_iter().collect())
}

fn workspace_exclude_patterns(value: &TomlValue) -> Vec<String> {
    value
        .get("workspace")
        .and_then(|workspace| workspace.get("exclude"))
        .and_then(TomlValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(TomlValue::as_str)
        .map(str::to_string)
        .collect()
}

fn is_excluded_workspace_member(root: &Path, manifest: &Path, excludes: &[String]) -> bool {
    let member_root = manifest.parent().unwrap_or(manifest);
    excludes
        .iter()
        .any(|exclude| workspace_member_root_matches(root, member_root, exclude))
}
