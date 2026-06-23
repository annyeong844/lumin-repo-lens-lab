use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub(super) fn member_manifest_paths_for_pattern(root: &Path, member: &str) -> Result<Vec<PathBuf>> {
    if member_contains_glob(member) {
        let mut paths = Vec::new();
        collect_glob_member_manifests(&member_components(member), 0, root, &mut paths)?;
        paths.sort();
        paths.dedup();
        return Ok(paths);
    }

    let manifest = root.join(member).join("Cargo.toml");
    Ok(manifest.is_file().then_some(manifest).into_iter().collect())
}

pub(super) fn workspace_member_root_matches(
    root: &Path,
    member_root: &Path,
    pattern: &str,
) -> bool {
    let components = member_components(pattern);
    let member_components = member_root
        .strip_prefix(root)
        .unwrap_or(member_root)
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(str::to_string)
        .collect::<Vec<_>>();
    path_components_match(&components, &member_components)
}

fn collect_glob_member_manifests(
    components: &[String],
    index: usize,
    current: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<()> {
    if index == components.len() {
        let manifest = current.join("Cargo.toml");
        if manifest.is_file() {
            paths.push(manifest);
        }
        return Ok(());
    }

    let component = &components[index];
    if component == "**" {
        collect_glob_member_manifests(components, index + 1, current, paths)?;
        for child in child_directories(current)? {
            collect_glob_member_manifests(components, index, &child, paths)?;
        }
        return Ok(());
    }

    if component_contains_glob(component) {
        for child in child_directories(current)? {
            let Some(name) = child.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if glob_component_matches(component, name) {
                collect_glob_member_manifests(components, index + 1, &child, paths)?;
            }
        }
        return Ok(());
    }

    let next = current.join(component);
    if next.is_dir() {
        collect_glob_member_manifests(components, index + 1, &next, paths)?;
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

fn path_components_match(pattern: &[String], path: &[String]) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }
    if pattern[0] == "**" {
        return path_components_match(&pattern[1..], path)
            || (!path.is_empty() && path_components_match(pattern, &path[1..]));
    }
    !path.is_empty()
        && glob_component_matches(&pattern[0], &path[0])
        && path_components_match(&pattern[1..], &path[1..])
}

fn member_components(pattern: &str) -> Vec<String> {
    pattern
        .replace('\\', "/")
        .split('/')
        .filter(|component| !component.is_empty())
        .map(str::to_string)
        .collect()
}

fn member_contains_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

fn component_contains_glob(component: &str) -> bool {
    component.contains('*') || component.contains('?')
}

fn glob_component_matches(pattern: &str, value: &str) -> bool {
    glob_chars_match(
        &pattern.chars().collect::<Vec<_>>(),
        &value.chars().collect::<Vec<_>>(),
    )
}

fn glob_chars_match(pattern: &[char], value: &[char]) -> bool {
    if pattern.is_empty() {
        return value.is_empty();
    }
    match pattern[0] {
        '*' => {
            glob_chars_match(&pattern[1..], value)
                || (!value.is_empty() && glob_chars_match(pattern, &value[1..]))
        }
        '?' => !value.is_empty() && glob_chars_match(&pattern[1..], &value[1..]),
        ch => value.first() == Some(&ch) && glob_chars_match(&pattern[1..], &value[1..]),
    }
}
