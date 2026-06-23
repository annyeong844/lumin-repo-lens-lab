use std::collections::BTreeSet;

use lumin_rust_source_health::protocol::AstDefinitionKind;

use super::super::{ServiceOperationFamily, ServiceOperationMuteReason, SuppressionReason};
use super::candidate::MergedServiceCandidate;
use crate::prewrite::index::MatchedField;
use crate::prewrite::operation::OperationInfo;

pub(super) fn service_mute_reason(
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
