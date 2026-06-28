use serde::Serialize;

use super::{
    ClaimKind, ClassificationRule, CodeKind, CodeNamespace, CodePresence, ConfidenceTier,
    CoverageEffect, Disposition, PrimarySpan, PrimarySpanClass, RustcDiagnosticLevel,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEvidence {
    pub level: Option<RustcDiagnosticLevel>,
    pub raw_code: DiagnosticCode,
    pub normalized: NormalizedDiagnostic,
    pub classification: ClassificationEvidence,
    pub message: Option<String>,
    pub primary_spans: Vec<PrimarySpan>,
    pub rendered_first_line: Option<String>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DiagnosticCode {
    Detail(DiagnosticCodeDetail),
    Text(String),
    Null,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize)]
pub struct DiagnosticCodeDetail {
    pub code: Option<String>,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedDiagnostic {
    pub code_presence: CodePresence,
    pub code_value: Option<String>,
    pub code_namespace: CodeNamespace,
    pub code_kind: CodeKind,
    pub primary_span: PrimarySpanClass,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationEvidence {
    pub disposition: Disposition,
    pub confidence: Option<ConfidenceTier>,
    pub claim_kind: Option<ClaimKind>,
    pub coverage_effect: Option<CoverageEffect>,
    pub rule: ClassificationRule,
}
