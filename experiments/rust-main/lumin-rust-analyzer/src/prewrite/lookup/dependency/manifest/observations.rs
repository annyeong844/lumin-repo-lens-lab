use super::CargoDependencyDeclaration;
use crate::prewrite::lookup::dependency::graph::{
    DependencyImportGraph, DependencyImportObservation,
};
use crate::prewrite::lookup::dependency::scope::CargoManifestScope;
use matching::{push_alias_root_observation, push_requested_root_observation};
use scope::scope_for_file;

mod matching;
mod scope;

pub(in crate::prewrite::lookup::dependency) struct DependencyObservations<'a> {
    pub(in crate::prewrite::lookup::dependency) observations: Vec<&'a DependencyImportObservation>,
    pub(in crate::prewrite::lookup::dependency) observed_scope_misses: Vec<String>,
    pub(in crate::prewrite::lookup::dependency) unowned_observation_count: usize,
}

pub(super) fn observations_for_dependency<'a>(
    scopes: &[CargoManifestScope],
    workspace_exclude_roots: &[String],
    graph: &'a DependencyImportGraph,
    requested_root: &str,
    declarations: &[CargoDependencyDeclaration],
) -> DependencyObservations<'a> {
    let requested_roots =
        crate::prewrite::lookup::dependency::roots::rust_code_root_candidates(requested_root);
    let mut observations = Vec::new();
    let mut misses = std::collections::BTreeSet::new();
    let mut unowned_observation_count = 0;
    for observation in graph.observations() {
        let scope = scope_for_file(scopes, &observation.file);
        let workspace_excluded = scope.is_none()
            && file_is_under_workspace_exclude(&observation.file, workspace_exclude_roots);
        if requested_roots.contains(&observation.root) {
            push_requested_root_observation(
                observation,
                scope,
                workspace_excluded,
                declarations,
                &mut observations,
                &mut misses,
                &mut unowned_observation_count,
            );
            continue;
        }
        push_alias_root_observation(
            observation,
            scope,
            workspace_excluded,
            declarations,
            &mut observations,
            &mut misses,
            &mut unowned_observation_count,
        );
    }
    DependencyObservations {
        observations,
        observed_scope_misses: misses.into_iter().collect(),
        unowned_observation_count,
    }
}

fn file_is_under_workspace_exclude(file: &str, exclude_roots: &[String]) -> bool {
    exclude_roots.iter().any(|root| {
        file == root
            || file
                .strip_prefix(root)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
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
