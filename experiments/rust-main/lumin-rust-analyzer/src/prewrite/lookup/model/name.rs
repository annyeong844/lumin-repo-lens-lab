use serde::Serialize;

use super::hints::TaintSummary;
use super::hints::{NearNameHint, SemanticHint, SuppressedNearNameHint, SuppressedSemanticHint};
use super::identity::{CandidateRecord, LookupResult};
use super::local::LocalOperationSiblingPolicy;
use super::service::ServiceOperationSiblingPolicy;

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
    pub(in crate::prewrite) local_operation_sibling_policy: LocalOperationSiblingPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::prewrite) tainted_by: Option<TaintSummary>,
    pub(in crate::prewrite) citations: Vec<String>,
}
