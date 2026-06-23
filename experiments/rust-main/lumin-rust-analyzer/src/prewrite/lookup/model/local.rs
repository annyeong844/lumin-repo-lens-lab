use serde::Serialize;

use super::identity::Locality;
use super::service::ServiceSignatureSupport;
use super::suppression::PolicySupportingReason;
use crate::prewrite::index::{LocalOperationCandidate, MatchedField};
use crate::prewrite::operation::ServiceOperationFamily;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum LocalOperationMuteReason {
    #[serde(rename = "local-operation-insufficient-metadata")]
    InsufficientMetadata,
    #[serde(rename = "local-operation-surface-kind-unsupported")]
    SurfaceKindUnsupported,
    #[serde(rename = "local-operation-policy-excluded")]
    PolicyExcluded,
    #[serde(rename = "local-operation-locality-mismatch")]
    LocalityMismatch,
    #[serde(rename = "local-operation-unknown-operation")]
    UnknownOperation,
    #[serde(rename = "local-operation-domain-mismatch")]
    DomainMismatch,
    #[serde(rename = "local-operation-family-mismatch")]
    FamilyMismatch,
    #[serde(rename = "local-operation-family-not-promotable")]
    FamilyNotPromotable,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum LocalOperationPolicyStatus {
    Complete,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct LocalOperationPolicyEntry {
    pub(in crate::prewrite) identity: String,
    pub(in crate::prewrite) name: String,
    pub(in crate::prewrite) owner_file: String,
    pub(in crate::prewrite) matched_field: MatchedField,
    pub(in crate::prewrite) surface_kind: &'static str,
    pub(in crate::prewrite) operation_family: ServiceOperationFamily,
    pub(in crate::prewrite) shared_domain_tokens: Vec<String>,
    pub(in crate::prewrite) locality: Locality,
    pub(in crate::prewrite) eligible_for_dead_export_ranking: bool,
    pub(in crate::prewrite) eligible_for_safe_fix: bool,
    pub(in crate::prewrite) signature_support: ServiceSignatureSupport,
    pub(in crate::prewrite) supporting_reasons: Vec<PolicySupportingReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) reason: Option<LocalOperationMuteReason>,
    pub(in crate::prewrite) container_name: String,
    pub(in crate::prewrite) container_kind: &'static str,
    pub(in crate::prewrite) line: usize,
    pub(in crate::prewrite) container_line: usize,
    pub(in crate::prewrite) domain_tokens: Vec<String>,
}

impl LocalOperationPolicyEntry {
    pub(in crate::prewrite) fn from_candidate(
        candidate: &LocalOperationCandidate<'_>,
        domain_tokens: Vec<String>,
        shared_domain_tokens: Vec<String>,
        supporting_reasons: Vec<PolicySupportingReason>,
        reason: Option<LocalOperationMuteReason>,
        locality: Locality,
    ) -> Self {
        Self {
            identity: candidate.identity(),
            name: candidate.name.to_string(),
            owner_file: candidate.file.to_string(),
            matched_field: MatchedField::PreWriteLocalOperation,
            surface_kind: "nested-local-operation",
            operation_family: candidate.operation_family,
            shared_domain_tokens,
            locality,
            eligible_for_dead_export_ranking: false,
            eligible_for_safe_fix: false,
            signature_support: ServiceSignatureSupport::unavailable(),
            supporting_reasons,
            reason,
            container_name: candidate.container_name.to_string(),
            container_kind: candidate.container_kind,
            line: candidate.location.line,
            container_line: candidate.container_location.line,
            domain_tokens,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct LocalOperationSiblingPolicy {
    pub(in crate::prewrite) policy_id: &'static str,
    pub(in crate::prewrite) policy_version: &'static str,
    pub(in crate::prewrite) status: LocalOperationPolicyStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) reason: Option<&'static str>,
    pub(in crate::prewrite) evaluated_candidate_count: usize,
    pub(in crate::prewrite) promoted_candidate_count: usize,
    pub(in crate::prewrite) muted_candidate_count: usize,
    pub(in crate::prewrite) promoted: Vec<LocalOperationPolicyEntry>,
    pub(in crate::prewrite) muted: Vec<LocalOperationPolicyEntry>,
}
