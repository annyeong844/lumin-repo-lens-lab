use std::collections::BTreeSet;
use std::path::Path;

use lumin_rust_source_health::protocol::PathClassification;

use super::model::{
    LocalOperationMuteReason, LocalOperationPolicyEntry, LocalOperationPolicyStatus,
    LocalOperationSiblingPolicy, Locality, PolicySupportingReason,
};
use super::ServiceOperationFamily;
use crate::prewrite::index::LocalOperationCandidate;
use crate::prewrite::operation::service_operation_info;

const LOCAL_OPERATION_POLICY_ID: &str = "prewrite-local-operation-sibling";
const LOCAL_OPERATION_POLICY_VERSION: &str = "prewrite-local-operation-sibling-v1";
const LOCAL_OPERATION_POLICY_MAX_RESULTS: usize = 5;
const INTENT_OWNER_FILE_MISSING: &str = "intent-owner-file-missing";

pub(super) fn local_operation_sibling_policy(
    intent_name: &str,
    owner_file: Option<&str>,
    local_operations: &[LocalOperationCandidate<'_>],
) -> LocalOperationSiblingPolicy {
    let Some(owner_file) = owner_file else {
        return empty_policy(Some(INTENT_OWNER_FILE_MISSING));
    };

    let intent_operation = service_operation_info(intent_name);
    let intent_domains = intent_operation
        .domain_tokens
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut promoted = Vec::new();
    let mut muted = Vec::new();

    for candidate in local_operations
        .iter()
        .filter(|candidate| candidate.file == owner_file)
    {
        let candidate_domains = candidate
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
        let locality = locality_for(owner_file, candidate.file);
        let reason = local_mute_reason(
            candidate,
            intent_operation.operation_family,
            &intent_domains,
            &shared_domain_tokens,
            locality,
        );
        let supporting_reasons = if reason.is_none() {
            vec![PolicySupportingReason::LocalOperationSameFileDomainOverlap]
        } else {
            Vec::new()
        };
        let entry = LocalOperationPolicyEntry::from_candidate(
            candidate,
            shared_domain_tokens,
            supporting_reasons,
            reason,
            locality,
        );
        if reason.is_some() {
            muted.push(entry);
        } else {
            promoted.push(entry);
        }
    }

    sort_local_entries(&mut promoted);
    sort_local_entries(&mut muted);
    let promoted_candidate_count = promoted.len();
    let muted_candidate_count = muted.len();
    promoted.truncate(LOCAL_OPERATION_POLICY_MAX_RESULTS);
    muted.truncate(LOCAL_OPERATION_POLICY_MAX_RESULTS);

    LocalOperationSiblingPolicy {
        policy_id: LOCAL_OPERATION_POLICY_ID,
        policy_version: LOCAL_OPERATION_POLICY_VERSION,
        status: LocalOperationPolicyStatus::Complete,
        reason: None,
        evaluated_candidate_count: promoted_candidate_count + muted_candidate_count,
        promoted_candidate_count,
        muted_candidate_count,
        promoted,
        muted,
    }
}

fn empty_policy(reason: Option<&'static str>) -> LocalOperationSiblingPolicy {
    LocalOperationSiblingPolicy {
        policy_id: LOCAL_OPERATION_POLICY_ID,
        policy_version: LOCAL_OPERATION_POLICY_VERSION,
        status: LocalOperationPolicyStatus::Complete,
        reason,
        evaluated_candidate_count: 0,
        promoted_candidate_count: 0,
        muted_candidate_count: 0,
        promoted: Vec::new(),
        muted: Vec::new(),
    }
}

fn local_mute_reason(
    candidate: &LocalOperationCandidate<'_>,
    intent_family: Option<ServiceOperationFamily>,
    intent_domains: &BTreeSet<String>,
    shared_domain_tokens: &[String],
    locality: Locality,
) -> Option<LocalOperationMuteReason> {
    if candidate.identity().is_empty()
        || candidate.name.is_empty()
        || candidate.file.is_empty()
        || candidate.container_name.is_empty()
    {
        Some(LocalOperationMuteReason::InsufficientMetadata)
    } else if candidate.container_kind != "function-declaration" {
        Some(LocalOperationMuteReason::SurfaceKindUnsupported)
    } else if is_policy_excluded(candidate) {
        Some(LocalOperationMuteReason::PolicyExcluded)
    } else if !locality.same_file {
        Some(LocalOperationMuteReason::LocalityMismatch)
    } else if intent_family.is_none() {
        Some(LocalOperationMuteReason::UnknownOperation)
    } else if intent_domains.is_empty() || shared_domain_tokens.is_empty() {
        Some(LocalOperationMuteReason::DomainMismatch)
    } else if intent_family != Some(candidate.operation_family) {
        Some(LocalOperationMuteReason::FamilyMismatch)
    } else if intent_family != Some(ServiceOperationFamily::ReadQuery) {
        Some(LocalOperationMuteReason::FamilyNotPromotable)
    } else {
        None
    }
}

fn is_policy_excluded(candidate: &LocalOperationCandidate<'_>) -> bool {
    candidate.path.suppressed
        || candidate.classifications().iter().any(|classification| {
            matches!(
                classification,
                PathClassification::Test | PathClassification::Generated
            )
        })
}

fn locality_for(owner_file: &str, candidate_file: &str) -> Locality {
    Locality {
        same_file: owner_file == candidate_file,
        same_dir: parent_path(owner_file) == parent_path(candidate_file),
    }
}

fn parent_path(path: &str) -> Option<&Path> {
    Path::new(path).parent()
}

fn sort_local_entries(entries: &mut [LocalOperationPolicyEntry]) {
    entries.sort_by(|left, right| {
        right
            .locality
            .rank()
            .cmp(&left.locality.rank())
            .then(
                left.operation_family
                    .as_str()
                    .cmp(right.operation_family.as_str()),
            )
            .then(left.name.cmp(&right.name))
            .then(left.owner_file.cmp(&right.owner_file))
            .then(left.identity.cmp(&right.identity))
    });
}
