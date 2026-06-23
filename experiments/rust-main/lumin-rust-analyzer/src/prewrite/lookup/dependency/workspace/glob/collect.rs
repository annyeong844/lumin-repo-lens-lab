use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::pattern::{component_contains_glob, glob_component_matches};
use super::workspace_member_root_is_excluded;

pub(super) fn collect_glob_member_manifests(
    root: &Path,
    components: &[String],
    excludes: &[Vec<String>],
    matched_member_roots: &mut usize,
    paths: &mut Vec<PathBuf>,
) -> Result<()> {
    collect_glob_member_manifests_at(
        root,
        components,
        excludes,
        matched_member_roots,
        0,
        root,
        paths,
    )
}

fn collect_glob_member_manifests_at(
    root: &Path,
    components: &[String],
    excludes: &[Vec<String>],
    matched_member_roots: &mut usize,
    index: usize,
    current: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<()> {
    if index == components.len() {
        *matched_member_roots += 1;
        if excludes
            .iter()
            .any(|exclude| workspace_member_root_is_excluded(root, current, exclude))
        {
            return Ok(());
        }
        let manifest = current.join("Cargo.toml");
        if manifest.is_file() {
            paths.push(manifest);
        } else {
            bail!(
                "blocked-prewrite-dependency-manifest: workspace member directory {} does not contain Cargo.toml",
                current.display()
            );
        }
        return Ok(());
    }

    let component = &components[index];
    if component == "**" {
        collect_glob_member_manifests_at(
            root,
            components,
            excludes,
            matched_member_roots,
            index + 1,
            current,
            paths,
        )?;
        for child in child_directories(current)? {
            collect_glob_member_manifests_at(
                root,
                components,
                excludes,
                matched_member_roots,
                index,
                &child,
                paths,
            )?;
        }
        return Ok(());
    }

    if component_contains_glob(component) {
        for child in child_directories(current)? {
            let Some(name) = child.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if glob_component_matches(component, name) {
                collect_glob_member_manifests_at(
                    root,
                    components,
                    excludes,
                    matched_member_roots,
                    index + 1,
                    &child,
                    paths,
                )?;
            }
        }
        return Ok(());
    }

    let next = current.join(component);
    if next.is_dir() {
        collect_glob_member_manifests_at(
            root,
            components,
            excludes,
            matched_member_roots,
            index + 1,
            &next,
            paths,
        )?;
    }
    Ok(())
}

fn child_directories(parent: &Path) -> Result<Vec<PathBuf>> {
    if !parent.is_dir() {
        return Ok(Vec::new());
    }
    let mut children = Vec::new();
    for entry in fs::read_dir(parent).with_context(|| {
        format!(
            "blocked-prewrite-dependency-manifest: failed to read workspace member directory {}",
            parent.display()
        )
    })? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            children.push(entry.path());
        }
    }
    children.sort();
    Ok(children)
}
