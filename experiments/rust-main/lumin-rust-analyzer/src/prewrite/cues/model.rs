use lumin_rust_source_health::protocol::PathClassification;
use serde::Serialize;

use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::{
    CandidateRecord, FileLookupResult, LocalOperationMuteReason, LocalOperationPolicyEntry,
    Locality, PolicySupportingReason, ServiceOperationFamily, ServiceOperationMuteReason,
    ServiceOperationPolicyEntry, SuppressionReason,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(in crate::prewrite) enum CueTier {
    #[serde(rename = "SAFE_CUE")]
    Safe,
    #[serde(rename = "AGENT_REVIEW_CUE")]
    AgentReview,
    #[serde(rename = "MUTED_CUE")]
    Muted,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum EvidenceLane {
    ExactSymbol,
    ExactFile,
    ImplMethodName,
    NearName,
    IntentToken,
    FileDomainCluster,
    ServiceOperationSibling,
    LocalOperationSibling,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite::cues) enum CueConfidence {
    Grounded,
    HeuristicReview,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite::cues) enum SafeMeaning {
    ClaimOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite::cues) enum NotSafeFor {
    SemanticEquivalence,
    AutoReuse,
    AutoFix,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite::cues) enum CueClaim {
    ExactRustDefinitionExists,
    #[serde(rename = "exact file exists")]
    ExactFileExists,
    NearRustDefinitionName,
    NearRustImplMethodName,
    SupportedIntentTokenOverlap,
    RustImplMethodIntentTokenOverlap,
    #[serde(rename = "related Rust file domain cluster")]
    RelatedRustFileDomainCluster,
    #[serde(rename = "related service operation sibling")]
    RelatedServiceOperationSibling,
    #[serde(rename = "related local service operation")]
    RelatedLocalServiceOperation,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum CueMatchedField {
    #[serde(rename = "defIndex")]
    DefIndex,
    #[serde(rename = "implMethodIndex")]
    ImplMethodIndex,
    #[serde(rename = "files")]
    RustSourceHealthFiles,
    #[serde(rename = "fileLookups[].domainCluster")]
    FileDomainCluster,
    #[serde(rename = "preWriteLocalOperationIndex")]
    PreWriteLocalOperationIndex,
    #[serde(rename = "lookups[].serviceOperationSiblingPolicy.promoted")]
    ServiceOperationSiblingPolicyPromoted,
    #[serde(rename = "lookups[].localOperationSiblingPolicy.promoted")]
    LocalOperationSiblingPolicyPromoted,
}

impl From<MatchedField> for CueMatchedField {
    fn from(field: MatchedField) -> Self {
        match field {
            MatchedField::Def => Self::DefIndex,
            MatchedField::ImplMethod => Self::ImplMethodIndex,
            MatchedField::PreWriteLocalOperation => Self::PreWriteLocalOperationIndex,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct CueEvidence {
    pub(in crate::prewrite::cues) artifact: &'static str,
    pub(in crate::prewrite) matched_field: CueMatchedField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) matched_field_source: Option<MatchedField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) algorithm_version: Option<&'static str>,
    pub(in crate::prewrite::cues) candidate_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) file_lookup_result: Option<FileLookupResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) distance: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite::cues) tokens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) policy_id: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) policy_version: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) operation_family: Option<ServiceOperationFamily>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite::cues) shared_domain_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) locality: Option<Locality>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite::cues) supporting_reasons: Vec<PolicySupportingReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) surface_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) container_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) container_kind: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct Cue {
    pub(in crate::prewrite) cue_tier: CueTier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) safe_meaning: Option<SafeMeaning>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite::cues) not_safe_for: Vec<NotSafeFor>,
    pub(in crate::prewrite) evidence_lane: EvidenceLane,
    pub(in crate::prewrite::cues) claim: CueClaim,
    pub(in crate::prewrite::cues) confidence: CueConfidence,
    pub(in crate::prewrite) evidence: Vec<CueEvidence>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct CueCandidate {
    pub(in crate::prewrite) identity: String,
    pub(in crate::prewrite::cues) owner_file: String,
    pub(in crate::prewrite::cues) name: String,
}

impl From<&CandidateRecord> for CueCandidate {
    fn from(candidate: &CandidateRecord) -> Self {
        Self {
            identity: candidate.identity.clone(),
            owner_file: candidate.owner_file.clone(),
            name: candidate.name.clone(),
        }
    }
}

impl From<&ServiceOperationPolicyEntry> for CueCandidate {
    fn from(candidate: &ServiceOperationPolicyEntry) -> Self {
        Self {
            identity: candidate.identity.clone(),
            owner_file: candidate.owner_file.clone(),
            name: candidate.name.clone(),
        }
    }
}

impl From<&LocalOperationPolicyEntry> for CueCandidate {
    fn from(candidate: &LocalOperationPolicyEntry) -> Self {
        Self {
            identity: candidate.identity.clone(),
            owner_file: candidate.owner_file.clone(),
            name: candidate.name.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct CueCard {
    pub(in crate::prewrite) candidate: CueCandidate,
    pub(in crate::prewrite) render_tier: CueTier,
    pub(in crate::prewrite) cues: Vec<Cue>,
}

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

pub(in crate::prewrite) struct CueProjection {
    pub(in crate::prewrite) cue_cards: Vec<CueCard>,
    pub(in crate::prewrite) suppressed_cues: Vec<SuppressedCue>,
}

pub(in crate::prewrite::cues) struct CueCardBuilder {
    pub(in crate::prewrite::cues) candidate: CueCandidate,
    pub(in crate::prewrite::cues) render_tier: CueTier,
    pub(in crate::prewrite::cues) cues: Vec<Cue>,
}
