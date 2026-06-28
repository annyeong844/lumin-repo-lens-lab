use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use toml::Value as TomlValue;

pub(super) fn is_package_manifest(value: &TomlValue) -> bool {
    value.get("package").and_then(TomlValue::as_table).is_some()
}

pub(super) fn manifest_target_files(
    root: &Path,
    manifest: &Path,
    value: &TomlValue,
) -> BTreeSet<String> {
    let manifest_dir = manifest.parent().unwrap_or(root);
    let mut paths = BTreeSet::new();
    if let Some(target) = value.get("lib") {
        insert_target_file(root, manifest_dir, target, &mut paths);
    }
    for section in ["bin", "example", "test", "bench"] {
        if let Some(targets) = value.get(section).and_then(TomlValue::as_array) {
            for target in targets {
                insert_target_file(root, manifest_dir, target, &mut paths);
            }
        }
    }
    paths
}

fn insert_target_file(
    root: &Path,
    manifest_dir: &Path,
    target: &TomlValue,
    paths: &mut BTreeSet<String>,
) {
    let Some(path) = target
        .as_table()
        .and_then(|table| table.get("path"))
        .and_then(TomlValue::as_str)
    else {
        return;
    };
    if let Some(relative) = repo_relative_path(root, &manifest_dir.join(path)) {
        paths.insert(relative);
    }
}

fn repo_relative_path(root: &Path, path: &Path) -> Option<String> {
    normalize_path(path)
        .strip_prefix(normalize_path(root))
        .ok()
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
