use std::path::Path;

mod configuration;
mod target;

use crate::cargo_json::CargoJsonMessages;
use crate::environment::CompilationEnvironment;
use crate::metadata::{CargoMetadata, CargoPackage};
use crate::protocol::{
    OracleScope, OracleScopeFeatureSelection, OracleScopeKind, OracleScopeProfile,
};
use crate::toolchain::Toolchain;
use configuration::{resolve_cfg_set, resolve_target_triple};
use target::target_entries;

pub(crate) fn build_scope(
    root: &Path,
    metadata: Option<&CargoMetadata>,
    messages: CargoJsonMessages<'_>,
    selected: &[CargoPackage],
    features: Option<&str>,
    toolchain: &Toolchain,
    environment: &CompilationEnvironment,
) -> OracleScope {
    let feature_selection = feature_selection(features);
    let targets = target_entries(metadata, messages, selected);
    let target_names: Vec<String> = targets
        .iter()
        .map(|target| target.target_name.clone())
        .collect();
    let target = match target_names.as_slice() {
        [] => "unknown".to_string(),
        [only] => only.clone(),
        _ => "<multiple>".to_string(),
    };
    let target_triple = resolve_target_triple(root, toolchain, environment);
    let cfg = resolve_cfg_set(root, environment);

    OracleScope {
        kind: OracleScopeKind::CrateTargetConfiguration,
        package: selected
            .first()
            .map(|pkg| pkg.name.clone())
            .unwrap_or_else(|| "<unknown>".to_string()),
        package_names: selected.iter().map(|pkg| pkg.name.clone()).collect(),
        target,
        targets,
        feature_set: feature_selection.0,
        feature_selection: feature_selection.1,
        target_triple: target_triple.0,
        target_triples: target_triple.1,
        target_triple_source: target_triple.2,
        cfg_set: cfg.0,
        cfg_set_source: cfg.1,
        cfg_set_complete: false,
        profile: OracleScopeProfile::Dev,
    }
}

fn feature_selection(features: Option<&str>) -> (Vec<String>, OracleScopeFeatureSelection) {
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
        OracleScopeFeatureSelection {
            default_features: true,
            all_features: false,
            explicit_features: explicit,
        },
    )
}
