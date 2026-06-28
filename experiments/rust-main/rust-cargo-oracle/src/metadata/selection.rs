use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{CargoMetadata, CargoPackage};
use crate::path_util::normalize_path_for_compare;

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

pub(crate) fn packages_by_name_or_id(
    metadata: Option<&CargoMetadata>,
    package_names: &[String],
) -> Vec<CargoPackage> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    let by_name_or_id = metadata
        .packages
        .iter()
        .flat_map(|pkg| [(pkg.name.as_str(), pkg), (pkg.id.as_str(), pkg)])
        .collect::<BTreeMap<_, _>>();
    package_names
        .iter()
        .filter_map(|name| by_name_or_id.get(name.as_str()).copied())
        .cloned()
        .collect()
}

pub(crate) fn package_root(pkg: &CargoPackage) -> PathBuf {
    Path::new(&pkg.manifest_path)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf()
}

fn same_path(left: &Path, right: &Path) -> bool {
    normalize_path_for_compare(left) == normalize_path_for_compare(right)
}
