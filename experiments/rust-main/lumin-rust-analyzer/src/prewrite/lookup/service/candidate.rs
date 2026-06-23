use std::collections::BTreeMap;

use super::super::model::{PolicySupportingReason, ServiceSignatureSupport, ServiceSuppressedLane};
use super::super::{
    CandidateRecord, Locality, ServiceOperationFamily, ServiceOperationMuteReason,
    ServiceOperationPolicyEntry, SuppressedNearNameHint, SuppressedSemanticHint, SuppressionReason,
};

#[derive(Debug)]
pub(super) struct MergedServiceCandidate {
    pub(super) record: CandidateRecord,
    pub(super) locality: Locality,
    pub(super) supporting_reasons: Vec<SuppressionReason>,
    matched_tokens: Vec<String>,
    suppressed_lanes: Vec<ServiceSuppressedLane>,
}

impl MergedServiceCandidate {
    pub(super) fn into_entry(
        self,
        operation_family: Option<ServiceOperationFamily>,
        shared_domain_tokens: Vec<String>,
        reason: Option<ServiceOperationMuteReason>,
    ) -> ServiceOperationPolicyEntry {
        ServiceOperationPolicyEntry {
            identity: self.record.identity,
            name: self.record.name,
            owner_file: self.record.owner_file,
            matched_field: self.record.matched_field,
            definition_kind: self.record.definition_kind,
            reason,
            operation_family,
            shared_domain_tokens,
            supporting_reasons: self
                .supporting_reasons
                .into_iter()
                .map(PolicySupportingReason::from)
                .collect(),
            locality: self.locality,
            signature_support: ServiceSignatureSupport::unavailable(),
            suppressed_lanes: self.suppressed_lanes,
        }
    }
}

pub(super) fn merge_suppressed_policy_candidates(
    suppressed_near_names: &[SuppressedNearNameHint],
    suppressed_semantic_hints: &[SuppressedSemanticHint],
) -> Vec<MergedServiceCandidate> {
    let mut by_identity = BTreeMap::<String, MergedServiceCandidate>::new();
    for hint in suppressed_near_names {
        append_candidate(
            &mut by_identity,
            &hint.candidate,
            hint.locality,
            hint.reason,
            &hint.matched_tokens,
            ServiceSuppressedLane::NearName,
        );
    }
    for hint in suppressed_semantic_hints {
        append_candidate(
            &mut by_identity,
            &hint.candidate,
            hint.locality,
            hint.reason,
            &hint.matched_tokens,
            ServiceSuppressedLane::Semantic,
        );
    }

    let mut candidates = by_identity.into_values().collect::<Vec<_>>();
    for candidate in &mut candidates {
        candidate.supporting_reasons.sort_by(|left, right| {
            supporting_reason_rank(*left)
                .cmp(&supporting_reason_rank(*right))
                .then(reason_name(*left).cmp(reason_name(*right)))
        });
        candidate.supporting_reasons.dedup();
        candidate.suppressed_lanes.sort_by_key(|lane| match lane {
            ServiceSuppressedLane::NearName => 0,
            ServiceSuppressedLane::Semantic => 1,
        });
        candidate.suppressed_lanes.dedup();
    }
    candidates
}

fn append_candidate(
    by_identity: &mut BTreeMap<String, MergedServiceCandidate>,
    record: &CandidateRecord,
    locality: Locality,
    reason: SuppressionReason,
    matched_tokens: &[String],
    lane: ServiceSuppressedLane,
) {
    let entry = by_identity
        .entry(record.identity.clone())
        .or_insert_with(|| MergedServiceCandidate {
            record: record.clone(),
            locality,
            supporting_reasons: Vec::new(),
            matched_tokens: Vec::new(),
            suppressed_lanes: Vec::new(),
        });
    if locality.rank() > entry.locality.rank() {
        entry.locality = locality;
    }
    if !entry.supporting_reasons.contains(&reason) {
        entry.supporting_reasons.push(reason);
    }
    for token in matched_tokens {
        if !entry.matched_tokens.contains(token) {
            entry.matched_tokens.push(token.clone());
        }
    }
    if !entry.suppressed_lanes.contains(&lane) {
        entry.suppressed_lanes.push(lane);
    }
}

fn supporting_reason_rank(reason: SuppressionReason) -> usize {
    match reason {
        SuppressionReason::SingleNonWeakTokenOnly => 0,
        SuppressionReason::NearDistanceExceeded => 1,
        SuppressionReason::NearLengthDeltaExceeded => 2,
        SuppressionReason::DomainTokenOverlap => 3,
        SuppressionReason::NearPrefixMismatch => 4,
        SuppressionReason::InsufficientNonWeakSupport => 5,
    }
}

fn reason_name(reason: SuppressionReason) -> &'static str {
    match reason {
        SuppressionReason::DomainTokenOverlap => "domain-token-overlap",
        SuppressionReason::NearLengthDeltaExceeded => "near-length-delta-exceeded",
        SuppressionReason::NearPrefixMismatch => "near-prefix-mismatch",
        SuppressionReason::NearDistanceExceeded => "near-distance-exceeded",
        SuppressionReason::SingleNonWeakTokenOnly => "single-non-weak-token-only",
        SuppressionReason::InsufficientNonWeakSupport => "insufficient-non-weak-support",
    }
}
