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

fn path_components_start_with(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path[..prefix.len()] == *prefix
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
    pattern.contains('*') || pattern.contains('?') || pattern.contains('[')
}

fn component_contains_glob(component: &str) -> bool {
    component.contains('*') || component.contains('?') || component.contains('[')
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
        '[' => {
            if let Some((matched, consumed)) = match_char_class(pattern, value.first().copied()) {
                matched && glob_chars_match(&pattern[consumed..], &value[1..])
            } else {
                value.first() == Some(&'[') && glob_chars_match(&pattern[1..], &value[1..])
            }
        }
        ch => value.first() == Some(&ch) && glob_chars_match(&pattern[1..], &value[1..]),
    }
}

fn match_char_class(pattern: &[char], value: Option<char>) -> Option<(bool, usize)> {
    let value = value?;
    let negated = matches!(pattern.get(1), Some('!' | '^'));
    let mut index = if negated { 2 } else { 1 };
    let mut matched = false;
    let mut has_member = false;
    while index < pattern.len() {
        if pattern[index] == ']' && has_member {
            return Some((if negated { !matched } else { matched }, index + 1));
        }
        if index + 2 < pattern.len() && pattern[index + 1] == '-' && pattern[index + 2] != ']' {
            let start = pattern[index];
            let end = pattern[index + 2];
            matched |= start <= value && value <= end;
            index += 3;
        } else {
            matched |= pattern[index] == value;
            index += 1;
        }
        has_member = true;
    }
    None
}
