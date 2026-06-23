use lumin_rust_source_health::protocol::PathClassification;
use serde::Serialize;

use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::{
    LocalOperationMuteReason, Locality, PolicySupportingReason, ServiceOperationFamily,
    ServiceOperationMuteReason, SuppressionReason,
};

use super::candidate::CueCandidate;
use super::vocabulary::{CueTier, EvidenceLane};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum MutedReason {
    PolicyExcluded,
    DomainTokenOverlap,
    NearLengthDeltaExceeded,
    NearPrefixMismatch,
    NearDistanceExceeded,
    SingleNonWeakTokenOnly,
    InsufficientNonWeakSupport,
    ServiceSiblingInsufficientMetadata,
    ServiceSiblingPolicyExcluded,
    ServiceSiblingSurfaceKindUnsupported,
    ServiceSiblingClassMethodLane,
    ServiceSiblingNonCallableDefinition,
    ServiceSiblingInsufficientSuppressedSupport,
    ServiceSiblingLocalityMismatch,
    ServiceSiblingUnknownOperation,
    ServiceSiblingDomainMismatch,
    ServiceSiblingOperationFamilyMismatch,
    ServiceSiblingFamilyNotPromotable,
    LocalOperationInsufficientMetadata,
    LocalOperationSurfaceKindUnsupported,
    LocalOperationPolicyExcluded,
    LocalOperationLocalityMismatch,
    LocalOperationUnknownOperation,
    LocalOperationDomainMismatch,
    LocalOperationFamilyMismatch,
    LocalOperationFamilyNotPromotable,
}

impl From<SuppressionReason> for MutedReason {
    fn from(reason: SuppressionReason) -> Self {
        match reason {
            SuppressionReason::DomainTokenOverlap => Self::DomainTokenOverlap,
            SuppressionReason::NearLengthDeltaExceeded => Self::NearLengthDeltaExceeded,
            SuppressionReason::NearPrefixMismatch => Self::NearPrefixMismatch,
            SuppressionReason::NearDistanceExceeded => Self::NearDistanceExceeded,
            SuppressionReason::SingleNonWeakTokenOnly => Self::SingleNonWeakTokenOnly,
            SuppressionReason::InsufficientNonWeakSupport => Self::InsufficientNonWeakSupport,
        }
    }
}

impl From<ServiceOperationMuteReason> for MutedReason {
    fn from(reason: ServiceOperationMuteReason) -> Self {
        match reason {
            ServiceOperationMuteReason::InsufficientMetadata => {
                Self::ServiceSiblingInsufficientMetadata
            }
            ServiceOperationMuteReason::PolicyExcluded => Self::ServiceSiblingPolicyExcluded,
            ServiceOperationMuteReason::SurfaceKindUnsupported => {
                Self::ServiceSiblingSurfaceKindUnsupported
            }
            ServiceOperationMuteReason::NonCallableDefinition => {
                Self::ServiceSiblingNonCallableDefinition
            }
            ServiceOperationMuteReason::InsufficientSuppressedSupport => {
                Self::ServiceSiblingInsufficientSuppressedSupport
            }
            ServiceOperationMuteReason::LocalityMismatch => Self::ServiceSiblingLocalityMismatch,
            ServiceOperationMuteReason::UnknownOperation => Self::ServiceSiblingUnknownOperation,
            ServiceOperationMuteReason::DomainMismatch => Self::ServiceSiblingDomainMismatch,
            ServiceOperationMuteReason::OperationFamilyMismatch => {
                Self::ServiceSiblingOperationFamilyMismatch
            }
            ServiceOperationMuteReason::FamilyNotPromotable => {
                Self::ServiceSiblingFamilyNotPromotable
            }
        }
    }
}

impl From<LocalOperationMuteReason> for MutedReason {
    fn from(reason: LocalOperationMuteReason) -> Self {
        match reason {
            LocalOperationMuteReason::InsufficientMetadata => {
                Self::LocalOperationInsufficientMetadata
            }
            LocalOperationMuteReason::SurfaceKindUnsupported => {
                Self::LocalOperationSurfaceKindUnsupported
            }
            LocalOperationMuteReason::PolicyExcluded => Self::LocalOperationPolicyExcluded,
            LocalOperationMuteReason::LocalityMismatch => Self::LocalOperationLocalityMismatch,
            LocalOperationMuteReason::UnknownOperation => Self::LocalOperationUnknownOperation,
            LocalOperationMuteReason::DomainMismatch => Self::LocalOperationDomainMismatch,
            LocalOperationMuteReason::FamilyMismatch => Self::LocalOperationFamilyMismatch,
            LocalOperationMuteReason::FamilyNotPromotable => {
                Self::LocalOperationFamilyNotPromotable
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct SuppressedCue {
    pub(in crate::prewrite::cues) cue_tier: CueTier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) original_cue_tier: Option<CueTier>,
    pub(in crate::prewrite::cues) evidence_lane: EvidenceLane,
    pub(in crate::prewrite) reason: MutedReason,
    pub(in crate::prewrite) candidate: CueCandidate,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite) path_classifications: Vec<PathClassification>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite::cues) tokens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) distance: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) score: Option<usize>,
    pub(in crate::prewrite::cues) candidate_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) policy_id: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) policy_version: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) matched_field: Option<MatchedField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) operation_family: Option<ServiceOperationFamily>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite::cues) shared_domain_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite::cues) supporting_reasons: Vec<PolicySupportingReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) locality: Option<Locality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) surface_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) container_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) container_kind: Option<&'static str>,
}
