use super::graph::{DependencyImportGraph, DependencyImportObservation};
use super::manifest::CargoDependencyDeclaration;
use super::model::{
    DependencyLookup, DependencyLookupKind, DependencyLookupResult, ExistingImports,
};

mod citations;
mod counts;
mod examples;

use citations::{declaration_citations, push_lookup_result_citation};
use counts::existing_import_count_fields;
use examples::dependency_examples;

pub(super) fn project_dependency_lookup(
    dependency: &str,
    declaration: Option<&CargoDependencyDeclaration>,
    observed_scope_misses: &[String],
    unowned_observation_count: usize,
    observations: &[&DependencyImportObservation],
    graph: &DependencyImportGraph,
) -> DependencyLookup {
    let observed_import_count = observations.len();
    let examples = dependency_examples(observations);
    let mut citations = declaration_citations(
        dependency,
        declaration,
        observed_import_count,
        observed_scope_misses,
        unowned_observation_count,
    );
    let result = lookup_result(
        declaration.is_some(),
        observed_import_count,
        unowned_observation_count,
        graph,
    );
    push_lookup_result_citation(
        dependency,
        result,
        observed_import_count,
        unowned_observation_count,
        graph,
        &mut citations,
    );
    let count_reason = match result {
        DependencyLookupResult::AvailableImportGraphUnavailable => {
            Some(graph.zero_observed_unavailable_reason())
        }
        DependencyLookupResult::ScopeUnavailable => {
            Some("observed Rust path consumers are outside Cargo package manifest scopes")
        }
        _ => graph.partial_reason.as_deref(),
    };
    let (observed_import_count, count_confidence, unavailable_reason) =
        existing_import_count_fields(result, observed_import_count, count_reason);

    DependencyLookup {
        kind: DependencyLookupKind::Dependency,
        dep_name: dependency.to_string(),
        declared_in: declaration.map(|declaration| declaration.section.clone()),
        result,
        existing_imports: ExistingImports {
            examples,
            observed_import_count,
            count_confidence,
            unavailable_reason: unavailable_reason.map(str::to_string),
        },
        citations,
    }
}

fn lookup_result(
    has_declaration: bool,
    observed_import_count: usize,
    unowned_observation_count: usize,
    graph: &DependencyImportGraph,
) -> DependencyLookupResult {
    if !has_declaration {
        if observed_import_count == 0 && unowned_observation_count > 0 {
            return DependencyLookupResult::ScopeUnavailable;
        }
        return DependencyLookupResult::NewPackage;
    }
    if observed_import_count > 0 {
        return DependencyLookupResult::Available;
    }
    if graph.zero_observed_is_grounded() {
        return DependencyLookupResult::AvailableNoObservedImports;
    }
    DependencyLookupResult::AvailableImportGraphUnavailable
}
