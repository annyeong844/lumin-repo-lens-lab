use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

mod collect;
mod pattern;

use collect::collect_glob_member_manifests;
use pattern::{member_components, member_contains_glob, path_components_start_with};

pub(super) fn member_manifest_paths_for_pattern(
    root: &Path,
    member: &str,
    excludes: &[String],
) -> Result<Vec<PathBuf>> {
    if member_contains_glob(member) {
        let mut paths = Vec::new();
        let mut matched_member_roots = 0;
        let exclude_components = excludes
            .iter()
            .map(|exclude| member_components(exclude))
            .filter(|components| !components.is_empty())
            .collect::<Vec<_>>();
        collect_glob_member_manifests(
            root,
            &member_components(member),
            &exclude_components,
            &mut matched_member_roots,
            &mut paths,
        )?;
        paths.sort();
        paths.dedup();
        if paths.is_empty() && matched_member_roots == 0 {
            bail!(
                "blocked-prewrite-dependency-manifest: workspace member pattern '{member}' did not resolve to any Cargo.toml files"
            );
        }
        return Ok(paths);
    }

    let manifest = root.join(member).join("Cargo.toml");
    if !manifest.is_file() {
        bail!(
            "blocked-prewrite-dependency-manifest: workspace member '{member}' does not contain Cargo.toml"
        );
    }
    Ok(vec![manifest])
}

pub(super) fn workspace_member_root_is_excluded(
    root: &Path,
    member_root: &Path,
    exclude: &[String],
) -> bool {
    let member_components = member_root
        .strip_prefix(root)
        .unwrap_or(member_root)
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(str::to_string)
        .collect::<Vec<_>>();
    path_components_start_with(&member_components, exclude)
}
