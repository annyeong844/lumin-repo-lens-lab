use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use lumin_rust_common::{sha256_file, sha256_text};
use serde::Serialize;

use crate::config::cargo_config_paths;
use crate::environment::CompilationEnvironment;
use crate::metadata::{package_root, CargoMetadata, CargoPackage};
use crate::path_util::is_inside_path;
use crate::protocol::{ArtifactProfile, CargoCheckMode, CargoTargetDirMode, RustcCommandSource};
use crate::toolchain::Toolchain;
use crate::DIAGNOSTIC_POLICY_VERSION;

pub(crate) struct AnalysisInputSet<'a> {
    pub(crate) root: &'a Path,
    pub(crate) metadata: Option<&'a CargoMetadata>,
    pub(crate) cargo_args: &'a [String],
    pub(crate) selected: &'a [CargoPackage],
    pub(crate) registry_hash: &'a str,
    pub(crate) compilation_environment: &'a CompilationEnvironment,
    pub(crate) features: Option<&'a str>,
    pub(crate) package_name: Option<&'a str>,
    pub(crate) toolchain: &'a Toolchain,
    pub(crate) cargo_check_mode: CargoCheckMode,
    pub(crate) cargo_target_dir_mode: CargoTargetDirMode,
    pub(crate) target_paths: &'a [String],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisInputIdentity<'a> {
    policy_version: &'static str,
    registry_hash: &'a str,
    cargo_args: &'a [String],
    selected_package_ids: Vec<&'a str>,
    features: Option<&'a str>,
    package_name: Option<&'a str>,
    cargo_check_mode: CargoCheckMode,
    cargo_target_dir_mode: CargoTargetDirMode,
    target_paths: &'a [String],
    toolchain: ToolchainIdentity<'a>,
    compilation_environment: &'a BTreeMap<String, String>,
    file_hashes: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainIdentity<'a> {
    cargo_version: Option<&'a str>,
    rustc_version_verbose: Option<&'a str>,
    rustc_bin: &'a str,
    rustc_source: RustcCommandSource,
    host: Option<&'a str>,
    profile: ArtifactProfile,
}

impl<'a> From<&'a Toolchain> for ToolchainIdentity<'a> {
    fn from(toolchain: &'a Toolchain) -> Self {
        Self {
            cargo_version: toolchain.cargo_version.as_deref(),
            rustc_version_verbose: toolchain.rustc_version_verbose.as_deref(),
            rustc_bin: &toolchain.rustc_bin,
            rustc_source: toolchain.rustc_source,
            host: toolchain.host.as_deref(),
            profile: ArtifactProfile::Dev,
        }
    }
}

pub(crate) fn analysis_input_set_hash(input: AnalysisInputSet<'_>) -> serde_json::Result<String> {
    let identity = AnalysisInputIdentity {
        policy_version: DIAGNOSTIC_POLICY_VERSION,
        registry_hash: input.registry_hash,
        cargo_args: input.cargo_args,
        selected_package_ids: input.selected.iter().map(|pkg| pkg.id.as_str()).collect(),
        features: input.features,
        package_name: input.package_name,
        cargo_check_mode: input.cargo_check_mode,
        cargo_target_dir_mode: input.cargo_target_dir_mode,
        target_paths: input.target_paths,
        toolchain: ToolchainIdentity::from(input.toolchain),
        compilation_environment: input.compilation_environment.values(),
        file_hashes: collect_input_file_hashes(
            input.root,
            input.metadata,
            input.selected,
            input.compilation_environment,
        ),
    };
    serde_json::to_string(&identity).map(|text| sha256_text(&text))
}

fn collect_input_file_hashes(
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
