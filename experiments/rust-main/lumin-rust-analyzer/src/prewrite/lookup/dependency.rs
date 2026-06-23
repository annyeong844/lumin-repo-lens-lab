use std::path::Path;

use anyhow::Result;
use lumin_rust_source_health::protocol::HealthResponse;

use super::super::intent::NormalizedIntent;

mod declarations;
mod graph;
mod manifest;
mod model;
mod projection;
mod roots;
mod scope;
mod targets;
mod workspace;

use graph::DependencyImportGraph;
use manifest::CargoManifest;
pub(in crate::prewrite) use model::{
    DependencyLookup, DependencyLookupResult, DEPENDENCY_EXAMPLE_LIMIT,
    DEPENDENCY_WATCH_FOR_THRESHOLD,
};
use projection::project_dependency_lookup;
use roots::dependency_root;

pub(in crate::prewrite) fn lookup_dependencies(
    intent: &NormalizedIntent,
    syntax: &HealthResponse,
    root: &Path,
) -> Result<Vec<DependencyLookup>> {
    if intent.dependencies.is_empty() {
        return Ok(Vec::new());
    }
    let manifest = CargoManifest::read(root)?;
    let graph = DependencyImportGraph::from_syntax(syntax);
    Ok(intent
        .dependencies
        .iter()
        .map(|dependency| lookup_dependency(dependency, &manifest, &graph))
        .collect())
}

fn lookup_dependency(
    dependency: &str,
    manifest: &CargoManifest,
    graph: &DependencyImportGraph,
) -> DependencyLookup {
    let requested_root = dependency_root(dependency);
    let declarations = manifest.find_declarations(&requested_root);
    let (observations, observed_scope_misses) =
        manifest.observations_for_dependency(graph, &requested_root, &declarations);
    let declaration = if observed_scope_misses.is_empty() {
        manifest
            .declaration_for_observations(&observations, &declarations)
            .or_else(|| declarations.first().cloned())
    } else {
        None
    };
    project_dependency_lookup(
        dependency,
        declaration.as_ref(),
        &observed_scope_misses,
        &observations,
        graph,
    )
}
