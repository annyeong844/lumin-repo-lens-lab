mod child_process;
mod execution;
mod memory;
mod observations;
mod protocol;
mod rust_analyzer;
mod validation;

pub use execution::{execute_base_plan, execute_runtime_request};
pub use protocol::{
    CommandRun, ExecutorBasePipelineInput, ExecutorExitPolicy, ExecutorMemoryObservation,
    ExecutorPlanInput, ExecutorPlannedSkipInput, ExecutorRequest, ExecutorResult,
    ExecutorStepInput, RuntimeExecutorRequest, RuntimeExecutorResult, RustAnalysisRunResult,
    RustAnalyzerArtifactInvocation, RustAnalyzerExecutorRequest, RustAnalyzerInvocation,
    SkippedRun,
};
pub use validation::validate_executor_request;

pub const EXECUTOR_REQUEST_SCHEMA_VERSION: &str = "lumin-audit-executor-request.v2";
pub const EXECUTOR_RESULT_SCHEMA_VERSION: &str = "lumin-audit-executor-result.v1";
pub const RUNTIME_EXECUTOR_REQUEST_SCHEMA_VERSION: &str = "lumin-audit-runtime-executor-request.v2";
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
const TRIAGE_STEP: &str = "triage-repo.mjs";
const RUST_ONLY_SKIP_REASON: &str =
    "current-run source inventory contains Rust files and no supported non-Rust source files";
