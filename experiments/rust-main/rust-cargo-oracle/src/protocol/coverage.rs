use serde::Serialize;

use super::{
    ClaimKind, CleanKind, CleanScope, CoverageId, CoverageKind, CoverageStatus,
    CoverageUnavailableReasons, OracleId, OracleScope, StreamParseStatus,
};

pub const EVENT_STREAM_COVERAGE_ID: CoverageId = CoverageId::CargoCheckEventStream;
pub const ABSENCE_CLEAN_COVERAGE_ID: CoverageId = CoverageId::AbsenceClean;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageEntry {
    pub id: CoverageId,
    pub oracle_id: OracleId,
    pub coverage_kind: CoverageKind,
    pub status: CoverageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_parse_status: Option<StreamParseStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_json_line_count: Option<usize>,
    pub scope: OracleScope,
    pub command: String,
    pub command_args: Vec<String>,
    pub exit_code: Option<i32>,
    pub elapsed_ms: u128,
    pub analysis_input_set_hash: String,
    pub registry_content_hash: String,
    pub diagnostic_policy_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<CoverageUnavailableReasons>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clean_kind: Option<CleanKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clean_scope: Option<CleanScope>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub absence_of_claim_kinds: Vec<ClaimKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allows_concurrent_claim_kinds: Vec<ClaimKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clean: Option<bool>,
}
