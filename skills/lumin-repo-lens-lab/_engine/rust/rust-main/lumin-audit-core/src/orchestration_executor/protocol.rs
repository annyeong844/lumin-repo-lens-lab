use crate::orchestration_events::{
    LedgerCache, LedgerEvent, LedgerGeneratedArtifacts, LedgerScanRange, MemorySnapshot,
};
use crate::orchestration_plan::OrchestrationPlan;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorRequest {
    pub schema_version: String,
    pub run_id: String,
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
    pub run_id: String,
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default)]
    pub profile_explicit: bool,
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

fn default_profile() -> String {
    "quick".to_string()
}
