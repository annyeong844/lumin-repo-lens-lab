use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use toml::Value as TomlValue;

mod glob;

use glob::member_manifest_paths_for_pattern;

pub(super) fn workspace_member_manifest_paths(
    root: &Path,
    value: &TomlValue,
) -> Result<Vec<PathBuf>> {
    let Some(workspace) = workspace_table(value)? else {
        return Ok(Vec::new());
    };
    let Some(members) = workspace_string_array(workspace, "members")? else {
        return Ok(Vec::new());
    };
    let excludes = workspace_exclude_patterns(workspace)?;
    let mut paths = BTreeSet::new();
    for member in members {
        if member.starts_with('!') {
            bail!(
                "blocked-prewrite-dependency-manifest: workspace.members does not support negated member '{member}'; use workspace.exclude"
            );
        }
        for manifest in member_manifest_paths_for_pattern(root, member, &excludes)? {
            paths.insert(manifest);
        }
    }
    Ok(paths.into_iter().collect())
}

pub(super) fn workspace_exclude_roots(value: &TomlValue) -> Result<Vec<String>> {
    let Some(workspace) = workspace_table(value)? else {
        return Ok(Vec::new());
    };
    workspace_exclude_patterns(workspace)
}

fn workspace_exclude_patterns(
    workspace: &toml::map::Map<String, TomlValue>,
) -> Result<Vec<String>> {
    Ok(workspace_string_array(workspace, "exclude")?
        .unwrap_or_default()
        .into_iter()
        .map(normalize_workspace_path)
        .collect())
}

fn normalize_workspace_path(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .collect::<Vec<_>>()
        .join("/")
}

fn workspace_table(value: &TomlValue) -> Result<Option<&toml::map::Map<String, TomlValue>>> {
    let Some(workspace) = value.get("workspace") else {
        return Ok(None);
    };
    let Some(table) = workspace.as_table() else {
        bail!("blocked-prewrite-dependency-manifest: workspace must be a table");
    };
    Ok(Some(table))
}

fn workspace_string_array<'a>(
    workspace: &'a toml::map::Map<String, TomlValue>,
    field: &str,
) -> Result<Option<Vec<&'a str>>> {
    let Some(value) = workspace.get(field) else {
        return Ok(None);
    };
    let Some(entries) = value.as_array() else {
        bail!(
            "blocked-prewrite-dependency-manifest: workspace.{field} must be an array of strings"
        );
    };
    let mut strings = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let Some(text) = entry.as_str() else {
            bail!(
                "blocked-prewrite-dependency-manifest: workspace.{field}[{index}] must be a string"
            );
        };
        strings.push(text);
    }
    Ok(Some(strings))
}
