use crate::protocol::{
    CoverageEntry, CoverageKind, CoverageStatus, CoverageUnavailableReason,
    CoverageUnavailableReasons, OracleId, EVENT_STREAM_COVERAGE_ID,
};
use crate::{CargoCheckMode, DIAGNOSTIC_POLICY_VERSION};

use super::{targeted_not_run_reason, CoverageInput};

pub(super) fn build_event_stream_coverage(
    input: &CoverageInput<'_>,
    command: String,
    command_args: Vec<String>,
) -> CoverageEntry {
    let stream_complete =
        input.stream_parse_status.is_complete() && input.invalid_json_line_count == 0;

    CoverageEntry {
        id: EVENT_STREAM_COVERAGE_ID,
        oracle_id: OracleId::RustCargoCheck,
        coverage_kind: CoverageKind::CargoEventStream,
        status: if stream_complete {
            CoverageStatus::Ran
        } else {
            CoverageStatus::Unavailable
        },
        stream_parse_status: Some(input.stream_parse_status),
        invalid_json_line_count: Some(input.invalid_json_line_count),
        scope: input.scope.clone(),
        command,
        command_args,
        exit_code: input.check_output.status,
        elapsed_ms: input.check_output.elapsed_ms,
        analysis_input_set_hash: input.input_hash.to_string(),
        registry_content_hash: input.registry_hash.to_string(),
        diagnostic_policy_version: DIAGNOSTIC_POLICY_VERSION,
        reason: (!stream_complete).then(|| stream_unavailable_reason(input)),
        clean_kind: None,
        clean_scope: None,
        absence_of_claim_kinds: Vec::new(),
        allows_concurrent_claim_kinds: Vec::new(),
        clean: None,
    }
}

fn stream_unavailable_reason(input: &CoverageInput<'_>) -> CoverageUnavailableReasons {
    if input.cargo_check_mode == CargoCheckMode::MetadataOnly {
        CoverageUnavailableReasons::one(
            CoverageUnavailableReason::CargoCheckOracleNotRunInMetadataOnlyMode,
        )
    } else if input.cargo_check_mode == CargoCheckMode::TargetedCargoCheck
        && input.check_output.status.is_none()
    {
        CoverageUnavailableReasons::one(targeted_not_run_reason(input.check_output))
    } else {
        CoverageUnavailableReasons::one(
            CoverageUnavailableReason::CargoJsonStreamUnavailableOrIncomplete,
        )
    }
}
