use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::metadata::{package_root, packages_by_name_or_id, CargoMetadata, CargoPackage};
use crate::path_util::{has_windows_drive_prefix, is_inside_path, normalize_path_for_compare};

#[derive(Debug, Clone, Default)]
pub(crate) struct TargetPackageSelection {
    pub(crate) target_paths: Vec<String>,
    pub(crate) candidate_package_names: Vec<String>,
    pub(crate) package_names: Vec<String>,
    pub(crate) packages: Vec<CargoPackage>,
    pub(crate) paths_by_package: BTreeMap<String, Vec<String>>,
    pub(crate) unmatched_paths: Vec<String>,
}

pub(crate) fn target_package_selection(
    root: &Path,
    metadata: Option<&CargoMetadata>,
    target_paths: &[String],
    package_name: Option<&str>,
) -> TargetPackageSelection {
    let mut normalized_target_paths = target_paths
        .iter()
        .map(|path| path.replace('\\', "/"))
        .collect::<Vec<_>>();
    normalized_target_paths.sort();
    normalized_target_paths.dedup();

    let Some(metadata) = metadata else {
        return TargetPackageSelection {
            unmatched_paths: normalized_target_paths.clone(),
            target_paths: normalized_target_paths,
            ..TargetPackageSelection::default()
        };
    };

    let mut paths_by_package = BTreeMap::<String, Vec<String>>::new();
    let mut unmatched_paths = Vec::<String>::new();
    for target_path in &normalized_target_paths {
        let absolute_path = absolute_target_path(root, target_path);
        let Some(pkg) = best_package_for_path(metadata, &absolute_path, package_name) else {
            unmatched_paths.push(target_path.clone());
            continue;
        };
        paths_by_package
            .entry(pkg.name.clone())
            .or_default()
            .push(target_path.clone());
    }

    let candidate_package_names = ranked_package_names(metadata, &paths_by_package);
    let package_names = candidate_package_names.clone();
    let packages = packages_by_name_or_id(Some(metadata), &package_names);
    TargetPackageSelection {
        target_paths: normalized_target_paths,
        candidate_package_names,
        package_names,
        packages,
        paths_by_package,
        unmatched_paths,
    }
}

fn ranked_package_names(
    metadata: &CargoMetadata,
    paths_by_package: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let packages_by_name = metadata
        .packages
        .iter()
        .filter(|pkg| pkg.source.is_none())
        .map(|pkg| (pkg.name.as_str(), pkg))
        .collect::<BTreeMap<_, _>>();
    let workspace_members = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut ranked = paths_by_package
        .iter()
        .map(|(name, paths)| {
            let local_dependency_count = packages_by_name
                .get(name.as_str())
                .map(|pkg| direct_workspace_dependency_count(metadata, &workspace_members, &pkg.id))
                .unwrap_or(usize::MAX);
            (name.as_str(), local_dependency_count, paths.len())
        })
        .collect::<Vec<_>>();
    ranked.sort_by(
        |(left_name, left_dependency_count, left_path_count),
         (right_name, right_dependency_count, right_path_count)| {
            left_dependency_count
                .cmp(right_dependency_count)
                .then_with(|| left_path_count.cmp(right_path_count))
                .then_with(|| left_name.cmp(right_name))
        },
    );
    ranked
        .into_iter()
        .map(|(name, _, _)| name.to_string())
        .collect()
}

fn direct_workspace_dependency_count(
    metadata: &CargoMetadata,
    workspace_members: &BTreeSet<&str>,
    package_id: &str,
) -> usize {
    metadata
        .resolve
        .as_ref()
        .and_then(|resolve| resolve.nodes.iter().find(|node| node.id == package_id))
        .map(|node| {
            node.dependencies
                .iter()
                .filter(|dependency_id| workspace_members.contains(dependency_id.as_str()))
                .count()
        })
        .unwrap_or(usize::MAX)
}

fn absolute_target_path(root: &Path, target_path: &str) -> PathBuf {
    let path = PathBuf::from(target_path);
    if path.is_absolute() || has_windows_drive_prefix(target_path) {
        path
    } else {
        root.join(path)
    }
}

fn best_package_for_path<'a>(
    metadata: &'a CargoMetadata,
    target_path: &Path,
    package_name: Option<&str>,
) -> Option<&'a CargoPackage> {
    metadata
        .packages
        .iter()
        .filter(|pkg| pkg.source.is_none())
        .filter(|pkg| {
            package_name
                .map(|package_name| pkg.name == package_name || pkg.id == package_name)
                .unwrap_or(true)
        })
        .filter_map(|pkg| {
            let root = package_root(pkg);
            is_inside_path(target_path, &root).then(|| {
                let root_len = normalize_path_for_compare(&root).len();
                (root_len, pkg)
            })
        })
        .max_by_key(|(root_len, _)| *root_len)
        .map(|(_, pkg)| pkg)
}
