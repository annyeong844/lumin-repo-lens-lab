use lumin_rust_cargo_oracle::protocol::{ClaimKind, CleanKind, ConfidenceTier, OracleId};
use serde::Serialize;

use crate::policy::{CoverageRunStatus, FileParseStatus};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SupportEvidenceKind {
    RustAstFile,
    CargoRustcDiagnostics,
    CargoCheckEventStream,
    CargoCheckAbsenceClean,
    CargoRustcDiagnostic,
    RustcMachineApplicableSafeAction,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(in crate::policy) enum SupportEvidence {
    RustAstFile(RustAstFileSupport),
    CargoRustcDiagnostics(CargoRustcDiagnosticsSupport),
    CargoCheckEventStream(CargoCheckEventStreamSupport),
    CargoCheckAbsenceClean(CargoCheckAbsenceCleanSupport),
    CargoRustcDiagnostic(CargoRustcDiagnosticSupport),
    RustcMachineApplicableSafeAction(RustcMachineApplicableSafeActionSupport),
}

impl SupportEvidence {
    pub(in crate::policy) fn rust_ast_file(
        parse_status: FileParseStatus,
        parse_errors: usize,
    ) -> Self {
        Self::RustAstFile(RustAstFileSupport::new(parse_status, parse_errors))
    }

    pub(in crate::policy) fn cargo_rustc_diagnostics(findings: usize, diagnostics: usize) -> Self {
        Self::CargoRustcDiagnostics(CargoRustcDiagnosticsSupport::new(findings, diagnostics))
    }

    pub(in crate::policy) fn cargo_check_event_stream(status: CoverageRunStatus) -> Self {
        Self::CargoCheckEventStream(CargoCheckEventStreamSupport::new(status))
    }

    pub(in crate::policy) fn cargo_check_absence_clean(
        status: CoverageRunStatus,
        clean: Option<bool>,
        clean_kind: Option<CleanKind>,
    ) -> Self {
        Self::CargoCheckAbsenceClean(CargoCheckAbsenceCleanSupport::new(
            status, clean, clean_kind,
        ))
    }

    pub(in crate::policy) fn cargo_rustc_diagnostic(
        oracle_id: OracleId,
        claim_kind: ClaimKind,
        confidence_tier: ConfidenceTier,
    ) -> Self {
        Self::CargoRustcDiagnostic(CargoRustcDiagnosticSupport::new(
            oracle_id,
            claim_kind,
            confidence_tier,
        ))
    }

    pub(in crate::policy) fn rustc_machine_applicable_safe_action() -> Self {
        Self::RustcMachineApplicableSafeAction(RustcMachineApplicableSafeActionSupport::new())
    }
}

pub(in crate::policy) fn push_ast_file_support(
    supported_by: &mut Vec<SupportEvidence>,
    syntax_present: bool,
    parse_status: FileParseStatus,
    parse_errors: usize,
) {
    if syntax_present {
        supported_by.push(SupportEvidence::rust_ast_file(parse_status, parse_errors));
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct RustAstFileSupport {
    kind: SupportEvidenceKind,
    parse_status: FileParseStatus,
    parse_errors: usize,
}

impl RustAstFileSupport {
    fn new(parse_status: FileParseStatus, parse_errors: usize) -> Self {
        Self {
            kind: SupportEvidenceKind::RustAstFile,
            parse_status,
            parse_errors,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct CargoCheckEventStreamSupport {
    kind: SupportEvidenceKind,
    status: CoverageRunStatus,
}

impl CargoCheckEventStreamSupport {
    fn new(status: CoverageRunStatus) -> Self {
        Self {
            kind: SupportEvidenceKind::CargoCheckEventStream,
            status,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct CargoCheckAbsenceCleanSupport {
    kind: SupportEvidenceKind,
    status: CoverageRunStatus,
    clean: Option<bool>,
    clean_kind: Option<CleanKind>,
}

impl CargoCheckAbsenceCleanSupport {
    fn new(status: CoverageRunStatus, clean: Option<bool>, clean_kind: Option<CleanKind>) -> Self {
        Self {
            kind: SupportEvidenceKind::CargoCheckAbsenceClean,
            status,
            clean,
            clean_kind,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct CargoRustcDiagnosticsSupport {
    kind: SupportEvidenceKind,
    findings: usize,
    diagnostics: usize,
}

impl CargoRustcDiagnosticsSupport {
    fn new(findings: usize, diagnostics: usize) -> Self {
        Self {
            kind: SupportEvidenceKind::CargoRustcDiagnostics,
            findings,
            diagnostics,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct CargoRustcDiagnosticSupport {
    kind: SupportEvidenceKind,
    oracle_id: OracleId,
    claim_kind: ClaimKind,
    confidence_tier: ConfidenceTier,
}

impl CargoRustcDiagnosticSupport {
    fn new(oracle_id: OracleId, claim_kind: ClaimKind, confidence_tier: ConfidenceTier) -> Self {
        Self {
            kind: SupportEvidenceKind::CargoRustcDiagnostic,
            oracle_id,
            claim_kind,
            confidence_tier,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct RustcMachineApplicableSafeActionSupport {
    kind: SupportEvidenceKind,
    proof_complete: bool,
}

impl RustcMachineApplicableSafeActionSupport {
    fn new() -> Self {
        Self {
            kind: SupportEvidenceKind::RustcMachineApplicableSafeAction,
            proof_complete: true,
        }
    }
}
