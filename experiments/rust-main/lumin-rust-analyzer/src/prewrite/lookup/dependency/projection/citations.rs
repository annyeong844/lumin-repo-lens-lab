use super::super::graph::DependencyImportGraph;
use super::super::manifest::CargoDependencyDeclaration;
use super::super::model::DependencyLookupResult;

pub(super) fn declaration_citations(
    dependency: &str,
    declaration: Option<&CargoDependencyDeclaration>,
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

pub(super) fn push_lookup_result_citation(
    dependency: &str,
    result: DependencyLookupResult,
    observed_import_count: usize,
    graph: &DependencyImportGraph,
    citations: &mut Vec<String>,
) {
    match result {
        DependencyLookupResult::Available => {
            citations.push(format!(
                "[grounded, Rust AST static import graph observed {observed_import_count} consumer(s) for '{dependency}']"
            ));
        }
        DependencyLookupResult::AvailableNoObservedImports => {
            citations.push(format!(
                "[확인 불가, scan range: Rust AST import graph only; '{dependency}' may still be consumed by build scripts, cfg-gated code, generated code, runtime plugins, examples, or external cargo commands]"
            ));
        }
        DependencyLookupResult::AvailableImportGraphUnavailable => {
            let reason = graph.zero_observed_unavailable_reason();
            citations.push(format!(
                "[확인 불가, reason: {reason}; zero observed Rust path consumers is not a grounded absence claim]"
            ));
        }
        DependencyLookupResult::NewPackage => {}
    }
}
