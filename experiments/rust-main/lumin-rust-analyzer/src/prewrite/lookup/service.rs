use std::collections::BTreeSet;

mod candidate;
mod mute;
mod order;

use super::model::ServiceOperationSiblingPolicy;
use super::{SuppressedNearNameHint, SuppressedSemanticHint};
use crate::prewrite::operation::service_operation_info;

use candidate::merge_suppressed_policy_candidates;
use mute::service_mute_reason;
use order::sort_service_entries;

pub(in crate::prewrite) const SERVICE_OPERATION_POLICY_ID: &str =
    "prewrite-service-operation-sibling-cue";
pub(in crate::prewrite) const SERVICE_OPERATION_POLICY_VERSION: &str =
    "prewrite-service-operation-sibling-cue-v1";
pub(in crate::prewrite) const SERVICE_OPERATION_POLICY_MAX_RESULTS: usize = 5;

pub(super) fn service_operation_sibling_policy(
    intent_name: &str,
    suppressed_near_names: &[SuppressedNearNameHint],
    suppressed_semantic_hints: &[SuppressedSemanticHint],
) -> ServiceOperationSiblingPolicy {
    let mut candidates =
        merge_suppressed_policy_candidates(suppressed_near_names, suppressed_semantic_hints);
    if candidates.is_empty() {
        return empty_policy();
    }

    let intent_operation = service_operation_info(intent_name);
    let intent_domains = intent_operation
        .domain_tokens
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut promoted = Vec::new();
    let mut muted = Vec::new();

    for candidate in candidates.drain(..) {
        let candidate_operation = service_operation_info(&candidate.record.name);
        let candidate_domains = candidate_operation
            .domain_tokens
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let shared_domain_tokens = intent_operation
            .domain_tokens
            .iter()
            .filter(|token| candidate_domains.contains(*token))
            .cloned()
            .collect::<Vec<_>>();

        let reason = service_mute_reason(
            &candidate,
            &intent_operation,
            &intent_domains,
            &candidate_operation,
            &shared_domain_tokens,
        );
        let entry = candidate.into_entry(
            candidate_operation.operation_family,
            shared_domain_tokens,
            reason,
        );
        if reason.is_some() {
            muted.push(entry);
        } else {
            promoted.push(entry);
        }
    }

    sort_service_entries(&mut promoted);
    sort_service_entries(&mut muted);
    let promoted_candidate_count = promoted.len();
    let muted_candidate_count = muted.len();
    promoted.truncate(SERVICE_OPERATION_POLICY_MAX_RESULTS);
    muted.truncate(SERVICE_OPERATION_POLICY_MAX_RESULTS);

    ServiceOperationSiblingPolicy {
        policy_id: SERVICE_OPERATION_POLICY_ID,
        policy_version: SERVICE_OPERATION_POLICY_VERSION,
        evaluated_candidate_count: promoted_candidate_count + muted_candidate_count,
        promoted_candidate_count,
        muted_candidate_count,
        promoted,
        muted,
    }
}

fn empty_policy() -> ServiceOperationSiblingPolicy {
    ServiceOperationSiblingPolicy {
        policy_id: SERVICE_OPERATION_POLICY_ID,
        policy_version: SERVICE_OPERATION_POLICY_VERSION,
        evaluated_candidate_count: 0,
        promoted_candidate_count: 0,
        muted_candidate_count: 0,
        promoted: Vec::new(),
        muted: Vec::new(),
    }
}
