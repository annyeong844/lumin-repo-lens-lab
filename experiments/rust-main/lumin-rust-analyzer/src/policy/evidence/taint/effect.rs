use serde::{Serialize, Serializer};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(in crate::policy) enum TaintEffect {
    FileAstParseErrorsMayBeIncomplete,
    FileSemanticNoSyntaxProjection,
    FileAstOpaqueUntilOracleCalibration,
    FindingFileParseErrorsMayBeIncomplete,
    FindingNoSyntaxProjection,
    FindingOverlapsReviewOpaqueSurface,
    SafeFixBlockedByActionBlocker,
    CandidateDiagnosticNotVerified,
    FindingCargoEventCoverageUnavailable,
    ScopeAbsenceCleanUnavailable,
}

impl Serialize for TaintEffect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::FileAstParseErrorsMayBeIncomplete => {
                "the Rust AST parser reported errors for this file; syntax facts may be incomplete"
            }
            Self::FileSemanticNoSyntaxProjection => {
                "the file has semantic evidence but no syntax phase projection in this artifact"
            }
            Self::FileAstOpaqueUntilOracleCalibration => {
                "macro expansion or cfg gating remains AST-opaque until Cargo/rustc evidence is calibrated for this file"
            }
            Self::FindingFileParseErrorsMayBeIncomplete => {
                "the file declaring this finding failed to parse; syntax evidence may be incomplete"
            }
            Self::FindingNoSyntaxProjection => {
                "this semantic finding has no syntax phase file projection in the artifact"
            }
            Self::FindingOverlapsReviewOpaqueSurface => {
                "a review-visible macro or cfg opaque surface overlaps this finding span"
            }
            Self::SafeFixBlockedByActionBlocker => {
                "rustc suggested edit evidence exists, but the selected action is blocked from SAFE_FIX"
            }
            Self::CandidateDiagnosticNotVerified => {
                "the cargo diagnostic is not yet verified or rule-backed"
            }
            Self::FindingCargoEventCoverageUnavailable => {
                "rustc diagnostic event coverage is unavailable for this finding"
            }
            Self::ScopeAbsenceCleanUnavailable => {
                "the run cannot prove absence of rustc errors for the declared Cargo-check scope"
            }
        })
    }
}
