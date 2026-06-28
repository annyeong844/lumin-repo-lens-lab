use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use lumin_rust_common::sha256_file;

use crate::config::cargo_config_paths;
use crate::environment::CompilationEnvironment;
use crate::metadata::{package_root, CargoMetadata, CargoPackage};
use crate::path_util::is_inside_path;

pub(super) fn collect_input_file_hashes(
    root: &Path,
    metadata: Option<&CargoMetadata>,
    selected: &[CargoPackage],
    compilation_environment: &CompilationEnvironment,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(metadata) = metadata {
        for pkg in selected_input_packages(metadata, selected) {
            insert_file_hash(&mut out, Path::new(&pkg.manifest_path));
            let pkg_root = package_root(pkg);
            collect_rust_source_files(
                &pkg_root,
                metadata.target_directory.as_deref().map(Path::new),
                &mut out,
            );
        }
    }

    for base in [
        Some(root),
        metadata
            .and_then(|m| m.workspace_root.as_deref())
            .map(Path::new),
    ]
    .into_iter()
    .flatten()
    {
        for name in [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain",
            "rust-toolchain.toml",
        ] {
            insert_file_hash(&mut out, &base.join(name));
        }
        for config in cargo_config_paths(base, compilation_environment) {
            insert_file_hash(&mut out, &config);
        }
    }
    out
}

fn selected_input_packages<'a>(
    metadata: &'a CargoMetadata,
    selected: &'a [CargoPackage],
) -> Vec<&'a CargoPackage> {
    if selected.is_empty() {
        return metadata
            .packages
            .iter()
            .filter(|pkg| pkg.source.is_none())
            .collect();
    }

    let packages_by_id = metadata
        .packages
        .iter()
        .map(|pkg| (pkg.id.as_str(), pkg))
        .collect::<BTreeMap<_, _>>();
    let mut input_ids = BTreeSet::new();
    let mut queue = VecDeque::new();
    for pkg in selected {
        if pkg.source.is_none() && input_ids.insert(pkg.id.as_str()) {
            queue.push_back(pkg.id.as_str());
        }
    }
    let nodes_by_id = metadata
        .resolve
        .as_ref()
        .map(|resolve| {
            resolve
                .nodes
                .iter()
                .map(|node| (node.id.as_str(), node))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    while let Some(package_id) = queue.pop_front() {
        let Some(node) = nodes_by_id.get(package_id) else {
            continue;
        };
        for dependency_id in &node.dependencies {
            let dependency_id = dependency_id.as_str();
            let Some(dependency) = packages_by_id.get(dependency_id) else {
                continue;
            };
            if dependency.source.is_none() && input_ids.insert(dependency_id) {
                queue.push_back(dependency_id);
            }
        }
    }

    metadata
        .packages
        .iter()
        .filter(|pkg| input_ids.contains(pkg.id.as_str()) && pkg.source.is_none())
        .collect()
}

fn collect_rust_source_files(
    dir: &Path,
    target_dir: Option<&Path>,
    out: &mut BTreeMap<String, String>,
) {
    if target_dir.is_some_and(|target_dir| is_inside_path(dir, target_dir)) {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if entry.file_name() == OsStr::new(".git") {
                continue;
            }
            collect_rust_source_files(&path, target_dir, out);
        } else if file_type.is_file() && path.extension() == Some(OsStr::new("rs")) {
            insert_file_hash(out, &path);
        }
    }
}

fn insert_file_hash(out: &mut BTreeMap<String, String>, path: &Path) {
    if let Ok(hash) = sha256_file(path) {
        out.insert(path.display().to_string(), hash);
    }
}
