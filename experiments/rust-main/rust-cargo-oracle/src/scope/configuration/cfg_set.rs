use std::path::Path;

use crate::config::{cargo_config_paths, read_build_rustflags_from_config};
use crate::environment::CompilationEnvironment;
use crate::protocol::OracleCfgSetSource;

use super::rel;

pub(in crate::scope) fn resolve_cfg_set(
    root: &Path,
    environment: &CompilationEnvironment,
) -> (Vec<String>, OracleCfgSetSource) {
    let mut cfgs = Vec::new();
    if let Some(value) = environment.get("RUSTFLAGS") {
        cfgs.extend(cfgs_from_rustflags(split_rustflags(value, false)));
    }
    if let Some(value) = environment.get("CARGO_BUILD_RUSTFLAGS") {
        cfgs.extend(cfgs_from_rustflags(split_rustflags(value, false)));
    }
    if let Some(value) = environment.get("CARGO_ENCODED_RUSTFLAGS") {
        cfgs.extend(cfgs_from_rustflags(split_rustflags(value, true)));
    }
    if !cfgs.is_empty() {
        cfgs.sort();
        cfgs.dedup();
        return (cfgs, OracleCfgSetSource::EnvRustflagsBestEffort);
    }

    let mut cfgs = Vec::new();
    let mut sources = Vec::new();
    for config_path in cargo_config_paths(root, environment).into_iter().rev() {
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
            OracleCfgSetSource::cargo_config(sources[0].clone())
        } else {
            OracleCfgSetSource::CargoConfigMergedBestEffort
        };
        return (cfgs, source);
    }

    (Vec::new(), OracleCfgSetSource::NotResolved)
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
