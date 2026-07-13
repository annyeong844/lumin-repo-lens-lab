use super::protocol::{
    CommandRun, ExecutorExitPolicy, ExecutorRequest, ExecutorResult, RustAnalysisRunResult,
    SkippedRun,
};
use super::EXECUTOR_RESULT_SCHEMA_VERSION;
use crate::orchestration_events::{
    LedgerEvent, ProducerLedgerEvent, ProducerMemory, SkippedLedgerEvent,
};

pub(super) fn push_command(
    commands_run: &mut Vec<CommandRun>,
    events: &mut Vec<LedgerEvent>,
    run: CommandRun,
) {
    events.push(producer_event_from_command(&run));
    commands_run.push(run);
}

fn producer_event_from_command(run: &CommandRun) -> LedgerEvent {
    LedgerEvent::Producer(Box::new(ProducerLedgerEvent {
        name: run.step.clone(),
        status: run.status.clone(),
        wall_ms: Some(run.ms),
        phases: None,
        counters: None,
        memory: Some(ProducerMemory {
            before: run.memory.before.clone(),
            after: run.memory.after.clone(),
            delta: run.memory.delta.clone(),
        }),
        stderr_snippet: run.stderr.clone(),
    }))
}

pub(super) fn push_skip(
    skipped: &mut Vec<SkippedRun>,
    events: &mut Vec<LedgerEvent>,
    step: &str,
    reason: &str,
) {
    skipped.push(SkippedRun {
        step: step.to_string(),
        reason: reason.to_string(),
    });
    events.push(LedgerEvent::Skipped(Box::new(SkippedLedgerEvent {
        name: step.to_string(),
        reason: reason.to_string(),
    })));
}

pub(super) fn append_planned_skips(
    request: &ExecutorRequest,
    skipped: &mut Vec<SkippedRun>,
    events: &mut Vec<LedgerEvent>,
) {
    for skip in &request.plan.skipped {
        push_skip(skipped, events, &skip.step, &skip.reason);
    }
}

pub(super) fn result_from_parts(
    commands_run: Vec<CommandRun>,
    skipped: Vec<SkippedRun>,
    events: Vec<LedgerEvent>,
    rust_analysis_run: RustAnalysisRunResult,
    failed_required: bool,
) -> ExecutorResult {
    ExecutorResult {
        schema_version: EXECUTOR_RESULT_SCHEMA_VERSION,
        events,
        commands_run,
        skipped,
        rust_analysis_run,
        exit_policy: ExecutorExitPolicy {
            base_pipeline_failed_required: failed_required,
            recommended_exit_code: if failed_required { 1 } else { 0 },
        },
    }
}
