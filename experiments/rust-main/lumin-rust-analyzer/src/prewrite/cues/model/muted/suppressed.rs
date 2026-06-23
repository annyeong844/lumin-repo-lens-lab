use lumin_rust_source_health::protocol::PathClassification;
use serde::Serialize;

use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::{Locality, PolicySupportingReason, ServiceOperationFamily};

use super::super::candidate::CueCandidate;
use super::super::vocabulary::{CueTier, EvidenceLane};
use super::reason::MutedReason;

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
