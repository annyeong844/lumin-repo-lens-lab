use crate::cargo_json::CargoBuildFinished;
use crate::classify::Diagnostic;
use crate::command::{CommandOutput, CommandSkipReason};
mod absence;
mod stream;

use crate::protocol::{CoverageEntry, CoverageUnavailableReason, OracleScope, StreamParseStatus};
use crate::CargoCheckMode;

use super::command_line::command_args;
use absence::build_absence_clean_coverage;
use stream::build_event_stream_coverage;

pub(super) struct CoverageInput<'a> {
    pub(super) build_finished: Option<CargoBuildFinished>,
    pub(super) stream_parse_status: StreamParseStatus,
    pub(super) invalid_json_line_count: usize,
    pub(super) diagnostics: &'a [Diagnostic],
    pub(super) scope: &'a OracleScope,
    pub(super) check_output: &'a CommandOutput,
    pub(super) cargo_bin: &'a str,
    pub(super) cargo_args: &'a [String],
    pub(super) cargo_check_mode: CargoCheckMode,
    pub(super) metadata_unavailable_reason: Option<&'a str>,
    pub(super) input_hash: &'a str,
    pub(super) registry_hash: &'a str,
}

pub(super) fn build_coverage(input: CoverageInput<'_>) -> Vec<CoverageEntry> {
    let command_args_value = command_args(input.cargo_bin, input.cargo_args);
    let command = command_args_value.join(" ");
    let stream = build_event_stream_coverage(&input, command.clone(), command_args_value.clone());
    let absence = build_absence_clean_coverage(&input, command, command_args_value);

    vec![stream, absence]
}

fn targeted_not_run_reason(check_output: &CommandOutput) -> CoverageUnavailableReason {
    match check_output.skip_reason {
        Some(CommandSkipReason::TargetedCargoCheckSelectedNoPackages) | None => {
            CoverageUnavailableReason::TargetedCargoCheckSelectedNoPackages
        }
    }
}
