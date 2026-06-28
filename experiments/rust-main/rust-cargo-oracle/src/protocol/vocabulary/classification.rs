use serde::Serialize;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
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
#[serde(rename_all = "kebab-case")]
pub enum ClassificationRule {
    NoteHelpFailureNoteAreNotFindings,
    NonUserPrimaryErrorMakesAbsenceCleanUnavailable,
    NonUserPrimaryDiagnosticsAreNotUserFacingFindings,
    NonEcodeCodeNameTreatedAsRuleBackedBeforeLevel,
    EcodeErrorUserCodePrimary,
    CodelessErrorUserCodePrimary,
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
