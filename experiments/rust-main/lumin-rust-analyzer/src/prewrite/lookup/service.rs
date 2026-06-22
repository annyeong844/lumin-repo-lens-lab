use std::collections::{BTreeMap, BTreeSet};

use lumin_rust_source_health::protocol::AstDefinitionKind;

use super::model::{
    PolicySupportingReason, ServiceOperationSiblingPolicy, ServiceSignatureSupport,
    ServiceSuppressedLane,
};
use super::{
    CandidateRecord, Locality, ServiceOperationFamily, ServiceOperationMuteReason,
    ServiceOperationPolicyEntry, SuppressedNearNameHint, SuppressedSemanticHint, SuppressionReason,
};
use crate::prewrite::index::MatchedField;
use crate::prewrite::operation::{service_operation_info, OperationInfo};

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

fn service_mute_reason(
    candidate: &MergedServiceCandidate,
    intent_operation: &OperationInfo,
    intent_domains: &BTreeSet<String>,
    candidate_operation: &OperationInfo,
    shared_domain_tokens: &[String],
) -> Option<ServiceOperationMuteReason> {
    let has_promotable_suppression = candidate.supporting_reasons.iter().any(|reason| {
        matches!(
            reason,
            SuppressionReason::SingleNonWeakTokenOnly
                | SuppressionReason::NearDistanceExceeded
                | SuppressionReason::NearLengthDeltaExceeded
        )
    });

    if candidate.record.identity.is_empty()
        || candidate.record.name.is_empty()
        || candidate.record.owner_file.is_empty()
    {
        Some(ServiceOperationMuteReason::InsufficientMetadata)
    } else if candidate.record.policy_excluded {
        Some(ServiceOperationMuteReason::PolicyExcluded)
    } else if candidate.record.matched_field != MatchedField::Def {
        Some(ServiceOperationMuteReason::SurfaceKindUnsupported)
    } else if is_non_callable_service_definition(candidate.record.definition_kind) {
        Some(ServiceOperationMuteReason::NonCallableDefinition)
    } else if !has_promotable_suppression {
        Some(ServiceOperationMuteReason::InsufficientSuppressedSupport)
    } else if !candidate.locality.same_file && !candidate.locality.same_dir {
        Some(ServiceOperationMuteReason::LocalityMismatch)
    } else if intent_operation.operation_family.is_none()
        || candidate_operation.operation_family.is_none()
    {
        Some(ServiceOperationMuteReason::UnknownOperation)
    } else if intent_domains.is_empty() || shared_domain_tokens.is_empty() {
        Some(ServiceOperationMuteReason::DomainMismatch)
    } else if intent_operation.operation_family != candidate_operation.operation_family {
        Some(ServiceOperationMuteReason::OperationFamilyMismatch)
    } else if intent_operation.operation_family != Some(ServiceOperationFamily::ReadQuery) {
        Some(ServiceOperationMuteReason::FamilyNotPromotable)
    } else {
        None
    }
}

fn is_non_callable_service_definition(kind: Option<AstDefinitionKind>) -> bool {
    !matches!(kind, Some(AstDefinitionKind::Function))
}

fn merge_suppressed_policy_candidates(
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

fn sort_service_entries(entries: &mut [ServiceOperationPolicyEntry]) {
    entries.sort_by(|left, right| {
        right
            .locality
            .rank()
            .cmp(&left.locality.rank())
            .then(
                operation_family_name(left.operation_family)
                    .cmp(operation_family_name(right.operation_family)),
            )
            .then(left.name.cmp(&right.name))
            .then(left.owner_file.cmp(&right.owner_file))
            .then(left.identity.cmp(&right.identity))
    });
}

fn operation_family_name(family: Option<ServiceOperationFamily>) -> &'static str {
    family.map(ServiceOperationFamily::as_str).unwrap_or("")
}

#[derive(Debug)]
struct MergedServiceCandidate {
    record: CandidateRecord,
    locality: Locality,
    supporting_reasons: Vec<SuppressionReason>,
    matched_tokens: Vec<String>,
    suppressed_lanes: Vec<ServiceSuppressedLane>,
}

impl MergedServiceCandidate {
    fn into_entry(
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
