use lumin_rust_source_health::protocol::AstDefinitionKind;
use serde::Serialize;

use super::identity::{FunctionSignatureEvidence, Locality};
use super::suppression::PolicySupportingReason;
use crate::prewrite::index::MatchedField;
use crate::prewrite::operation::ServiceOperationFamily;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum ServiceOperationMuteReason {
    #[serde(rename = "service-sibling-insufficient-metadata")]
    InsufficientMetadata,
    #[serde(rename = "service-sibling-policy-excluded")]
    PolicyExcluded,
    #[serde(rename = "service-sibling-surface-kind-unsupported")]
    SurfaceKindUnsupported,
    #[serde(rename = "service-sibling-non-callable-definition")]
    NonCallableDefinition,
    #[serde(rename = "service-sibling-insufficient-suppressed-support")]
    InsufficientSuppressedSupport,
    #[serde(rename = "service-sibling-locality-mismatch")]
    LocalityMismatch,
    #[serde(rename = "service-sibling-unknown-operation")]
    UnknownOperation,
    #[serde(rename = "service-sibling-domain-mismatch")]
    DomainMismatch,
    #[serde(rename = "service-sibling-operation-family-mismatch")]
    OperationFamilyMismatch,
    #[serde(rename = "service-sibling-family-not-promotable")]
    FamilyNotPromotable,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum ServiceSuppressedLane {
    NearName,
    Semantic,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ServiceSignatureSupport {
    status: ServiceSignatureSupportStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<ServiceSignatureSupportReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matched_field: Option<ServiceSignatureSupportMatchedField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    normalized_version: Option<&'static str>,
}

impl ServiceSignatureSupport {
    pub(in crate::prewrite) fn unavailable() -> Self {
        Self {
            status: ServiceSignatureSupportStatus::Unavailable,
            reason: Some(ServiceSignatureSupportReason::NoSignatureFacts),
            artifact: None,
            matched_field: None,
            hash: None,
            normalized_version: None,
        }
    }

    pub(in crate::prewrite) fn from_evidence(evidence: Option<&FunctionSignatureEvidence>) -> Self {
        let Some(evidence) = evidence else {
            return Self::unavailable();
        };
        Self {
            status: ServiceSignatureSupportStatus::Grounded,
            reason: None,
            artifact: Some("rust-source-health"),
            matched_field: Some(ServiceSignatureSupportMatchedField::FunctionSignatureHash),
            hash: Some(evidence.hash.clone()),
            normalized_version: Some(evidence.normalized_version),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ServiceSignatureSupportStatus {
    Grounded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
enum ServiceSignatureSupportReason {
    #[serde(rename = "no-signature-facts")]
    NoSignatureFacts,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
enum ServiceSignatureSupportMatchedField {
    #[serde(rename = "files[].ast.functionSignatures[].hash")]
    FunctionSignatureHash,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ServiceOperationPolicyEntry {
    pub(in crate::prewrite) identity: String,
    pub(in crate::prewrite) name: String,
    pub(in crate::prewrite) owner_file: String,
    pub(in crate::prewrite) matched_field: MatchedField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) definition_kind: Option<AstDefinitionKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) reason: Option<ServiceOperationMuteReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) operation_family: Option<ServiceOperationFamily>,
    pub(in crate::prewrite) shared_domain_tokens: Vec<String>,
    pub(in crate::prewrite) supporting_reasons: Vec<PolicySupportingReason>,
    pub(in crate::prewrite) locality: Locality,
    pub(in crate::prewrite) signature_support: ServiceSignatureSupport,
    pub(in crate::prewrite) suppressed_lanes: Vec<ServiceSuppressedLane>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ServiceOperationSiblingPolicy {
    pub(in crate::prewrite) policy_id: &'static str,
    pub(in crate::prewrite) policy_version: &'static str,
    pub(in crate::prewrite) evaluated_candidate_count: usize,
    pub(in crate::prewrite) promoted_candidate_count: usize,
    pub(in crate::prewrite) muted_candidate_count: usize,
    pub(in crate::prewrite) promoted: Vec<ServiceOperationPolicyEntry>,
    pub(in crate::prewrite) muted: Vec<ServiceOperationPolicyEntry>,
}
