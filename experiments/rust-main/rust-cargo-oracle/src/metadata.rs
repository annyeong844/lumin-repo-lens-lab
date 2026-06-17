use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::path_util::normalize_path_for_compare;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoMetadata {
    #[serde(default)]
    pub(crate) packages: Vec<CargoPackage>,
    #[serde(default)]
    pub(crate) workspace_members: Vec<String>,
    #[serde(default)]
    pub(crate) workspace_default_members: Vec<String>,
    #[serde(default)]
    pub(crate) workspace_root: Option<String>,
    #[serde(default)]
    pub(crate) target_directory: Option<String>,
    #[serde(default)]
    pub(crate) resolve: Option<CargoResolve>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoResolve {
    #[serde(default)]
    pub(crate) root: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoPackage {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) manifest_path: String,
    #[serde(default)]
    pub(crate) source: Option<Value>,
    #[serde(default)]
    pub(crate) targets: Vec<CargoTarget>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CargoTarget {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) kind: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) src_path: Option<String>,
    #[serde(default)]
    pub(crate) required_features: Vec<String>,
}

pub(crate) fn selected_packages(
    metadata: Option<&CargoMetadata>,
    package_name: Option<&str>,
    root: &Path,
) -> Vec<CargoPackage> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if let Some(package_name) = package_name {
        return metadata
            .packages
            .iter()
            .filter(|pkg| pkg.name == package_name || pkg.id == package_name)
            .cloned()
            .collect();
    }

    let root_is_workspace_root = metadata
        .workspace_root
        .as_deref()
        .map(Path::new)
        .is_some_and(|workspace_root| same_path(workspace_root, root));
    let selected_ids: BTreeSet<&str> = if root_is_workspace_root {
        if !metadata.workspace_default_members.is_empty() {
            metadata
                .workspace_default_members
                .iter()
                .map(String::as_str)
                .collect()
        } else if let Some(root_id) = metadata
            .resolve
            .as_ref()
            .and_then(|resolve| resolve.root.as_deref())
        {
            BTreeSet::from([root_id])
        } else {
            metadata
                .workspace_members
                .iter()
                .map(String::as_str)
                .collect()
        }
    } else if let Some(root_id) = metadata
        .resolve
        .as_ref()
        .and_then(|resolve| resolve.root.as_deref())
    {
        BTreeSet::from([root_id])
    } else if !metadata.workspace_default_members.is_empty() {
        metadata
            .workspace_default_members
            .iter()
            .map(String::as_str)
            .collect()
    } else {
        metadata
            .workspace_members
            .iter()
            .map(String::as_str)
            .collect()
    };
    metadata
        .packages
        .iter()
        .filter(|pkg| selected_ids.contains(pkg.id.as_str()))
        .cloned()
        .collect()
}

pub(crate) fn same_path(left: &Path, right: &Path) -> bool {
    normalize_path_for_compare(left) == normalize_path_for_compare(right)
}

pub(crate) fn package_root(pkg: &CargoPackage) -> PathBuf {
    Path::new(&pkg.manifest_path)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf()
}
