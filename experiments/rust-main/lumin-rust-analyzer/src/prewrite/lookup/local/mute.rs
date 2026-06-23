use std::collections::BTreeSet;

use lumin_rust_source_health::protocol::PathClassification;

use super::super::model::{LocalOperationMuteReason, Locality};
use super::super::ServiceOperationFamily;
use crate::prewrite::index::LocalOperationCandidate;

pub(super) fn local_mute_reason(
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
