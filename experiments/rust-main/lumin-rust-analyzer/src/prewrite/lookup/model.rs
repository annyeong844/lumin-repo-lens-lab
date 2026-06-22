use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::{
    AstDefinitionKind, AstOpaqueSurfaceKind, AstVisibility, PathClassification,
};
use serde::Serialize;

use crate::prewrite::index::{Candidate, MatchedField};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum LookupResult {
    #[serde(rename = "NOT_OBSERVED")]
    NotObserved,
    #[serde(rename = "EXISTS")]
    Exists,
    #[serde(rename = "EXISTS_MULTIPLE")]
    ExistsMultiple,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct CandidateRecord {
    pub(in crate::prewrite) identity: String,
    pub(in crate::prewrite) owner_file: String,
    pub(in crate::prewrite) name: String,
    pub(in crate::prewrite) matched_field: MatchedField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) definition_kind: Option<AstDefinitionKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) impl_target: Option<String>,
    #[serde(rename = "trait", skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) trait_path: Option<String>,
    pub(in crate::prewrite) visibility: AstVisibility,
    pub(in crate::prewrite) line: usize,
    pub(in crate::prewrite) column: usize,
    pub(in crate::prewrite) policy_excluded: bool,
    pub(in crate::prewrite) path_classifications: Vec<PathClassification>,
}

impl CandidateRecord {
    pub(in crate::prewrite) fn from_candidate(candidate: Candidate<'_>) -> Self {
        let path_classifications = candidate.classifications();
        let policy_excluded = candidate.path.suppressed
            || path_classifications.iter().any(|classification| {
                matches!(
                    classification,
                    PathClassification::Test | PathClassification::Generated
                )
            });
        Self {
            identity: candidate.identity(),
            owner_file: candidate.file.to_string(),
            name: candidate.name.to_string(),
            matched_field: candidate.lane.matched_field(),
            definition_kind: candidate.definition_kind,
            impl_target: candidate.owner.map(|owner| owner.target.to_string()),
            trait_path: candidate
                .owner
                .and_then(|owner| owner.trait_path)
                .map(str::to_string),
            visibility: candidate.visibility,
            line: candidate.location.line,
            column: candidate.location.column,
            policy_excluded,
            path_classifications,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct Locality {
    pub(in crate::prewrite) same_dir: bool,
    pub(in crate::prewrite) same_file: bool,
}

impl Locality {
    pub(in crate::prewrite) fn rank(self) -> usize {
        if self.same_file {
            2
        } else if self.same_dir {
            1
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct NearNameHint {
    #[serde(flatten)]
    pub(in crate::prewrite) candidate: CandidateRecord,
    pub(in crate::prewrite) distance: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite) matched_tokens: Vec<String>,
    pub(in crate::prewrite) locality: Locality,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct SemanticHint {
    #[serde(flatten)]
    pub(in crate::prewrite) candidate: CandidateRecord,
    pub(in crate::prewrite) matched_tokens: Vec<String>,
    pub(in crate::prewrite) matched_name_tokens: Vec<String>,
    pub(in crate::prewrite) matched_support_tokens: Vec<String>,
    pub(in crate::prewrite) score: usize,
    pub(in crate::prewrite) locality: Locality,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum SuppressionReason {
    DomainTokenOverlap,
    NearLengthDeltaExceeded,
    NearPrefixMismatch,
    NearDistanceExceeded,
    SingleNonWeakTokenOnly,
    InsufficientNonWeakSupport,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum ServiceOperationFamily {
    ReadQuery,
    MutationCreate,
    MutationDelete,
    MutationSend,
    MutationUpdate,
    MutationSave,
}

impl ServiceOperationFamily {
    pub(in crate::prewrite) fn as_str(self) -> &'static str {
        match self {
            Self::ReadQuery => "read-query",
            Self::MutationCreate => "mutation-create",
            Self::MutationDelete => "mutation-delete",
            Self::MutationSend => "mutation-send",
            Self::MutationUpdate => "mutation-update",
            Self::MutationSave => "mutation-save",
        }
    }
}

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

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct ServiceSignatureSupport {
    pub(in crate::prewrite) status: &'static str,
    pub(in crate::prewrite) reason: &'static str,
}

impl ServiceSignatureSupport {
    pub(in crate::prewrite) fn unavailable() -> Self {
        Self {
            status: "unavailable",
            reason: "no-signature-facts",
        }
    }
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
    pub(in crate::prewrite) supporting_reasons: Vec<SuppressionReason>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct SuppressedNearNameHint {
    #[serde(flatten)]
    pub(in crate::prewrite) candidate: CandidateRecord,
    pub(in crate::prewrite) reason: SuppressionReason,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite) matched_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) distance: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) length_delta: Option<usize>,
    pub(in crate::prewrite) locality: Locality,
    pub(in crate::prewrite) candidate_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct SuppressedSemanticHint {
    #[serde(flatten)]
    pub(in crate::prewrite) candidate: CandidateRecord,
    pub(in crate::prewrite) reason: SuppressionReason,
    pub(in crate::prewrite) matched_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite) matched_name_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::prewrite) matched_support_tokens: Vec<String>,
    pub(in crate::prewrite) score: usize,
    pub(in crate::prewrite) locality: Locality,
    pub(in crate::prewrite) candidate_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct TaintSummary {
    pub(in crate::prewrite) parse_error_files: usize,
    pub(in crate::prewrite) review_opaque_surfaces: usize,
    pub(in crate::prewrite) review_opaque_surfaces_by_kind: BTreeMap<AstOpaqueSurfaceKind, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct NameLookup {
    pub(in crate::prewrite) intent_name: String,
    pub(in crate::prewrite) result: LookupResult,
    pub(in crate::prewrite) identities: Vec<CandidateRecord>,
    pub(in crate::prewrite) intent_tokens: Vec<String>,
    pub(in crate::prewrite) near_names: Vec<NearNameHint>,
    pub(in crate::prewrite) semantic_hints: Vec<SemanticHint>,
    pub(in crate::prewrite) suppressed_near_names: Vec<SuppressedNearNameHint>,
    pub(in crate::prewrite) suppressed_near_name_count: usize,
    pub(in crate::prewrite) suppressed_semantic_hints: Vec<SuppressedSemanticHint>,
    pub(in crate::prewrite) suppressed_semantic_hint_count: usize,
    pub(in crate::prewrite) service_operation_sibling_policy: ServiceOperationSiblingPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) tainted_by: Option<TaintSummary>,
    pub(in crate::prewrite) citations: Vec<String>,
}
