use std::collections::BTreeSet;

use crate::prewrite::lookup::dependency::graph::DependencyImportObservation;
use crate::prewrite::lookup::dependency::manifest::CargoDependencyDeclaration;
use crate::prewrite::lookup::dependency::scope::CargoManifestScope;

pub(super) fn push_requested_root_observation<'a>(
    observation: &'a DependencyImportObservation,
    scope: Option<&CargoManifestScope>,
    workspace_excluded: bool,
    declarations: &[CargoDependencyDeclaration],
    observations: &mut Vec<&'a DependencyImportObservation>,
    misses: &mut BTreeSet<String>,
    unowned_observation_count: &mut usize,
) {
    let Some(scope) = scope else {
        if !workspace_excluded {
            *unowned_observation_count += 1;
        }
        return;
    };
    if declarations.is_empty() {
        observations.push(observation);
        return;
    }
    if declarations
        .iter()
        .any(|declaration| declaration.manifest_path == scope.manifest_path)
    {
        observations.push(observation);
    } else {
        misses.insert(scope.manifest_path.clone());
        observations.push(observation);
    }
}

pub(super) fn push_alias_root_observation<'a>(
    observation: &'a DependencyImportObservation,
    scope: Option<&CargoManifestScope>,
    workspace_excluded: bool,
    declarations: &[CargoDependencyDeclaration],
    observations: &mut Vec<&'a DependencyImportObservation>,
    misses: &mut BTreeSet<String>,
    unowned_observation_count: &mut usize,
) {
    let alias_matches_dependency = declarations
        .iter()
        .any(|declaration| declaration.matches_manifest_key_root(&observation.root));
    if !alias_matches_dependency {
        return;
    }
    let Some(scope) = scope else {
        if !workspace_excluded {
            *unowned_observation_count += 1;
        }
        return;
    };
    if declarations.iter().any(|declaration| {
        declaration.manifest_path == scope.manifest_path
            && declaration.matches_manifest_key_root(&observation.root)
    }) {
        observations.push(observation);
    } else {
        misses.insert(scope.manifest_path.clone());
        observations.push(observation);
    }
}
