use lumin_rust_cargo_oracle::protocol::ActionBlockerReason;
use lumin_rust_source_health::protocol::AstOpaqueSurface;
use serde::{Serialize, Serializer};

use crate::policy::{CoverageRunStatus, FileParseStatus};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum TaintEvidenceKind {
    RustFileParseError,
    RustAstFileMissing,
    RustAstReviewOpaqueSurface,
    CargoEventStreamNotRun,
    CargoAbsenceCleanUnavailable,
    RustAstReviewOpaqueSurfaceNearFinding,
    SemanticActionBlocker,
    SemanticCandidateFinding,
}

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

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(in crate::policy) enum TaintEvidence<'a> {
    RustFileParseError(RustFileParseErrorTaint),
    RustAstFileMissing(RustAstFileMissingTaint),
    RustAstReviewOpaqueSurface(RustAstReviewOpaqueSurfaceTaint),
    CargoEventStreamNotRun(CargoEventStreamNotRunTaint),
    CargoAbsenceCleanUnavailable(CargoAbsenceCleanUnavailableTaint),
    RustAstReviewOpaqueSurfaceNearFinding(RustAstReviewOpaqueSurfaceNearFindingTaint<'a>),
    SemanticActionBlocker(SemanticActionBlockerTaint),
    SemanticCandidateFinding(SemanticCandidateFindingTaint),
}

impl<'a> TaintEvidence<'a> {
    pub(in crate::policy) fn lowers_finding_confidence_to_low(&self) -> bool {
        matches!(
            self,
            Self::RustFileParseError(_)
                | Self::RustAstFileMissing(_)
                | Self::CargoEventStreamNotRun(_)
                | Self::SemanticCandidateFinding(_)
        )
    }

    pub(in crate::policy) fn rust_file_parse_error(
        parse_errors: usize,
        effect: TaintEffect,
    ) -> Self {
        Self::RustFileParseError(RustFileParseErrorTaint {
            kind: TaintEvidenceKind::RustFileParseError,
            parse_errors,
            effect,
        })
    }

    pub(in crate::policy) fn rust_ast_file_missing(effect: TaintEffect) -> Self {
        Self::RustAstFileMissing(RustAstFileMissingTaint {
            kind: TaintEvidenceKind::RustAstFileMissing,
            effect,
        })
    }

    pub(in crate::policy) fn rust_ast_review_opaque_surface(
        count: usize,
        effect: TaintEffect,
    ) -> Self {
        Self::RustAstReviewOpaqueSurface(RustAstReviewOpaqueSurfaceTaint {
            kind: TaintEvidenceKind::RustAstReviewOpaqueSurface,
            count,
            effect,
        })
    }

    pub(in crate::policy) fn cargo_event_stream_not_run(
        status: CoverageRunStatus,
        effect: TaintEffect,
    ) -> Self {
        Self::CargoEventStreamNotRun(CargoEventStreamNotRunTaint {
            kind: TaintEvidenceKind::CargoEventStreamNotRun,
            status,
            effect,
        })
    }

    pub(in crate::policy) fn cargo_absence_clean_unavailable(
        status: CoverageRunStatus,
        effect: TaintEffect,
    ) -> Self {
        Self::CargoAbsenceCleanUnavailable(CargoAbsenceCleanUnavailableTaint {
            kind: TaintEvidenceKind::CargoAbsenceCleanUnavailable,
            status,
            effect,
        })
    }

    pub(in crate::policy) fn rust_ast_review_opaque_surface_near_finding(
        total: usize,
        sample: Vec<&'a AstOpaqueSurface>,
        effect: TaintEffect,
    ) -> Self {
        Self::RustAstReviewOpaqueSurfaceNearFinding(RustAstReviewOpaqueSurfaceNearFindingTaint {
            kind: TaintEvidenceKind::RustAstReviewOpaqueSurfaceNearFinding,
            total,
            sample,
            effect,
        })
    }

    pub(in crate::policy) fn semantic_action_blocker(
        reasons: Vec<ActionBlockerReason>,
        effect: TaintEffect,
    ) -> Self {
        Self::SemanticActionBlocker(SemanticActionBlockerTaint {
            kind: TaintEvidenceKind::SemanticActionBlocker,
            reasons,
            effect,
        })
    }

    pub(in crate::policy) fn semantic_candidate_finding(effect: TaintEffect) -> Self {
        Self::SemanticCandidateFinding(SemanticCandidateFindingTaint {
            kind: TaintEvidenceKind::SemanticCandidateFinding,
            effect,
        })
    }
}

pub(in crate::policy) fn push_parse_taint<'a>(
    tainted_by: &mut Vec<TaintEvidence<'a>>,
    parse_status: FileParseStatus,
    parse_errors: usize,
    parse_error_effect: TaintEffect,
    missing_effect: TaintEffect,
) {
    match parse_status {
        FileParseStatus::Error => tainted_by.push(TaintEvidence::rust_file_parse_error(
            parse_errors,
            parse_error_effect,
        )),
        FileParseStatus::Missing => {
            tainted_by.push(TaintEvidence::rust_ast_file_missing(missing_effect));
        }
        FileParseStatus::Ok => {}
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct RustFileParseErrorTaint {
    kind: TaintEvidenceKind,
    parse_errors: usize,
    effect: TaintEffect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct RustAstFileMissingTaint {
    kind: TaintEvidenceKind,
    effect: TaintEffect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct RustAstReviewOpaqueSurfaceTaint {
    kind: TaintEvidenceKind,
    count: usize,
    effect: TaintEffect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct CargoEventStreamNotRunTaint {
    kind: TaintEvidenceKind,
    status: CoverageRunStatus,
    effect: TaintEffect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct CargoAbsenceCleanUnavailableTaint {
    kind: TaintEvidenceKind,
    status: CoverageRunStatus,
    effect: TaintEffect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct RustAstReviewOpaqueSurfaceNearFindingTaint<'a> {
    kind: TaintEvidenceKind,
    total: usize,
    sample: Vec<&'a AstOpaqueSurface>,
    effect: TaintEffect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct SemanticActionBlockerTaint {
    kind: TaintEvidenceKind,
    reasons: Vec<ActionBlockerReason>,
    effect: TaintEffect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct SemanticCandidateFindingTaint {
    kind: TaintEvidenceKind,
    effect: TaintEffect,
}
