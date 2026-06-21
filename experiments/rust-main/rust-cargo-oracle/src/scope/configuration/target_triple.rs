use std::path::Path;

use crate::config::{cargo_config_paths, read_build_target_from_config};
use crate::environment::CompilationEnvironment;
use crate::protocol::OracleTargetTripleSource;
use crate::toolchain::Toolchain;

use super::rel;

pub(in crate::scope) fn resolve_target_triple(
    root: &Path,
    toolchain: &Toolchain,
    environment: &CompilationEnvironment,
) -> (String, Vec<String>, OracleTargetTripleSource) {
    if let Some(value) = environment.get("CARGO_BUILD_TARGET") {
        if !value.trim().is_empty() {
            return (
                value.to_string(),
                vec![value.to_string()],
                OracleTargetTripleSource::EnvCargoBuildTarget,
            );
        }
    }
    for config_path in cargo_config_paths(root, environment) {
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
                    OracleTargetTripleSource::cargo_config(rel(root, &config_path)),
                );
            }
            if !targets.is_empty() {
                return (
                    "<multiple>".to_string(),
                    targets,
                    OracleTargetTripleSource::cargo_config(rel(root, &config_path)),
                );
            }
        }
    }
    if let Some(host) = &toolchain.host {
        return (
            host.clone(),
            vec![host.clone()],
            OracleTargetTripleSource::DefaultHost,
        );
    }
    (
        "<unknown>".to_string(),
        Vec::new(),
        OracleTargetTripleSource::NotResolved,
    )
}
