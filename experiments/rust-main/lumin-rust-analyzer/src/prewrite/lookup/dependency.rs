use std::path::Path;

use anyhow::Result;
use lumin_rust_source_health::protocol::HealthResponse;

use super::super::intent::NormalizedIntent;

mod declarations;
mod graph;
mod manifest;
mod model;
mod roots;
mod scope;
mod targets;
mod workspace;

use graph::{DependencyImportGraph, DependencyImportObservation};
use manifest::CargoManifest;
use model::{
    DependencyImportExample, DependencyLookupKind, ExistingImports, ImportCountConfidence,
};
pub(in crate::prewrite) use model::{
    DependencyLookup, DependencyLookupResult, DEPENDENCY_EXAMPLE_LIMIT,
    DEPENDENCY_WATCH_FOR_THRESHOLD,
};

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
    let observed_import_count = observations.len();
    let examples = dependency_examples(&observations);

    let mut citations =
        declaration_citations(dependency, declaration.as_ref(), &observed_scope_misses);
    let result = lookup_result(
        dependency,
        declaration.is_some(),
        observed_import_count,
        graph,
        &mut citations,
    );

    let count_reason = if result == DependencyLookupResult::AvailableImportGraphUnavailable {
        Some(graph.zero_observed_unavailable_reason())
    } else {
        graph.partial_reason.as_deref()
    };
    let (observed_import_count, count_confidence, unavailable_reason) =
        existing_import_count_fields(result, observed_import_count, count_reason);

    DependencyLookup {
        kind: DependencyLookupKind::Dependency,
        dep_name: dependency.to_string(),
        declared_in: declaration
            .as_ref()
            .map(|declaration| declaration.section.clone()),
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

fn dependency_examples(
    observations: &[&DependencyImportObservation],
) -> Vec<DependencyImportExample> {
    observations
        .iter()
        .take(DEPENDENCY_EXAMPLE_LIMIT)
        .map(|observation| DependencyImportExample {
            file: observation.file.clone(),
            from_spec: observation.from_spec.clone(),
        })
        .collect()
}

fn declaration_citations(
    dependency: &str,
    declaration: Option<&manifest::CargoDependencyDeclaration>,
    observed_scope_misses: &[String],
) -> Vec<String> {
    if let Some(declaration) = declaration {
        return vec![format!(
            "[grounded, {}.{}['{}'] declares {dependency} as {}]",
            declaration.manifest_path,
            declaration.section,
            declaration.manifest_key,
            declaration.display_value
        )];
    }
    if !observed_scope_misses.is_empty() {
        return vec![format!(
            "[grounded, observed Rust path consumers for '{dependency}' in Cargo manifest scope(s) without a matching declaration: {}]",
            observed_scope_misses.join(", ")
        )];
    }
    vec![format!(
        "[grounded, Cargo manifest scope does not declare '{dependency}' in dependency tables]"
    )]
}

fn lookup_result(
    dependency: &str,
    has_declaration: bool,
    observed_import_count: usize,
    graph: &DependencyImportGraph,
    citations: &mut Vec<String>,
) -> DependencyLookupResult {
    if !has_declaration {
        return DependencyLookupResult::NewPackage;
    }
    if observed_import_count > 0 {
        citations.push(format!(
            "[grounded, Rust AST static import graph observed {observed_import_count} consumer(s) for '{dependency}']"
        ));
        return DependencyLookupResult::Available;
    }
    if graph.zero_observed_is_grounded() {
        citations.push(format!(
            "[확인 불가, scan range: Rust AST import graph only; '{dependency}' may still be consumed by build scripts, cfg-gated code, generated code, runtime plugins, examples, or external cargo commands]"
        ));
        return DependencyLookupResult::AvailableNoObservedImports;
    }
    let reason = graph.zero_observed_unavailable_reason();
    citations.push(format!(
        "[확인 불가, reason: {reason}; zero observed Rust path consumers is not a grounded absence claim]"
    ));
    DependencyLookupResult::AvailableImportGraphUnavailable
}

fn existing_import_count_fields(
    result: DependencyLookupResult,
    count: usize,
    partial_reason: Option<&str>,
) -> (Option<usize>, ImportCountConfidence, Option<&str>) {
    match result {
        DependencyLookupResult::Available => {
            if partial_reason.is_some() {
                (
                    Some(count),
                    ImportCountConfidence::SampleOnly,
                    partial_reason,
                )
            } else {
                (Some(count), ImportCountConfidence::Grounded, None)
            }
        }
        DependencyLookupResult::AvailableNoObservedImports => {
            (Some(0), ImportCountConfidence::Grounded, None)
        }
        DependencyLookupResult::AvailableImportGraphUnavailable => {
            (None, ImportCountConfidence::Unavailable, partial_reason)
        }
        DependencyLookupResult::NewPackage => {
            if partial_reason.is_some() {
                (
                    Some(count),
                    ImportCountConfidence::SampleOnly,
                    partial_reason,
                )
            } else {
                (Some(count), ImportCountConfidence::Grounded, None)
            }
        }
    }
}

fn dependency_root(specifier: &str) -> String {
    specifier
        .split("::")
        .next()
        .unwrap_or(specifier)
        .split('/')
        .next()
        .unwrap_or(specifier)
        .to_string()
}
