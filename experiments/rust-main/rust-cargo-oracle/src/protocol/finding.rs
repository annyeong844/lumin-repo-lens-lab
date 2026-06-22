use serde::Serialize;

use super::{
    ActionBlockerReason, ClaimKind, ClassificationRule, ConfidenceTier, CoverageId,
    FindingSourceKind, FindingSourceVersion, OracleId, PrimarySpan, PrimarySpanClass,
    RustcSuggestionApplicability, SafeActionKind,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub oracle_id: OracleId,
    pub source: FindingSource,
    pub confidence: FindingConfidence,
    pub confidence_tier: ConfidenceTier,
    pub claim_kind: ClaimKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_code: Option<String>,
    pub message: Option<String>,
    pub span: Option<PrimarySpan>,
    pub primary_spans: Vec<PrimarySpan>,
    pub coverage_ref: CoverageId,
    pub analysis_input_set_hash: String,
    pub rule: ClassificationRule,
    pub action_blockers: Vec<ActionBlockerReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_action: Option<SafeAction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeAction {
    pub kind: SafeActionKind,
    pub proof_complete: bool,
    pub action_blockers: Vec<ActionBlockerReason>,
    pub stronger_action_blockers: Vec<ActionBlockerReason>,
    pub edits: Vec<SafeActionEdit>,
    pub proof: SafeActionProof,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeActionEdit {
    pub file_name: String,
    pub line_start: i64,
    pub line_end: i64,
    pub column_start: i64,
    pub column_end: i64,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeActionProof {
    pub oracle_id: OracleId,
    pub diagnostic_code: String,
    pub applicability: RustcSuggestionApplicability,
    pub primary_span_class: PrimarySpanClass,
    pub no_macro_expansion: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingSource {
    pub oracle_id: OracleId,
    pub source_kind: FindingSourceKind,
    pub version: FindingSourceVersion,
    pub command: String,
    pub command_args: Vec<String>,
    pub registry_content_hash: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingConfidence {
    pub tier: ConfidenceTier,
    pub authority_ids: Vec<&'static str>,
    pub rule_ids: Vec<&'static str>,
    pub claim_kind: ClaimKind,
}
