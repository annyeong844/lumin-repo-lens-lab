use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::AstOpaqueSurfaceKind;
use serde::Serialize;

use super::identity::{CandidateRecord, Locality};
use super::suppression::SuppressionReason;

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
