use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticHealthArtifact {
    #[serde(rename = "schemaVersion")]
    pub schema_version: &'static str,
    #[serde(rename = "policyVersion")]
    pub policy_version: &'static str,
    #[serde(rename = "oracleRegistryVersion")]
    pub oracle_registry_version: &'static str,
    pub meta: ArtifactMeta,
    pub findings: Vec<Finding>,
    pub diagnostics: Vec<DiagnosticEvidence>,
    pub coverage: Vec<CoverageEntry>,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactMeta {
    pub producer: &'static str,
    pub mode: &'static str,
    pub generated: String,
    #[serde(rename = "oracleRegistryVersion")]
    pub oracle_registry_version: &'static str,
    #[serde(rename = "evidencePolicyVersion")]
    pub evidence_policy_version: &'static str,
    #[serde(rename = "diagnosticPolicyVersion")]
    pub diagnostic_policy_version: &'static str,
    #[serde(rename = "registryContentHash")]
    pub registry_content_hash: String,
    #[serde(rename = "analysisInputSetHash")]
    pub analysis_input_set_hash: String,
    #[serde(rename = "analysisInputSetComplete")]
    pub analysis_input_set_complete: bool,
    #[serde(rename = "missingInfluenceKinds")]
    pub missing_influence_kinds: Vec<&'static str>,
    pub toolchain: ToolchainMeta,
    #[serde(rename = "cacheReusePolicy")]
    pub cache_reuse_policy: &'static str,
    #[serde(rename = "cacheReuse")]
    pub cache_reuse: CacheReuse,
    pub input: InputMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainMeta {
    #[serde(rename = "cargoVersion")]
    pub cargo_version: Option<String>,
    #[serde(rename = "rustcVersionVerbose")]
    pub rustc_version_verbose: Option<String>,
    #[serde(rename = "rustcBin")]
    pub rustc_bin: String,
    #[serde(rename = "rustcSource")]
    pub rustc_source: &'static str,
    pub host: Option<String>,
    pub profile: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMeta {
    pub root: String,
    #[serde(rename = "packageName")]
    pub package_name: Option<String>,
    pub features: Option<String>,
    #[serde(rename = "cargoBin")]
    pub cargo_bin: String,
    #[serde(rename = "cargoArgs")]
    pub cargo_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheReuse {
    pub policy: &'static str,
    pub reason: &'static str,
    #[serde(rename = "blockingTargets")]
    pub blocking_targets: Vec<BlockingTarget>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockingTarget {
    #[serde(rename = "packageId")]
    pub package_id: String,
    #[serde(rename = "packageName")]
    pub package_name: String,
    #[serde(rename = "targetName")]
    pub target_name: String,
    #[serde(rename = "targetKinds")]
    pub target_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    #[serde(rename = "oracleId")]
    pub oracle_id: &'static str,
    pub source: FindingSource,
    pub confidence: FindingConfidence,
    #[serde(rename = "confidenceTier")]
    pub confidence_tier: ConfidenceTier,
    #[serde(rename = "claimKind")]
    pub claim_kind: ClaimKind,
    pub message: Option<String>,
    pub span: Value,
    #[serde(rename = "primarySpans")]
    pub primary_spans: Vec<Value>,
    #[serde(rename = "coverageRef")]
    pub coverage_ref: &'static str,
    #[serde(rename = "analysisInputSetHash")]
    pub analysis_input_set_hash: String,
    pub rule: ClassificationRule,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingSource {
    #[serde(rename = "oracleId")]
    pub oracle_id: &'static str,
    #[serde(rename = "sourceKind")]
    pub source_kind: &'static str,
    pub version: &'static str,
    pub command: String,
    #[serde(rename = "commandArgs")]
    pub command_args: Vec<String>,
    #[serde(rename = "registryContentHash")]
    pub registry_content_hash: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingConfidence {
    pub tier: ConfidenceTier,
    #[serde(rename = "authorityIds")]
    pub authority_ids: Vec<&'static str>,
    #[serde(rename = "ruleIds")]
    pub rule_ids: Vec<&'static str>,
    #[serde(rename = "claimKind")]
    pub claim_kind: ClaimKind,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEvidence {
    pub level: Option<String>,
    #[serde(rename = "rawCode")]
    pub raw_code: Value,
    pub normalized: NormalizedDiagnostic,
    pub classification: ClassificationEvidence,
    pub message: Option<String>,
    #[serde(rename = "primarySpans")]
    pub primary_spans: Vec<Value>,
    #[serde(rename = "renderedFirstLine")]
    pub rendered_first_line: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedDiagnostic {
    #[serde(rename = "codePresence")]
    pub code_presence: CodePresence,
    #[serde(rename = "codeValue")]
    pub code_value: Option<String>,
    #[serde(rename = "codeNamespace")]
    pub code_namespace: CodeNamespace,
    #[serde(rename = "codeKind")]
    pub code_kind: CodeKind,
    #[serde(rename = "primarySpan")]
    pub primary_span: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationEvidence {
    pub disposition: Disposition,
    pub confidence: Option<ConfidenceTier>,
    #[serde(rename = "claimKind")]
    pub claim_kind: Option<ClaimKind>,
    #[serde(rename = "coverageEffect")]
    pub coverage_effect: Option<CoverageEffect>,
    pub rule: ClassificationRule,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageEntry {
    pub id: &'static str,
    #[serde(rename = "oracleId")]
    pub oracle_id: &'static str,
    #[serde(rename = "coverageKind")]
    pub coverage_kind: CoverageKind,
    pub status: CoverageStatus,
    #[serde(rename = "streamParseStatus", skip_serializing_if = "Option::is_none")]
    pub stream_parse_status: Option<String>,
    #[serde(
        rename = "invalidJsonLineCount",
        skip_serializing_if = "Option::is_none"
    )]
    pub invalid_json_line_count: Option<usize>,
    pub scope: Value,
    pub command: String,
    #[serde(rename = "commandArgs")]
    pub command_args: Vec<String>,
    #[serde(rename = "exitCode")]
    pub exit_code: Option<i32>,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u128,
    #[serde(rename = "analysisInputSetHash")]
    pub analysis_input_set_hash: String,
    #[serde(rename = "registryContentHash")]
    pub registry_content_hash: String,
    #[serde(rename = "diagnosticPolicyVersion")]
    pub diagnostic_policy_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(rename = "cleanKind", skip_serializing_if = "Option::is_none")]
    pub clean_kind: Option<&'static str>,
    #[serde(rename = "cleanScope", skip_serializing_if = "Option::is_none")]
    pub clean_scope: Option<&'static str>,
    #[serde(rename = "absenceOfClaimKinds", skip_serializing_if = "Vec::is_empty")]
    pub absence_of_claim_kinds: Vec<ClaimKind>,
    #[serde(
        rename = "allowsConcurrentClaimKinds",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub allows_concurrent_claim_kinds: Vec<ClaimKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clean: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub findings: usize,
    pub diagnostics: usize,
    pub coverage: usize,
    #[serde(rename = "verifiedFindings")]
    pub verified_findings: usize,
    #[serde(rename = "ruleBackedFindings")]
    pub rule_backed_findings: usize,
    #[serde(rename = "candidateFindings")]
    pub candidate_findings: usize,
    #[serde(rename = "coverageUnavailableDiagnostics")]
    pub coverage_unavailable_diagnostics: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Disposition {
    Finding,
    NonFinding,
    CoverageUnavailable,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfidenceTier {
    Verified,
    RuleBacked,
    Candidate,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum ClaimKind {
    #[serde(rename = "verified.rust.rustc-error-diagnostic")]
    RustcErrorDiagnostic,
    #[serde(rename = "verified.rust.rustc-codeless-error-diagnostic")]
    RustcCodelessErrorDiagnostic,
    #[serde(rename = "rule-backed.rust.rustc-lint-diagnostic")]
    RustcLintDiagnostic,
    #[serde(rename = "candidate.rust.unclassified-cargo-diagnostic")]
    UnclassifiedCargoDiagnostic,
}

impl ClaimKind {
    pub const EMITTED_BY_CLASSIFIER: [Self; 4] = [
        Self::RustcErrorDiagnostic,
        Self::RustcCodelessErrorDiagnostic,
        Self::RustcLintDiagnostic,
        Self::UnclassifiedCargoDiagnostic,
    ];

    pub const ABSENCE_CLEAN_CLAIM_KINDS: [Self; 2] = [
        Self::RustcErrorDiagnostic,
        Self::RustcCodelessErrorDiagnostic,
    ];

    pub const ABSENCE_CLEAN_CONCURRENT_CLAIM_KINDS: [Self; 2] =
        [Self::RustcLintDiagnostic, Self::UnclassifiedCargoDiagnostic];

    pub fn tier(self) -> ConfidenceTier {
        match self {
            Self::RustcErrorDiagnostic | Self::RustcCodelessErrorDiagnostic => {
                ConfidenceTier::Verified
            }
            Self::RustcLintDiagnostic => ConfidenceTier::RuleBacked,
            Self::UnclassifiedCargoDiagnostic => ConfidenceTier::Candidate,
        }
    }

    pub fn authority_ids(self) -> Vec<&'static str> {
        match self {
            Self::RustcErrorDiagnostic => vec!["rust.rustc.error-diagnostic"],
            Self::RustcCodelessErrorDiagnostic => vec!["rust.rustc.codeless-error-diagnostic"],
            _ => Vec::new(),
        }
    }

    pub fn rule_ids(self) -> Vec<&'static str> {
        match self {
            Self::RustcLintDiagnostic => vec!["rust.rustc.lint-diagnostic"],
            _ => Vec::new(),
        }
    }

    pub fn is_verified_rustc_error(self) -> bool {
        matches!(
            self,
            Self::RustcErrorDiagnostic | Self::RustcCodelessErrorDiagnostic
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum ClassificationRule {
    #[serde(rename = "note-help-failure-note-are-not-findings")]
    NoteHelpFailureNoteAreNotFindings,
    #[serde(rename = "non-user-primary-error-makes-absence-clean-unavailable")]
    NonUserPrimaryErrorMakesAbsenceCleanUnavailable,
    #[serde(rename = "non-user-primary-diagnostics-are-not-user-facing-findings")]
    NonUserPrimaryDiagnosticsAreNotUserFacingFindings,
    #[serde(rename = "non-ecode-code-name-treated-as-rule-backed-before-level")]
    NonEcodeCodeNameTreatedAsRuleBackedBeforeLevel,
    #[serde(rename = "ecode-error-user-code-primary")]
    EcodeErrorUserCodePrimary,
    #[serde(rename = "codeless-error-user-code-primary")]
    CodelessErrorUserCodePrimary,
    #[serde(rename = "fallback-real-warning-or-error-never-verified")]
    FallbackRealWarningOrErrorNeverVerified,
}

impl ClassificationRule {
    pub const EMITTED_BY_CLASSIFIER: [Self; 7] = [
        Self::NoteHelpFailureNoteAreNotFindings,
        Self::NonUserPrimaryErrorMakesAbsenceCleanUnavailable,
        Self::NonUserPrimaryDiagnosticsAreNotUserFacingFindings,
        Self::NonEcodeCodeNameTreatedAsRuleBackedBeforeLevel,
        Self::EcodeErrorUserCodePrimary,
        Self::CodelessErrorUserCodePrimary,
        Self::FallbackRealWarningOrErrorNeverVerified,
    ];
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum CoverageEffect {
    #[serde(rename = "absence-clean-unavailable")]
    AbsenceCleanUnavailable,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum CodePresence {
    #[serde(rename = "present-null")]
    PresentNull,
    #[serde(rename = "omitted")]
    Omitted,
    #[serde(rename = "present-value")]
    PresentValue,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum CodeNamespace {
    #[serde(rename = "rustc-codeless")]
    RustcCodeless,
    #[serde(rename = "rustc-error")]
    RustcError,
    #[serde(rename = "rustc-non-ecode")]
    RustcNonEcode,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum CodeKind {
    #[serde(rename = "null-error-code")]
    NullErrorCode,
    #[serde(rename = "rustc-error-code")]
    RustcErrorCode,
    #[serde(rename = "non-ecode-name")]
    NonEcodeName,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageKind {
    CargoEventStream,
    AbsenceClean,
}

impl CoverageKind {
    pub const EMITTED_BY_ORACLE: [Self; 2] = [Self::CargoEventStream, Self::AbsenceClean];
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageStatus {
    Ran,
    Unavailable,
}
