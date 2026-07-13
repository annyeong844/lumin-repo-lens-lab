use super::child_process::{
    failed_child_observation_from_spawn_error, remove_file_if_present, run_child,
};
use super::observations::push_command;
use super::protocol::{
    CommandRun, ExecutorRequest, RustAnalysisRunResult, RustAnalyzerArtifactInvocation,
    RustAnalyzerInvocation, SkippedRun,
};
use super::{RUST_ANALYZER_ARTIFACT, RUST_ANALYZER_STEP};
use crate::orchestration_events::{LedgerEvent, SkippedLedgerEvent};
use crate::source_commit::git_head_commit_or_unknown;
use crate::source_inventory::ValidatedSourceInventory;
use anyhow::Result;

pub(super) struct RustAnalyzerObserved {
    pub(super) commands_run: Vec<CommandRun>,
    pub(super) skipped: Vec<SkippedRun>,
    pub(super) events: Vec<LedgerEvent>,
    pub(super) rust_analysis_run: RustAnalysisRunResult,
}

pub(super) fn not_requested_rust_analysis(request: &ExecutorRequest) -> RustAnalysisRunResult {
    RustAnalysisRunResult {
        requested: request.rust_analyzer.requested,
        ran: false,
        status: "not-requested".to_string(),
        rust_files: request.rust_analyzer.rust_files,
        reason: None,
        artifact: None,
        path: None,
        source_commit: None,
        producer: None,
        analyzer_invocation: None,
    }
}

fn observed_rust_file_count(source_inventory: &ValidatedSourceInventory) -> u64 {
    source_inventory.file_count_for_extensions(&[".rs"]) as u64
}

fn artifact_invocation(invocation: &RustAnalyzerInvocation) -> RustAnalyzerArtifactInvocation {
    RustAnalyzerArtifactInvocation {
        source: invocation.source.clone(),
        manifest_path: invocation.manifest_path.clone(),
    }
}

pub(super) fn execute_rust_analyzer_step(
    request: &ExecutorRequest,
    source_inventory: &ValidatedSourceInventory,
) -> Result<RustAnalyzerObserved> {
    if !request.rust_analyzer.requested {
        return Ok(RustAnalyzerObserved {
            commands_run: Vec::new(),
            skipped: Vec::new(),
            events: Vec::new(),
            rust_analysis_run: not_requested_rust_analysis(request),
        });
    }
    let rust_files = observed_rust_file_count(source_inventory);
    if rust_files == 0 {
        let reason = "no Rust files counted by triage".to_string();
        return Ok(rust_analyzer_skip(request, "skipped", rust_files, reason));
    }
    let Some(invocation) = request.rust_analyzer.invocation.clone() else {
        let reason =
            "rust analyzer requested but no Rust analyzer invocation was supplied".to_string();
        return Ok(rust_analyzer_skip(
            request,
            "unavailable",
            rust_files,
            reason,
        ));
    };

    let artifact_path = request.output.join(RUST_ANALYZER_ARTIFACT);
    let source_commit = rust_analyzer_source_commit(request);
    let mut args = invocation.prefix_args.clone();
    args.extend([
        "--root".to_string(),
        request.root.to_string_lossy().to_string(),
        "--source-commit".to_string(),
        source_commit.clone(),
        "--output".to_string(),
        artifact_path.to_string_lossy().to_string(),
        "--source-health-profile".to_string(),
        "compact".to_string(),
        "--semantic-mode".to_string(),
        "metadata-only".to_string(),
    ]);
    args.extend(request.rust_analyzer.forwarded_args.clone());

    remove_file_if_present(&artifact_path)?;
    let observed = run_child(&invocation.command, &args, request.verbose)
        .unwrap_or_else(|error| failed_child_observation_from_spawn_error(&error));
    if observed.status == "ok" {
        let command = CommandRun {
            step: RUST_ANALYZER_STEP.to_string(),
            status: "ok".to_string(),
            ms: observed.ms,
            artifact: Some(RUST_ANALYZER_ARTIFACT.to_string()),
            rust_files: Some(rust_files),
            analyzer_invocation: Some(artifact_invocation(&invocation)),
            stderr: None,
            memory: observed.memory,
        };
        let mut events = Vec::new();
        let mut commands_run = Vec::new();
        push_command(&mut commands_run, &mut events, command);
        return Ok(RustAnalyzerObserved {
            commands_run,
            skipped: Vec::new(),
            events,
            rust_analysis_run: RustAnalysisRunResult {
                requested: true,
                ran: true,
                status: "complete".to_string(),
                rust_files,
                reason: None,
                artifact: Some(RUST_ANALYZER_ARTIFACT.to_string()),
                path: Some(artifact_path.to_string_lossy().to_string()),
                source_commit: Some(source_commit),
                producer: Some(RUST_ANALYZER_STEP.to_string()),
                analyzer_invocation: Some(artifact_invocation(&invocation)),
            },
        });
    }

    let command = CommandRun {
        step: RUST_ANALYZER_STEP.to_string(),
        status: "failed-optional".to_string(),
        ms: observed.ms,
        artifact: None,
        rust_files: Some(rust_files),
        analyzer_invocation: None,
        stderr: observed.stderr_snippet,
        memory: observed.memory,
    };
    let mut events = Vec::new();
    let mut commands_run = Vec::new();
    push_command(&mut commands_run, &mut events, command);
    Ok(RustAnalyzerObserved {
        commands_run,
        skipped: Vec::new(),
        events,
        rust_analysis_run: RustAnalysisRunResult {
            requested: true,
            ran: false,
            status: "failed-optional".to_string(),
            rust_files,
            reason: Some("lumin-rust-analyzer did not complete".to_string()),
            artifact: None,
            path: None,
            source_commit: Some(source_commit),
            producer: Some(RUST_ANALYZER_STEP.to_string()),
            analyzer_invocation: None,
        },
    })
}

fn rust_analyzer_source_commit(request: &ExecutorRequest) -> String {
    request
        .rust_analyzer
        .source_commit
        .as_deref()
        .map(str::trim)
        .filter(|commit| !commit.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| git_head_commit_or_unknown(&request.root))
}

fn rust_analyzer_skip(
    request: &ExecutorRequest,
    status: &str,
    rust_files: u64,
    reason: String,
) -> RustAnalyzerObserved {
    let skipped = vec![SkippedRun {
        step: RUST_ANALYZER_STEP.to_string(),
        reason: reason.clone(),
    }];
    let events = vec![LedgerEvent::Skipped(Box::new(SkippedLedgerEvent {
        name: RUST_ANALYZER_STEP.to_string(),
        reason: reason.clone(),
    }))];
    RustAnalyzerObserved {
        commands_run: Vec::new(),
        skipped,
        events,
        rust_analysis_run: RustAnalysisRunResult {
            requested: true,
            ran: false,
            status: status.to_string(),
            rust_files,
            reason: Some(reason),
            artifact: None,
            path: None,
            source_commit: Some(rust_analyzer_source_commit(request)),
            producer: None,
            analyzer_invocation: None,
        },
    }
}
