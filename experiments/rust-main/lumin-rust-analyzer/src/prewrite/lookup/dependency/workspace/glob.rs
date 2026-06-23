use std::path::{Path, PathBuf};

use anyhow::Result;

mod collect;
mod pattern;

use collect::collect_glob_member_manifests;
use pattern::{member_components, member_contains_glob, path_components_start_with};

pub(super) fn member_manifest_paths_for_pattern(root: &Path, member: &str) -> Result<Vec<PathBuf>> {
    if member_contains_glob(member) {
        let mut paths = Vec::new();
        collect_glob_member_manifests(root, &member_components(member), &mut paths)?;
        paths.sort();
        paths.dedup();
        return Ok(paths);
    }

    let manifest = root.join(member).join("Cargo.toml");
    Ok(manifest.is_file().then_some(manifest).into_iter().collect())
}

pub(super) fn workspace_member_root_is_excluded(
    root: &Path,
    member_root: &Path,
    exclude: &str,
) -> bool {
    let exclude_components = member_components(exclude);
    if exclude_components.is_empty() {
        return false;
    }
    let member_components = member_root
        .strip_prefix(root)
        .unwrap_or(member_root)
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(str::to_string)
        .collect::<Vec<_>>();
    path_components_start_with(&member_components, &exclude_components)
}
