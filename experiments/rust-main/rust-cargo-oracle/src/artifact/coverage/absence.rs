use crate::classify::Diagnostic;
use crate::protocol::{
    ClaimKind, CleanKind, CleanScope, ConfidenceTier, CoverageEffect, CoverageEntry, CoverageKind,
    CoverageStatus, CoverageUnavailableReason, CoverageUnavailableReasons, Disposition, OracleId,
    StreamParseStatus, ABSENCE_CLEAN_COVERAGE_ID,
};
use crate::{CargoCheckMode, DIAGNOSTIC_POLICY_VERSION};

use super::{targeted_not_run_reason, CoverageInput};

pub(super) fn build_absence_clean_coverage(
    input: &CoverageInput<'_>,
    command: String,
    command_args: Vec<String>,
) -> CoverageEntry {
    let absence_unavailable = absence_unavailable_reasons(input);
    let absence_ran = absence_unavailable.is_empty();
    let verified_errors = verified_rustc_error_count(input.diagnostics);

    CoverageEntry {
        id: ABSENCE_CLEAN_COVERAGE_ID,
        oracle_id: OracleId::RustCargoCheck,
        coverage_kind: CoverageKind::AbsenceClean,
        status: if absence_ran {
            CoverageStatus::Ran
        } else {
            CoverageStatus::Unavailable
        },
        stream_parse_status: None,
        invalid_json_line_count: None,
        scope: input.scope.clone(),
        command,
        command_args,
        exit_code: input.check_output.status,
        elapsed_ms: input.check_output.elapsed_ms,
        analysis_input_set_hash: input.input_hash.to_string(),
        registry_content_hash: input.registry_hash.to_string(),
        diagnostic_policy_version: DIAGNOSTIC_POLICY_VERSION,
        reason: CoverageUnavailableReasons::from_reasons(absence_unavailable),
        clean_kind: absence_ran.then_some(CleanKind::VerifiedRustcErrorAbsence),
        clean_scope: absence_ran.then_some(CleanScope::DeclaredCargoCheckRustcErrorDiagnostics),
        absence_of_claim_kinds: if absence_ran {
            ClaimKind::ABSENCE_CLEAN_CLAIM_KINDS.to_vec()
        } else {
            Vec::new()
        },
        allows_concurrent_claim_kinds: if absence_ran {
            ClaimKind::ABSENCE_CLEAN_CONCURRENT_CLAIM_KINDS.to_vec()
        } else {
            Vec::new()
        },
        clean: absence_ran.then_some(verified_errors == 0),
    }
}

fn verified_rustc_error_count(diagnostics: &[Diagnostic]) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(diagnostic.classification.disposition, Disposition::Finding)
                && matches!(
                    diagnostic.classification.confidence,
                    Some(ConfidenceTier::Verified)
                )
                && diagnostic
                    .classification
                    .claim_kind
                    .is_some_and(ClaimKind::is_verified_rustc_error)
        })
        .count()
}

fn absence_unavailable_reasons(input: &CoverageInput<'_>) -> Vec<CoverageUnavailableReason> {
    let mut reasons = Vec::<CoverageUnavailableReason>::new();
    match input.stream_parse_status {
        StreamParseStatus::Complete => {}
        StreamParseStatus::NotRun if input.cargo_check_mode == CargoCheckMode::MetadataOnly => {
            reasons.push(CoverageUnavailableReason::CargoCheckOracleNotRunInMetadataOnlyMode);
        }
        StreamParseStatus::NotRun
            if input.cargo_check_mode == CargoCheckMode::TargetedCargoCheck
                && input.check_output.status.is_none() =>
        {
            reasons.push(targeted_not_run_reason(input.check_output));
        }
        StreamParseStatus::NotRun => {
            reasons.push(CoverageUnavailableReason::CargoCheckOracleNotRun)
        }
        StreamParseStatus::NoJsonEvents => {
            reasons.push(CoverageUnavailableReason::CargoJsonStreamContainedNoEvents);
        }
        StreamParseStatus::Timeout | StreamParseStatus::InvalidJson => {
            reasons.push(CoverageUnavailableReason::CargoJsonStreamDidNotParseCompletely);
        }
    }
    if input.invalid_json_line_count != 0 {
        reasons.push(CoverageUnavailableReason::CargoJsonStreamContainedInvalidJsonLines);
    }
    if input.build_finished.is_none() {
        reasons.push(CoverageUnavailableReason::MissingBuildFinishedEvent);
    }
    if let Some(build_finished) = input.build_finished {
        match build_finished.success() {
            Some(true) => {}
            Some(false) => reasons.push(CoverageUnavailableReason::BuildFinishedSuccessWasFalse),
            None => reasons.push(CoverageUnavailableReason::BuildFinishedSuccessWasNotTrue),
        }
    }
    if input.diagnostics.iter().any(|diagnostic| {
        diagnostic.classification.coverage_effect == Some(CoverageEffect::AbsenceCleanUnavailable)
    }) {
        reasons.push(CoverageUnavailableReason::NonUserCodePrimaryErrorDiagnosticEncountered);
    }
    if let Some(reason) = input.metadata_unavailable_reason {
        reasons.push(CoverageUnavailableReason::CargoMetadataUnavailable(
            reason.to_string(),
        ));
    }
    reasons
}
