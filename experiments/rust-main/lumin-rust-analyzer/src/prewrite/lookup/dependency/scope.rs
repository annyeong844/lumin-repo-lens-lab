use std::collections::BTreeSet;
use std::path::Path;

use toml::Value as TomlValue;

use super::targets::{is_package_manifest, manifest_target_files};

pub(super) struct CargoManifestScope {
    pub(super) manifest_path: String,
    scope_root: String,
    is_package: bool,
    target_files: BTreeSet<String>,
    pub(super) value: TomlValue,
}

impl CargoManifestScope {
    pub(super) fn root(root: &Path, path: &Path, value: &TomlValue) -> Self {
        Self {
            manifest_path: "Cargo.toml".to_string(),
            scope_root: String::new(),
            is_package: is_package_manifest(value),
            target_files: manifest_target_files(root, path, value),
            value: value.clone(),
        }
    }

    pub(super) fn member(root: &Path, path: &Path, value: TomlValue) -> Self {
        let manifest_path = relative_manifest_path(root, path);
        let scope_root = manifest_scope_root(&manifest_path);
        let target_files = manifest_target_files(root, path, &value);
        Self {
            manifest_path,
            scope_root,
            is_package: is_package_manifest(&value),
            target_files,
            value,
        }
    }

    pub(super) fn file_is_in_scope(&self, file: &str) -> bool {
        self.target_files.contains(file)
            || (self.is_package && file_is_in_scope(file, &self.scope_root))
    }

    pub(super) fn scope_priority_len(&self) -> usize {
        self.scope_root.len()
    }
}

fn relative_manifest_path(root: &Path, manifest: &Path) -> String {
    manifest
        .strip_prefix(root)
        .unwrap_or(manifest)
        .to_string_lossy()
        .replace('\\', "/")
}

fn manifest_scope_root(manifest_path: &str) -> String {
    manifest_path
        .strip_suffix("/Cargo.toml")
        .unwrap_or("")
        .to_string()
}

fn file_is_in_scope(file: &str, scope_root: &str) -> bool {
    scope_root.is_empty()
        || file == scope_root
        || file
            .strip_prefix(scope_root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}
