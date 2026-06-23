use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use toml::Value as TomlValue;

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
            continue;
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
    let mut excludes = value
        .get("workspace")
        .and_then(|workspace| workspace.get("exclude"))
        .and_then(TomlValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(TomlValue::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if let Some(members) = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(TomlValue::as_array)
    {
        excludes.extend(
            members
                .iter()
                .filter_map(TomlValue::as_str)
                .filter_map(|member| member.strip_prefix('!'))
                .map(str::to_string),
        );
    }
    excludes
}

fn member_manifest_paths_for_pattern(root: &Path, member: &str) -> Result<Vec<PathBuf>> {
    if let Some(prefix) = member.strip_suffix("/**") {
        let parent = root.join(prefix);
        if !parent.is_dir() {
            return Ok(Vec::new());
        }
        return collect_recursive_member_manifests(&parent);
    }

    if let Some(prefix) = member.strip_suffix("/*") {
        let parent = root.join(prefix);
        if !parent.is_dir() {
            return Ok(Vec::new());
        }
        let mut paths = Vec::new();
        for entry in fs::read_dir(&parent).with_context(|| {
            format!(
                "blocked-prewrite-dependency-manifest: failed to read workspace member directory {}",
                parent.display()
            )
        })? {
            let entry = entry?;
            let manifest = entry.path().join("Cargo.toml");
            if manifest.is_file() {
                paths.push(manifest);
            }
        }
        paths.sort();
        return Ok(paths);
    }

    let manifest = root.join(member).join("Cargo.toml");
    Ok(manifest.is_file().then_some(manifest).into_iter().collect())
}

fn collect_recursive_member_manifests(parent: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_recursive_member_manifests_inner(parent, &mut paths)?;
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn collect_recursive_member_manifests_inner(parent: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(parent).with_context(|| {
        format!(
            "blocked-prewrite-dependency-manifest: failed to read workspace member directory {}",
            parent.display()
        )
    })? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name.starts_with('.') || file_name == "target" {
            continue;
        }
        let dir = entry.path();
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            paths.push(manifest);
        }
        collect_recursive_member_manifests_inner(&dir, paths)?;
    }
    Ok(())
}

fn is_excluded_workspace_member(root: &Path, manifest: &Path, excludes: &[String]) -> bool {
    let member_root = manifest.parent().unwrap_or(manifest);
    excludes.iter().any(|exclude| {
        let Some(exclude_root) = workspace_exclude_root(root, exclude) else {
            return false;
        };
        member_root == exclude_root || member_root.starts_with(exclude_root)
    })
}

fn workspace_exclude_root(root: &Path, exclude: &str) -> Option<PathBuf> {
    let normalized = exclude
        .trim_start_matches('!')
        .strip_suffix("/**")
        .or_else(|| exclude.trim_start_matches('!').strip_suffix("/*"))
        .unwrap_or_else(|| exclude.trim_start_matches('!'));
    (!normalized.is_empty()).then(|| root.join(normalized))
}
