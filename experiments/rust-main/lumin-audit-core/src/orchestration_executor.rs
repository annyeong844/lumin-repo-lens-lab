use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Instant;

use crate::orchestration_events::{
    LedgerCache, LedgerEvent, LedgerGeneratedArtifacts, LedgerScanRange, MemorySnapshot,
    ProducerLedgerEvent, ProducerMemory, SkippedLedgerEvent,
};
use crate::orchestration_plan::{
    build_orchestration_plan, AuditProfile, OrchestrationPlan, OrchestrationPlanOptions,
    ORCHESTRATION_PLAN_SCHEMA_VERSION,
};
use crate::source_commit::git_head_commit_or_unknown;

pub const EXECUTOR_REQUEST_SCHEMA_VERSION: &str = "lumin-audit-executor-request.v1";
pub const EXECUTOR_RESULT_SCHEMA_VERSION: &str = "lumin-audit-executor-result.v1";
pub const RUNTIME_EXECUTOR_REQUEST_SCHEMA_VERSION: &str = "lumin-audit-runtime-executor-request.v1";
pub const RUNTIME_EXECUTOR_RESULT_SCHEMA_VERSION: &str = "lumin-audit-runtime-executor-result.v1";

const INCREMENTAL_PRODUCER_STEPS: &[&str] = &[
    "measure-topology.mjs",
    "measure-staleness.mjs",
    "build-block-clone-index.mjs",
    "build-symbol-graph.mjs",
    "build-shape-index.mjs",
    "build-function-clone-index.mjs",
];

const RUST_ANALYZER_STEP: &str = "lumin-rust-analyzer";
const RUST_ANALYZER_ARTIFACT: &str = "rust-analyzer-health.latest.json";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorRequest {
    pub schema_version: String,
    pub plan: ExecutorPlanInput,
    pub root: PathBuf,
    pub output: PathBuf,
    pub scripts_dir: PathBuf,
    pub node_executable: String,
    #[serde(default)]
    pub verbose: bool,
    pub scan_range: LedgerScanRange,
    pub cache: LedgerCache,
    pub generated_artifacts: LedgerGeneratedArtifacts,
    #[serde(default)]
    pub rust_analyzer: RustAnalyzerExecutorRequest,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExecutorRequest {
    pub schema_version: String,
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default)]
    pub sarif: bool,
    #[serde(default)]
    pub pre_write: bool,
    #[serde(default)]
    pub post_write: bool,
    #[serde(default)]
    pub canon_draft: bool,
    #[serde(default)]
    pub check_canon: bool,
    pub root: PathBuf,
    pub output: PathBuf,
    pub scripts_dir: PathBuf,
    pub node_executable: String,
    #[serde(default)]
    pub verbose: bool,
    pub scan_range: LedgerScanRange,
    pub cache: LedgerCache,
    pub generated_artifacts: LedgerGeneratedArtifacts,
    #[serde(default)]
    pub rust_analyzer: RustAnalyzerExecutorRequest,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorPlanInput {
    pub schema_version: String,
    pub profile: String,
    #[serde(default)]
    pub emit_sarif: bool,
    pub base_pipeline: ExecutorBasePipelineInput,
    #[serde(default)]
    pub steps: Vec<ExecutorStepInput>,
    #[serde(default)]
    pub skipped: Vec<ExecutorPlannedSkipInput>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorBasePipelineInput {
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorStepInput {
    pub step: String,
    pub script: String,
    #[serde(default)]
    pub required: bool,
    pub producer_owner: String,
    pub execution_owner: String,
    #[serde(default)]
    pub skip_reason_when_unmet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorPlannedSkipInput {
    pub step: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalyzerExecutorRequest {
    #[serde(default)]
    pub requested: bool,
    #[serde(default)]
    pub rust_files: u64,
    #[serde(default)]
    pub source_commit: Option<String>,
    #[serde(default)]
    pub invocation: Option<RustAnalyzerInvocation>,
    #[serde(default)]
    pub forwarded_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalyzerInvocation {
    pub command: String,
    #[serde(default)]
    pub prefix_args: Vec<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalyzerArtifactInvocation {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorResult {
    pub schema_version: &'static str,
    pub events: Vec<LedgerEvent>,
    pub commands_run: Vec<CommandRun>,
    pub skipped: Vec<SkippedRun>,
    pub rust_analysis_run: RustAnalysisRunResult,
    pub exit_policy: ExecutorExitPolicy,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExecutorResult {
    pub schema_version: &'static str,
    pub plan: OrchestrationPlan,
    pub events: Vec<LedgerEvent>,
    pub commands_run: Vec<CommandRun>,
    pub skipped: Vec<SkippedRun>,
    pub rust_analysis_run: RustAnalysisRunResult,
    pub exit_policy: ExecutorExitPolicy,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRun {
    pub step: String,
    pub status: String,
    pub ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_invocation: Option<RustAnalyzerArtifactInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    pub memory: ExecutorMemoryObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedRun {
    pub step: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalysisRunResult {
    pub requested: bool,
    pub ran: bool,
    pub status: String,
    pub rust_files: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_invocation: Option<RustAnalyzerArtifactInvocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorExitPolicy {
    pub base_pipeline_failed_required: bool,
    pub recommended_exit_code: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorMemoryObservation {
    pub before: MemorySnapshot,
    pub after: MemorySnapshot,
    pub delta: MemorySnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreconditionOutcome {
    Met,
    Unmet,
}

struct ChildObservation {
    status: String,
    ms: u64,
    stderr_snippet: Option<String>,
    memory: ExecutorMemoryObservation,
}

struct RustAnalyzerObserved {
    commands_run: Vec<CommandRun>,
    skipped: Vec<SkippedRun>,
    events: Vec<LedgerEvent>,
    rust_analysis_run: RustAnalysisRunResult,
}

pub fn validate_executor_request(request: &ExecutorRequest) -> Result<()> {
    if request.schema_version != EXECUTOR_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-base-plan: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.plan.schema_version != ORCHESTRATION_PLAN_SCHEMA_VERSION {
        bail!(
            "execute-base-plan: unsupported plan.schemaVersion '{}'",
            request.plan.schema_version
        );
    }
    AuditProfile::parse(&request.plan.profile)?;
    validate_non_empty("nodeExecutable", &request.node_executable)?;
    validate_path("root", &request.root)?;
    validate_path("output", &request.output)?;
    validate_path("scriptsDir", &request.scripts_dir)?;
    validate_base_pipeline_status(&request.plan.base_pipeline.status)?;
    for step in &request.plan.steps {
        validate_step(step)?;
    }
    for skip in &request.plan.skipped {
        validate_non_empty("skipped.step", &skip.step)?;
        validate_non_empty("skipped.reason", &skip.reason)?;
    }
    Ok(())
}

pub fn execute_runtime_request(request: RuntimeExecutorRequest) -> Result<RuntimeExecutorResult> {
    validate_runtime_executor_request(&request)?;
    let profile = AuditProfile::parse(&request.profile)?;
    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        profile,
        sarif: request.sarif,
        pre_write: request.pre_write,
        post_write: request.post_write,
        canon_draft: request.canon_draft,
        check_canon: request.check_canon,
        rust_analyzer: request.rust_analyzer.requested,
    });
    let executor_request = ExecutorRequest {
        schema_version: EXECUTOR_REQUEST_SCHEMA_VERSION.to_string(),
        plan: executor_plan_input_from_plan(&plan),
        root: request.root,
        output: request.output,
        scripts_dir: request.scripts_dir,
        node_executable: request.node_executable,
        verbose: request.verbose,
        scan_range: request.scan_range,
        cache: request.cache,
        generated_artifacts: request.generated_artifacts,
        rust_analyzer: request.rust_analyzer,
    };
    let result = execute_base_plan(executor_request)?;
    Ok(RuntimeExecutorResult {
        schema_version: RUNTIME_EXECUTOR_RESULT_SCHEMA_VERSION,
        plan,
        events: result.events,
        commands_run: result.commands_run,
        skipped: result.skipped,
        rust_analysis_run: result.rust_analysis_run,
        exit_policy: result.exit_policy,
    })
}

pub fn execute_base_plan(request: ExecutorRequest) -> Result<ExecutorResult> {
    validate_executor_request(&request)?;
    let mut commands_run = Vec::new();
    let mut skipped = Vec::new();
    let mut events = Vec::new();
    let mut failed_required = false;
    let mut rust_analysis_run = not_requested_rust_analysis(&request);

    if request.plan.base_pipeline.status != "planned" {
        append_planned_skips(&request, &mut skipped, &mut events);
        return Ok(result_from_parts(
            commands_run,
            skipped,
            events,
            rust_analysis_run,
            false,
        ));
    }

    for step in request.plan.steps.clone() {
        if step.step == RUST_ANALYZER_STEP {
            let observed = execute_rust_analyzer_step(&request)?;
            rust_analysis_run = observed.rust_analysis_run;
            commands_run.extend(observed.commands_run);
            skipped.extend(observed.skipped);
            events.extend(observed.events);
            continue;
        }

        if precondition_outcome(&request, &step.step)? == PreconditionOutcome::Unmet {
            let reason = step
                .skip_reason_when_unmet
                .as_deref()
                .unwrap_or("precondition unmet");
            push_skip(&mut skipped, &mut events, &step.step, reason);
            continue;
        }

        let argv = argv_for_js_step(&request, &step.script);
        clear_producer_phase_timing(&request.output, &step.step);
        let observed = run_child(&request.node_executable, &argv, request.verbose)?;
        let status = command_status(&observed, step.required);
        if status == "failed-required" {
            failed_required = true;
        }
        push_command(
            &mut commands_run,
            &mut events,
            CommandRun {
                step: step.step.clone(),
                status,
                ms: observed.ms,
                artifact: None,
                rust_files: None,
                analyzer_invocation: None,
                stderr: observed.stderr_snippet,
                memory: observed.memory,
            },
        );
        if failed_required {
            break;
        }
    }

    append_planned_skips(&request, &mut skipped, &mut events);

    Ok(result_from_parts(
        commands_run,
        skipped,
        events,
        rust_analysis_run,
        failed_required,
    ))
}

fn validate_runtime_executor_request(request: &RuntimeExecutorRequest) -> Result<()> {
    if request.schema_version != RUNTIME_EXECUTOR_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-base-runtime: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    AuditProfile::parse(&request.profile)?;
    validate_non_empty_for(
        "execute-base-runtime",
        "nodeExecutable",
        &request.node_executable,
    )?;
    validate_path_for("execute-base-runtime", "root", &request.root)?;
    validate_path_for("execute-base-runtime", "output", &request.output)?;
    validate_path_for("execute-base-runtime", "scriptsDir", &request.scripts_dir)?;
    Ok(())
}

fn executor_plan_input_from_plan(plan: &OrchestrationPlan) -> ExecutorPlanInput {
    ExecutorPlanInput {
        schema_version: plan.schema_version.to_string(),
        profile: plan.profile.as_str().to_string(),
        emit_sarif: plan.emit_sarif,
        base_pipeline: ExecutorBasePipelineInput {
            status: plan.base_pipeline.status.as_str().to_string(),
        },
        steps: plan
            .steps
            .iter()
            .map(|step| ExecutorStepInput {
                step: step.step.to_string(),
                script: step.script.to_string(),
                required: step.required,
                producer_owner: step.producer_owner.as_str().to_string(),
                execution_owner: step.execution_owner.to_string(),
                skip_reason_when_unmet: step.skip_reason_when_unmet.map(str::to_string),
            })
            .collect(),
        skipped: plan
            .skipped
            .iter()
            .map(|skip| ExecutorPlannedSkipInput {
                step: skip.step.to_string(),
                reason: skip.reason.to_string(),
            })
            .collect(),
    }
}

fn default_profile() -> String {
    "quick".to_string()
}

fn argv_for_js_step(request: &ExecutorRequest, script: &str) -> Vec<String> {
    let mut argv = vec![
        request
            .scripts_dir
            .join(script)
            .to_string_lossy()
            .to_string(),
        "--root".to_string(),
        request.root.to_string_lossy().to_string(),
        "--output".to_string(),
        request.output.to_string_lossy().to_string(),
    ];

    if !request.scan_range.include_tests {
        argv.push("--production".to_string());
    }
    for exclude in &request.scan_range.excludes {
        argv.push("--exclude".to_string());
        argv.push(exclude.clone());
    }
    if is_incremental_step(script) {
        if request.cache.no_incremental {
            argv.push("--no-incremental".to_string());
        }
        if !request.cache.cache_root.trim().is_empty() {
            argv.push("--cache-root".to_string());
            argv.push(request.cache.cache_root.clone());
        }
    }
    if script == "build-symbol-graph.mjs" {
        argv.push("--generated-artifacts".to_string());
        argv.push(request.generated_artifacts.mode.clone());
    }
    argv
}

fn validate_non_empty(field: &str, value: &str) -> Result<()> {
    validate_non_empty_for("execute-base-plan", field, value)
}

fn validate_non_empty_for(label: &str, field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label}: {field} must be a non-empty string");
    }
    Ok(())
}

fn validate_path(field: &str, value: &Path) -> Result<()> {
    validate_path_for("execute-base-plan", field, value)
}

fn validate_path_for(label: &str, field: &str, value: &Path) -> Result<()> {
    if value.as_os_str().is_empty() {
        bail!("{label}: {field} must be provided");
    }
    Ok(())
}

fn validate_base_pipeline_status(status: &str) -> Result<()> {
    if !matches!(status, "planned" | "skipped") {
        bail!("execute-base-plan: unsupported basePipeline.status '{status}'");
    }
    Ok(())
}

fn validate_step(step: &ExecutorStepInput) -> Result<()> {
    validate_non_empty("step.step", &step.step)?;
    validate_non_empty("step.script", &step.script)?;
    if !matches!(step.producer_owner.as_str(), "js-mjs" | "rust") {
        bail!(
            "execute-base-plan: unsupported producerOwner '{}' for step '{}'",
            step.producer_owner,
            step.step
        );
    }
    if !matches!(
        step.execution_owner.as_str(),
        "audit-repo.mjs" | "lumin-audit-core"
    ) {
        bail!(
            "execute-base-plan: unsupported executionOwner '{}' for step '{}'",
            step.execution_owner,
            step.step
        );
    }
    Ok(())
}

fn is_incremental_step(step: &str) -> bool {
    INCREMENTAL_PRODUCER_STEPS.contains(&step)
}

fn precondition_outcome(request: &ExecutorRequest, step: &str) -> Result<PreconditionOutcome> {
    let output = &request.output;
    let root = &request.root;
    let exists_in_output = |name: &str| output.join(name).is_file();

    let met = match step {
        "build-resolver-diagnostics.mjs" | "build-entry-surface.mjs" => {
            exists_in_output("symbols.json")
        }
        "build-module-reachability.mjs" => {
            exists_in_output("symbols.json") && exists_in_output("entry-surface.json")
        }
        "export-action-safety.mjs" | "rank-fixes.mjs" => exists_in_output("dead-classify.json"),
        "merge-runtime-evidence.mjs" => {
            root.join("coverage").join("coverage-final.json").is_file()
                || root
                    .join(".nyc_output")
                    .join("coverage-final.json")
                    .is_file()
        }
        "measure-staleness.mjs" => is_git_work_tree(root)?,
        _ => true,
    };

    Ok(if met {
        PreconditionOutcome::Met
    } else {
        PreconditionOutcome::Unmet
    })
}

fn is_git_work_tree(root: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(root)
        .output();
    match output {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
        }
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn run_child(command: &str, args: &[String], verbose: bool) -> Result<ChildObservation> {
    let before = memory_snapshot();
    let started = Instant::now();
    let output = if verbose {
        Command::new(command)
            .args(args)
            .status()
            .map(|status| Output {
                status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
    } else {
        Command::new(command).args(args).output()
    }?;
    let ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let after = memory_snapshot();
    let status = if output.status.success() {
        "ok"
    } else {
        "failed"
    }
    .to_string();
    let stderr_text = String::from_utf8_lossy(&output.stderr).to_string();
    let stderr_snippet =
        (!stderr_text.trim().is_empty()).then(|| stderr_text.chars().take(500).collect::<String>());
    Ok(ChildObservation {
        status,
        ms,
        stderr_snippet,
        memory: ExecutorMemoryObservation {
            delta: memory_delta(&before, &after),
            before,
            after,
        },
    })
}

fn failed_child_observation_from_spawn_error(error: &dyn std::fmt::Display) -> ChildObservation {
    let before = memory_snapshot();
    let after = memory_snapshot();
    let stderr = format!("failed to start child process: {error}");
    ChildObservation {
        status: "failed".to_string(),
        ms: 0,
        stderr_snippet: Some(stderr.chars().take(500).collect()),
        memory: ExecutorMemoryObservation {
            delta: memory_delta(&before, &after),
            before,
            after,
        },
    }
}

fn command_status(observed: &ChildObservation, required: bool) -> String {
    if observed.status == "ok" {
        "ok"
    } else if required {
        "failed-required"
    } else {
        "failed-optional"
    }
    .to_string()
}

fn clear_producer_phase_timing(output: &Path, producer: &str) {
    let phase_path = output
        .join(".producer-phases")
        .join(format!("{}.json", safe_producer_file_name(producer)));
    remove_file_if_present(&phase_path);
}

fn remove_file_if_present(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => {}
    }
}

fn safe_producer_file_name(producer: &str) -> String {
    let base = producer
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_string();
    base.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn push_command(
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

fn push_skip(
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

fn append_planned_skips(
    request: &ExecutorRequest,
    skipped: &mut Vec<SkippedRun>,
    events: &mut Vec<LedgerEvent>,
) {
    for skip in &request.plan.skipped {
        push_skip(skipped, events, &skip.step, &skip.reason);
    }
}

fn not_requested_rust_analysis(request: &ExecutorRequest) -> RustAnalysisRunResult {
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

fn observed_rust_file_count(request: &ExecutorRequest) -> Result<u64> {
    let path = request.output.join("triage.json");
    if let Ok(text) = fs::read_to_string(path) {
        let triage = serde_json::from_str::<Value>(&text)?;
        let count = rust_file_count_from_triage(&triage);
        if count > 0 {
            return Ok(count);
        }
    }
    Ok(request.rust_analyzer.rust_files)
}

fn rust_file_count_from_triage(triage: &Value) -> u64 {
    for path in [
        &["byLanguage"][..],
        &["languages"][..],
        &["summary", "byLanguage"][..],
    ] {
        let Some(count) = value_at(triage, path).and_then(|value| value.get("rs")) else {
            continue;
        };
        if let Some(n) = count.as_u64() {
            if n > 0 {
                return n;
            }
        }
        if let Some(n) = count.get("files").and_then(Value::as_u64) {
            if n > 0 {
                return n;
            }
        }
    }
    triage
        .get("shape")
        .and_then(|shape| {
            shape
                .get("rustFiles")
                .or_else(|| shape.get("rsFiles"))
                .and_then(Value::as_u64)
        })
        .unwrap_or(0)
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, segment| current.get(*segment))
}

fn artifact_invocation(invocation: &RustAnalyzerInvocation) -> RustAnalyzerArtifactInvocation {
    RustAnalyzerArtifactInvocation {
        source: invocation.source.clone(),
        manifest_path: invocation.manifest_path.clone(),
    }
}

fn execute_rust_analyzer_step(request: &ExecutorRequest) -> Result<RustAnalyzerObserved> {
    if !request.rust_analyzer.requested {
        return Ok(RustAnalyzerObserved {
            commands_run: Vec::new(),
            skipped: Vec::new(),
            events: Vec::new(),
            rust_analysis_run: not_requested_rust_analysis(request),
        });
    }
    let rust_files = observed_rust_file_count(request)?;
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

    remove_file_if_present(&artifact_path);
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

fn result_from_parts(
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

fn memory_snapshot() -> MemorySnapshot {
    MemorySnapshot {
        rss_bytes: current_process_rss_bytes(),
        heap_total_bytes: 0,
        heap_used_bytes: 0,
        external_bytes: 0,
        array_buffers_bytes: 0,
    }
}

#[cfg(windows)]
fn current_process_rss_bytes() -> i64 {
    windows_working_set_bytes().unwrap_or(0)
}

#[cfg(windows)]
fn windows_working_set_bytes() -> Option<i64> {
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
    }

    #[link(name = "psapi")]
    extern "system" {
        fn GetProcessMemoryInfo(
            process: *mut std::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            size: u32,
        ) -> i32;
    }

    let mut counters = ProcessMemoryCounters {
        cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        page_fault_count: 0,
        peak_working_set_size: 0,
        working_set_size: 0,
        quota_peak_paged_pool_usage: 0,
        quota_paged_pool_usage: 0,
        quota_peak_non_paged_pool_usage: 0,
        quota_non_paged_pool_usage: 0,
        pagefile_usage: 0,
        peak_pagefile_usage: 0,
    };
    let ok = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
    (ok != 0).then_some(counters.working_set_size as i64)
}

#[cfg(unix)]
fn current_process_rss_bytes() -> i64 {
    linux_proc_status_rss_bytes().unwrap_or(0)
}

#[cfg(unix)]
fn linux_proc_status_rss_bytes() -> Option<i64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        let Some(rest) = line.strip_prefix("VmRSS:") else {
            continue;
        };
        let kb = rest
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<i64>().ok())?;
        return Some(kb.saturating_mul(1024));
    }
    None
}

#[cfg(not(any(windows, unix)))]
fn current_process_rss_bytes() -> i64 {
    0
}

fn memory_delta(before: &MemorySnapshot, after: &MemorySnapshot) -> MemorySnapshot {
    MemorySnapshot {
        rss_bytes: after.rss_bytes - before.rss_bytes,
        heap_total_bytes: after.heap_total_bytes - before.heap_total_bytes,
        heap_used_bytes: after.heap_used_bytes - before.heap_used_bytes,
        external_bytes: after.external_bytes - before.external_bytes,
        array_buffers_bytes: after.array_buffers_bytes - before.array_buffers_bytes,
    }
}
