use serde::Serialize;

use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::{
    DependencyLookupResult, FileLookupResult, Locality, PolicySupportingReason,
    ServiceOperationFamily, SignatureVisibility,
};

use super::vocabulary::{
    CueClaim, CueConfidence, CueMatchedField, CueTier, EvidenceLane, NotSafeFor, SafeMeaning,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct CueEvidence {
    pub(in crate::prewrite::cues) artifact: &'static str,
    pub(in crate::prewrite) matched_field: CueMatchedField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) matched_field_source: Option<MatchedField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) algorithm_version: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) visibility: Option<SignatureVisibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) local_name: Option<String>,
    pub(in crate::prewrite::cues) candidate_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) file_lookup_result: Option<FileLookupResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) dependency_lookup_result: Option<DependencyLookupResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) observed_import_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite::cues) consumer_threshold: Option<usize>,
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
