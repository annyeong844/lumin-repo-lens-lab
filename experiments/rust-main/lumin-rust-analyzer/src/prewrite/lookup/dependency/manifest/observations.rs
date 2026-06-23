use super::CargoDependencyDeclaration;
use crate::prewrite::lookup::dependency::graph::{
    DependencyImportGraph, DependencyImportObservation,
};
use crate::prewrite::lookup::dependency::scope::CargoManifestScope;
use matching::{push_alias_root_observation, push_requested_root_observation};
use scope::scope_for_file;

mod matching;
mod scope;

pub(super) fn observations_for_dependency<'a>(
    scopes: &[CargoManifestScope],
    graph: &'a DependencyImportGraph,
    requested_root: &str,
    declarations: &[CargoDependencyDeclaration],
) -> (Vec<&'a DependencyImportObservation>, Vec<String>) {
    let requested_roots =
        crate::prewrite::lookup::dependency::roots::rust_code_root_candidates(requested_root);
    let mut observations = Vec::new();
    let mut misses = std::collections::BTreeSet::new();
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
