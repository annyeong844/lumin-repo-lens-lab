use lumin_rust_source_health::protocol::HealthResponse;

use super::index::{CandidateIndex, CandidateLane};
use super::intent::{NameDeclaration, NormalizedIntent};

mod file;
mod local;
mod model;
mod near;
mod semantic;
mod service;
mod taint;

pub(super) use super::operation::ServiceOperationFamily;
pub(super) use file::{lookup_files, FileLookup, FileLookupResult};
use model::LookupResult;
pub(super) use model::{
    CandidateRecord, LocalOperationMuteReason, LocalOperationPolicyEntry, Locality, NameLookup,
    NearNameHint, PolicySupportingReason, SemanticHint, ServiceOperationMuteReason,
    ServiceOperationPolicyEntry, SuppressedNearNameHint, SuppressedSemanticHint, SuppressionReason,
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
            candidate.lane == CandidateLane::Definition && candidate.name == intent_name
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
        .map(|identity| {
            format!(
                "[grounded, rust-source-health.files['{}'].ast.definitions contains '{}' at line {}]",
                identity.owner_file, identity.name, identity.line
            )
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
