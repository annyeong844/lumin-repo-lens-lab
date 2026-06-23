use std::collections::BTreeSet;

use super::CargoDependencyDeclaration;
use crate::prewrite::lookup::dependency::graph::{
    DependencyImportGraph, DependencyImportObservation,
};
use crate::prewrite::lookup::dependency::scope::CargoManifestScope;

pub(super) fn observations_for_dependency<'a>(
    scopes: &[CargoManifestScope],
    graph: &'a DependencyImportGraph,
    requested_root: &str,
    declarations: &[CargoDependencyDeclaration],
) -> (Vec<&'a DependencyImportObservation>, Vec<String>) {
    let requested_roots = rust_code_root_candidates(requested_root);
    let mut observations = Vec::new();
    let mut misses = BTreeSet::new();
    for observation in graph.observations() {
        let scope = scope_for_file(scopes, &observation.file);
        if requested_roots.contains(&observation.root) {
            push_requested_root_observation(
                observation,
                scope,
                declarations,
                &mut observations,
                &mut misses,
            );
            continue;
        }
        push_alias_root_observation(
            observation,
            scope,
            declarations,
            &mut observations,
            &mut misses,
        );
    }
    (observations, misses.into_iter().collect())
}

pub(super) fn declaration_for_observations(
    scopes: &[CargoManifestScope],
    observations: &[&DependencyImportObservation],
    declarations: &[CargoDependencyDeclaration],
) -> Option<CargoDependencyDeclaration> {
    observations.iter().find_map(|observation| {
        let scope = scope_for_file(scopes, &observation.file)?;
        declarations
            .iter()
            .find(|declaration| declaration.manifest_path == scope.manifest_path)
            .cloned()
    })
}

fn push_requested_root_observation<'a>(
    observation: &'a DependencyImportObservation,
    scope: Option<&CargoManifestScope>,
    declarations: &[CargoDependencyDeclaration],
    observations: &mut Vec<&'a DependencyImportObservation>,
    misses: &mut BTreeSet<String>,
) {
    if declarations.is_empty() {
        observations.push(observation);
        return;
    }
    let Some(scope) = scope else {
        return;
    };
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

fn push_alias_root_observation<'a>(
    observation: &'a DependencyImportObservation,
    scope: Option<&CargoManifestScope>,
    declarations: &[CargoDependencyDeclaration],
    observations: &mut Vec<&'a DependencyImportObservation>,
    misses: &mut BTreeSet<String>,
) {
    let Some(scope) = scope else {
        return;
    };
    let alias_matches_dependency = declarations
        .iter()
        .any(|declaration| declaration.matches_manifest_key_root(&observation.root));
    if !alias_matches_dependency {
        return;
    }
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

fn scope_for_file<'a>(
    scopes: &'a [CargoManifestScope],
    file: &str,
) -> Option<&'a CargoManifestScope> {
    scopes
        .iter()
        .filter(|scope| scope.file_is_in_scope(file))
        .max_by_key(|scope| scope.scope_priority_len())
}

fn rust_code_root_candidates(root: &str) -> BTreeSet<String> {
    BTreeSet::from([root.to_string(), root.replace('-', "_")])
}
