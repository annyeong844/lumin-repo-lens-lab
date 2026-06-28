use std::collections::BTreeMap;

use crate::prewrite::cues::model::{
    Cue, CueCandidate, CueCardBuilder, CueClaim, CueConfidence, CueEvidence, CueMatchedField,
    CueTier, EvidenceLane, MutedReason, SuppressedCue,
};
use crate::prewrite::lookup::{LocalOperationPolicyEntry, NameLookup};

use super::super::add_cue_for_candidate;

pub(super) fn add_local_operation_sibling_policy(
    lookup: &NameLookup,
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
) {
    let policy = &lookup.local_operation_sibling_policy;
    for entry in &policy.promoted {
        add_cue_for_candidate(
            cards,
            CueCandidate::from(entry),
            local_operation_cue(policy.policy_id, policy.policy_version, entry),
        );
    }
    suppressed.extend(policy.muted.iter().map(|entry| {
        local_operation_muted_cue(
            policy.policy_id,
            policy.policy_version,
            policy.evaluated_candidate_count,
            entry,
            entry
                .reason
                .map(MutedReason::from)
                .unwrap_or(MutedReason::LocalOperationInsufficientMetadata),
        )
    }));
}

fn local_operation_cue(
    policy_id: &'static str,
    policy_version: &'static str,
    entry: &LocalOperationPolicyEntry,
) -> Cue {
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: EvidenceLane::LocalOperationSibling,
        claim: CueClaim::RelatedLocalServiceOperation,
        confidence: CueConfidence::HeuristicReview,
        evidence: vec![CueEvidence {
            artifact: "pre-write-advisory.json",
            matched_field: CueMatchedField::LocalOperationSiblingPolicyPromoted,
            matched_field_source: Some(entry.matched_field),
            algorithm_version: None,
            hash: None,
            visibility: None,
            local_name: None,
            candidate_identity: entry.identity.clone(),
            file: None,
            file_lookup_result: None,
            dependency_lookup_result: None,
            observed_import_count: None,
            consumer_threshold: None,
            distance: None,
            tokens: Vec::new(),
            policy_id: Some(policy_id),
            policy_version: Some(policy_version),
            operation_family: Some(entry.operation_family),
            shared_domain_tokens: entry.shared_domain_tokens.clone(),
            locality: Some(entry.locality),
            supporting_reasons: entry.supporting_reasons.clone(),
            surface_kind: Some(entry.surface_kind),
            container_name: Some(entry.container_name.clone()),
            container_kind: Some(entry.container_kind),
        }],
    }
}

fn local_operation_muted_cue(
    policy_id: &'static str,
    policy_version: &'static str,
    candidate_count: usize,
    entry: &LocalOperationPolicyEntry,
    reason: MutedReason,
) -> SuppressedCue {
    SuppressedCue {
        cue_tier: CueTier::Muted,
        original_cue_tier: None,
        evidence_lane: EvidenceLane::LocalOperationSibling,
        reason,
        candidate: CueCandidate::from(entry),
        path_classifications: Vec::new(),
        tokens: Vec::new(),
        distance: None,
        score: None,
        candidate_count,
        policy_id: Some(policy_id),
        policy_version: Some(policy_version),
        matched_field: Some(entry.matched_field),
        operation_family: Some(entry.operation_family),
        shared_domain_tokens: entry.shared_domain_tokens.clone(),
        supporting_reasons: entry.supporting_reasons.clone(),
        locality: Some(entry.locality),
        surface_kind: Some(entry.surface_kind),
        container_name: Some(entry.container_name.clone()),
        container_kind: Some(entry.container_kind),
    }
}
