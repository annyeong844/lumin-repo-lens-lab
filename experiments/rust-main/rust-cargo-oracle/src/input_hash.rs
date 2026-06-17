use serde_json::json;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::config::cargo_config_paths;
use crate::metadata::{package_root, CargoMetadata, CargoPackage};
use crate::path_util::is_inside_path;
use crate::toolchain::{toolchain_json, Toolchain};
use crate::util::{sha256_file, sha256_text};
use crate::DIAGNOSTIC_POLICY_VERSION;

pub(crate) fn analysis_input_set_hash(
    root: &Path,
    metadata: Option<&CargoMetadata>,
    cargo_args: &[String],
    selected: &[CargoPackage],
    registry_hash: &str,
    features: Option<&str>,
    package_name: Option<&str>,
    toolchain: &Toolchain,
) -> String {
    let selected_ids: Vec<&str> = selected.iter().map(|pkg| pkg.id.as_str()).collect();
    let value = json!({
        "policyVersion": DIAGNOSTIC_POLICY_VERSION,
        "registryHash": registry_hash,
        "cargoArgs": cargo_args,
        "selectedPackageIds": selected_ids,
        "features": features,
        "packageName": package_name,
        "toolchain": toolchain_json(toolchain),
        "compilationEnvironment": compilation_environment_snapshot(),
        "fileHashes": collect_input_file_hashes(root, metadata),
    });
    sha256_text(&value.to_string())
}

fn collect_input_file_hashes(
    root: &Path,
    metadata: Option<&CargoMetadata>,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(metadata) = metadata {
        for pkg in metadata.packages.iter().filter(|pkg| pkg.source.is_none()) {
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
        for config in cargo_config_paths(base, metadata) {
            insert_file_hash(&mut out, &config);
        }
    }
    out
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

fn compilation_environment_snapshot() -> BTreeMap<String, String> {
    const KEYS: &[&str] = &[
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_RUSTFLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_TARGET",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_TARGET_DIR",
        "CARGO_HOME",
        "CARGO_INCREMENTAL",
        "CARGO_BUILD_INCREMENTAL",
        "CARGO_BUILD_BUILD_DIR",
    ];
    let mut out = BTreeMap::new();
    for (key, value) in std::env::vars() {
        if KEYS.contains(&key.as_str())
            || key.starts_with("CARGO_TARGET_")
            || key.starts_with("CARGO_PROFILE_")
        {
            out.insert(key, value);
        }
    }
    out
}
