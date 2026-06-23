use lumin_rust_source_health::protocol::HealthResponse;

use super::index::{CandidateIndex, CandidateLane};
use super::intent::{NameDeclaration, NormalizedIntent};

mod dependency;
mod file;
mod inline_pattern;
mod local;
mod model;
mod near;
mod semantic;
mod service;
mod shape;
mod taint;

pub(super) use super::operation::ServiceOperationFamily;
pub(super) use dependency::{
    lookup_dependencies, DependencyLookup, DependencyLookupResult, DEPENDENCY_EXAMPLE_LIMIT,
    DEPENDENCY_WATCH_FOR_THRESHOLD,
};
pub(super) use file::{
    lookup_files, FileLookup, FileLookupResult, DOMAIN_CLUSTER_MAX_EXAMPLES,
    DOMAIN_CLUSTER_MIN_MATCHES, DOMAIN_CLUSTER_MIN_PREFIX_LEN,
};
pub(super) use inline_pattern::{
    lookup_inline_patterns, unavailable_evidence_from_inline_pattern_lookups, InlinePatternLookup,
};
pub(super) use local::{
    LOCAL_OPERATION_POLICY_ID, LOCAL_OPERATION_POLICY_MAX_RESULTS, LOCAL_OPERATION_POLICY_VERSION,
};
use model::LookupResult;
pub(super) use model::{
    CandidateRecord, LocalOperationMuteReason, LocalOperationPolicyEntry, Locality, NameLookup,
    NearNameHint, PolicySupportingReason, SemanticHint, ServiceOperationMuteReason,
    ServiceOperationPolicyEntry, SuppressedNearNameHint, SuppressedSemanticHint, SuppressionReason,
};
pub(super) use near::{
    NEAR_NAME_MAX_DISTANCE, NEAR_NAME_MAX_LENGTH_DELTA, NEAR_NAME_MAX_RESULTS,
    NEAR_NAME_SHARED_PREFIX_MIN,
};
pub(super) use semantic::{SEMANTIC_HINT_MAX_RESULTS, SEMANTIC_HINT_MIN_SCORE};
pub(super) use service::{
    SERVICE_OPERATION_POLICY_ID, SERVICE_OPERATION_POLICY_MAX_RESULTS,
    SERVICE_OPERATION_POLICY_VERSION,
};
pub(super) use shape::{
    lookup_shapes, unavailable_evidence_from_shape_lookups, ShapeLookup, ShapeMatch,
    UnavailableEvidence,
};

pub(super) fn lookup_names(
    intent: &NormalizedIntent,
    index: &CandidateIndex<'_>,
    syntax: &HealthResponse,
) -> Vec<NameLookup> {
    intent
        .names
        .iter()
        .map(|name| lookup_name(name, intent.declaration_for(name), index, syntax))
        .collect()
}

fn lookup_name(
    intent_name: &str,
    declaration: Option<&NameDeclaration>,
    index: &CandidateIndex<'_>,
    syntax: &HealthResponse,
) -> NameLookup {
    let identities = index
        .candidates
        .iter()
        .copied()
        .filter(|candidate| {
            matches!(
                candidate.lane,
                CandidateLane::Definition | CandidateLane::UseTree
            ) && candidate.name == intent_name
        })
        .map(CandidateRecord::from_candidate)
        .collect::<Vec<_>>();
    let result = match identities.len() {
        0 => LookupResult::NotObserved,
        1 => LookupResult::Exists,
        _ => LookupResult::ExistsMultiple,
    };
    let owner_file = declaration.and_then(NameDeclaration::effective_owner_file);
    let (near_names, suppressed_near_names, suppressed_near_name_count) = if identities.is_empty() {
        near::near_name_candidates(intent_name, owner_file, &index.candidates)
    } else {
        (Vec::new(), Vec::new(), 0)
    };
    let (intent_tokens, semantic_hints, suppressed_semantic_hints, suppressed_semantic_hint_count) =
        if identities.is_empty() {
            semantic::semantic_hint_candidates(intent_name, declaration, &index.candidates)
        } else {
            (
                semantic::query_tokens(intent_name, declaration),
                Vec::new(),
                Vec::new(),
                0,
            )
        };
    let service_operation_sibling_policy = service::service_operation_sibling_policy(
        intent_name,
        &suppressed_near_names,
        &suppressed_semantic_hints,
    );
    let local_operation_sibling_policy =
        local::local_operation_sibling_policy(intent_name, owner_file, &index.local_operations);

    let mut citations = identities
        .iter()
        .map(|identity| match identity.matched_field {
            super::index::MatchedField::Def => format!(
                "[grounded, rust-source-health.files['{}'].ast.definitions contains '{}' at line {}]",
                identity.owner_file, identity.name, identity.line
            ),
            super::index::MatchedField::UseTree => format!(
                "[grounded, rust-source-health.files['{}'].ast.useTrees contains '{}' at line {}]",
                identity.owner_file, identity.name, identity.line
            ),
            super::index::MatchedField::ImplMethod
            | super::index::MatchedField::PreWriteLocalOperation => format!(
                "[grounded, rust-source-health.files['{}'] contains '{}' at line {}]",
                identity.owner_file, identity.name, identity.line
            ),
        })
        .collect::<Vec<_>>();
    if !near_names.is_empty() {
        citations.push(
            "[degraded, fuzzy-name match; source: Rust AST definition/impl-method scan; search hint only]"
                .to_string(),
        );
    }
    if !semantic_hints.is_empty() {
        citations.push(
            "[degraded, intent-token match; source: Rust AST owner/name tokens; search hint only]"
                .to_string(),
        );
    }
    if identities.is_empty() && near_names.is_empty() && semantic_hints.is_empty() {
        citations.push(format!(
            "[확인 불가, Rust AST scan did not observe '{intent_name}'; this is not an absence claim]"
        ));
    }

    NameLookup {
        intent_name: intent_name.to_string(),
        result,
        identities,
        intent_tokens,
        near_names,
        semantic_hints,
        suppressed_near_names,
        suppressed_near_name_count,
        suppressed_semantic_hints,
        suppressed_semantic_hint_count,
        service_operation_sibling_policy,
        local_operation_sibling_policy,
        tainted_by: (result == LookupResult::NotObserved)
            .then(|| taint::taint_summary(syntax))
            .flatten(),
        citations,
    }
}
