use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

use crate::config::{
    cargo_config_paths, read_build_rustflags_from_config, read_build_target_from_config,
};
use crate::metadata::{CargoMetadata, CargoPackage, CargoTarget};
use crate::toolchain::Toolchain;

pub(crate) fn build_scope(
    root: &Path,
    metadata: Option<&CargoMetadata>,
    messages: &[Value],
    selected: &[CargoPackage],
    features: Option<&str>,
    toolchain: &Toolchain,
) -> Value {
    let feature_selection = feature_selection(features);
    let targets = target_entries(metadata, messages, selected);
    let target_names: Vec<String> = targets
        .iter()
        .filter_map(|target| target.get("targetName").and_then(Value::as_str))
        .map(str::to_string)
        .collect();
    let target = match target_names.as_slice() {
        [] => "unknown".to_string(),
        [only] => only.clone(),
        _ => "<multiple>".to_string(),
    };
    let target_triple = resolve_target_triple(root, metadata, toolchain);
    let cfg = resolve_cfg_set(root, metadata);

    json!({
        "kind": "crate-target-configuration",
        "package": selected.first().map(|pkg| pkg.name.as_str()).unwrap_or("<unknown>"),
        "packageNames": selected.iter().map(|pkg| pkg.name.as_str()).collect::<Vec<_>>(),
        "target": target,
        "targets": targets,
        "featureSet": feature_selection.0,
        "featureSelection": feature_selection.1,
        "targetTriple": target_triple.0,
        "targetTriples": target_triple.1,
        "targetTripleSource": target_triple.2,
        "cfgSet": cfg.0,
        "cfgSetSource": cfg.1,
        "cfgSetComplete": false,
        "profile": "dev",
    })
}

fn feature_selection(features: Option<&str>) -> (Vec<String>, Value) {
    let explicit: Vec<String> = features
        .map(|features| {
            features
                .split([',', ' '])
                .map(str::trim)
                .filter(|feature| !feature.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let mut feature_set = vec!["default".to_string()];
    feature_set.extend(explicit.iter().cloned());
    feature_set.sort();
    feature_set.dedup();
    (
        feature_set,
        json!({
            "defaultFeatures": true,
            "allFeatures": false,
            "explicitFeatures": explicit,
        }),
    )
}

fn target_entries(
    metadata: Option<&CargoMetadata>,
    messages: &[Value],
    selected: &[CargoPackage],
) -> Vec<Value> {
    let mut entries = std::collections::BTreeMap::<String, Value>::new();
    let selected_ids = selected
        .iter()
        .map(|pkg| pkg.id.as_str())
        .collect::<BTreeSet<_>>();
    for message in messages {
        if !matches!(
            message.get("reason").and_then(Value::as_str),
            Some("compiler-artifact" | "compiler-message")
        ) {
            continue;
        }
        let package_id = message.get("package_id").and_then(Value::as_str);
        if !selected_ids.is_empty()
            && package_id.is_some_and(|package_id| !selected_ids.contains(package_id))
        {
            continue;
        }
        let target = message.get("target").or_else(|| {
            message
                .get("message")
                .and_then(|message| message.get("target"))
        });
        if let Some(target) = target {
            let name = target
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            let package_id = package_id.unwrap_or("");
            entries.insert(
                format!("{package_id}:{name}"),
                json!({
                    "packageId": package_id,
                    "packageName": package_name_for_id(metadata, package_id),
                    "targetName": name,
                    "targetKinds": target.get("kind").cloned().unwrap_or_else(|| json!([])),
                    "source": "cargo-json-message",
                }),
            );
        }
    }
    if entries.is_empty() {
        for pkg in selected {
            for target in pkg
                .targets
                .iter()
                .filter(|target| is_default_checked_target(target))
            {
                entries.insert(
                    format!("{}:{}", pkg.id, target.name),
                    json!({
                        "packageId": pkg.id,
                        "packageName": pkg.name,
                        "targetName": target.name,
                        "targetKinds": target.kind,
                        "source": "cargo-metadata-default-selection",
                    }),
                );
            }
        }
    }
    entries.into_values().collect()
}

fn package_name_for_id(metadata: Option<&CargoMetadata>, package_id: &str) -> Value {
    metadata
        .and_then(|metadata| metadata.packages.iter().find(|pkg| pkg.id == package_id))
        .map(|pkg| json!(pkg.name))
        .unwrap_or(Value::Null)
}

fn is_default_checked_target(target: &CargoTarget) -> bool {
    target.required_features.is_empty()
        && target
            .kind
            .iter()
            .any(|kind| kind == "lib" || kind == "bin")
}

fn resolve_target_triple(
    root: &Path,
    metadata: Option<&CargoMetadata>,
    toolchain: &Toolchain,
) -> (String, Vec<String>, String) {
    if let Ok(value) = std::env::var("CARGO_BUILD_TARGET") {
        if !value.trim().is_empty() {
            return (
                value.clone(),
                vec![value],
                "env:CARGO_BUILD_TARGET".to_string(),
            );
        }
    }
    for config_path in cargo_config_paths(root, metadata) {
        if let Some(targets) = read_build_target_from_config(&config_path) {
            let mut targets: Vec<String> = targets
                .into_iter()
                .map(|target| target.trim().to_string())
                .filter(|target| !target.is_empty())
                .collect();
            targets.sort();
            targets.dedup();
            if targets.len() == 1 {
                return (
                    targets[0].clone(),
                    targets,
                    format!("cargo-config:{}", rel(root, &config_path)),
                );
            }
            if !targets.is_empty() {
                return (
                    "<multiple>".to_string(),
                    targets,
                    format!("cargo-config:{}", rel(root, &config_path)),
                );
            }
        }
    }
    if let Some(host) = &toolchain.host {
        return (host.clone(), vec![host.clone()], "default-host".to_string());
    }
    (
        "<unknown>".to_string(),
        Vec::new(),
        "not-resolved".to_string(),
    )
}

fn resolve_cfg_set(root: &Path, metadata: Option<&CargoMetadata>) -> (Vec<String>, String) {
    let mut cfgs = Vec::new();
    if let Ok(value) = std::env::var("RUSTFLAGS") {
        cfgs.extend(cfgs_from_rustflags(split_rustflags(&value, false)));
    }
    if let Ok(value) = std::env::var("CARGO_BUILD_RUSTFLAGS") {
        cfgs.extend(cfgs_from_rustflags(split_rustflags(&value, false)));
    }
    if let Ok(value) = std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        cfgs.extend(cfgs_from_rustflags(split_rustflags(&value, true)));
    }
    if !cfgs.is_empty() {
        cfgs.sort();
        cfgs.dedup();
        return (cfgs, "env-rustflags-best-effort".to_string());
    }

    let mut cfgs = Vec::new();
    let mut sources = Vec::new();
    for config_path in cargo_config_paths(root, metadata).into_iter().rev() {
        let path_cfgs = cfgs_from_rustflags(read_build_rustflags_from_config(&config_path));
        if !path_cfgs.is_empty() {
            cfgs.extend(path_cfgs);
            sources.push(rel(root, &config_path));
        }
    }
    if !cfgs.is_empty() {
        cfgs.sort();
        cfgs.dedup();
        let source = if sources.len() == 1 {
            format!("cargo-config:{}", sources[0])
        } else {
            "cargo-config-merged-best-effort".to_string()
        };
        return (cfgs, source);
    }

    (Vec::new(), "not-resolved".to_string())
}

fn split_rustflags(value: &str, encoded: bool) -> Vec<String> {
    if encoded {
        value
            .split('\u{1f}')
            .filter(|part| !part.is_empty())
            .map(str::to_string)
            .collect()
    } else {
        value.split_whitespace().map(str::to_string).collect()
    }
}

fn cfgs_from_rustflags(parts: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < parts.len() {
        if parts[index] == "--cfg" {
            if let Some(value) = parts.get(index + 1) {
                out.push(value.clone());
                index += 2;
                continue;
            }
        } else if let Some(value) = parts[index].strip_prefix("--cfg=") {
            out.push(value.to_string());
        }
        index += 1;
    }
    out.sort();
    out.dedup();
    out
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::resolve_target_triple;
    use crate::toolchain::Toolchain;
    use anyhow::Result;
    use std::fs;
    use tempfile::TempDir;

    fn fallback_toolchain() -> Toolchain {
        Toolchain {
            cargo_version: None,
            rustc_version_verbose: None,
            host: Some("host-target".to_string()),
        }
    }

    #[test]
    fn target_triple_prefers_deeper_project_config_over_parent() -> Result<()> {
        let temp = TempDir::new()?;
        let repo = temp.path().join("repo");
        let crate_root = repo.join("crate");
        fs::create_dir_all(repo.join(".cargo"))?;
        fs::create_dir_all(crate_root.join(".cargo"))?;
        fs::write(
            repo.join(".cargo").join("config.toml"),
            "[build]\ntarget = \"parent-target\"\n",
        )?;
        fs::write(
            crate_root.join(".cargo").join("config.toml"),
            "[build]\ntarget = \"project-target\"\n",
        )?;

        let (target, targets, source) =
            resolve_target_triple(&crate_root, None, &fallback_toolchain());

        assert_eq!(target, "project-target");
        assert_eq!(targets, vec!["project-target"]);
        assert!(source.contains(".cargo"));
        Ok(())
    }

    #[test]
    fn target_triple_prefers_extensionless_config_over_config_toml() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path().join("crate");
        fs::create_dir_all(root.join(".cargo"))?;
        fs::write(
            root.join(".cargo").join("config"),
            "[build]\ntarget = \"extensionless-target\"\n",
        )?;
        fs::write(
            root.join(".cargo").join("config.toml"),
            "[build]\ntarget = \"toml-target\"\n",
        )?;

        let (target, targets, source) = resolve_target_triple(&root, None, &fallback_toolchain());

        assert_eq!(target, "extensionless-target");
        assert_eq!(targets, vec!["extensionless-target"]);
        assert!(source.ends_with("config"));
        Ok(())
    }
}
