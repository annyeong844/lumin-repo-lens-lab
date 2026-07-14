use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use lumin_audit_core::artifact_read_metrics::ArtifactReadObservation;
use lumin_audit_core::lifecycle::ManifestLifecycleUpdateInput;
use lumin_audit_core::manifest_evidence::ManifestEvidenceSummary;
use lumin_audit_core::manifest_final::ManifestCloseoutCompanionInput;
use lumin_audit_core::manifest_root::{ManifestCommandRun, ManifestSkippedStep};
use lumin_audit_core::rust_analysis::RustAnalysisRunObservation;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestEvidenceWithArtifactReads<T: Serialize> {
    pub(super) schema_version: &'static str,
    pub(super) evidence: T,
    pub(super) artifact_reads: ManifestEvidenceArtifactReadEvents,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestEvidenceArtifactReadEvents {
    pub(super) schema_version: &'static str,
    pub(super) root_dir: String,
    pub(super) reads: Vec<ArtifactReadObservation>,
}

pub(super) struct ManifestEvidenceSummaryWithReads {
    pub(super) summary: ManifestEvidenceSummary,
    pub(super) artifact_reads: ManifestEvidenceArtifactReadEvents,
    pub(super) result_output: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestLifecycleEvidenceRefreshCliInput {
    pub(super) manifest: serde_json::Value,
    pub(super) lifecycle: ManifestLifecycleUpdateInput,
    pub(super) evidence: ManifestLifecycleEvidenceRefreshInput,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestLifecycleEvidenceRefreshInput {
    pub(super) root: String,
    pub(super) output: PathBuf,
    #[serde(default = "default_true")]
    pub(super) include_tests: bool,
    #[serde(default)]
    pub(super) production: bool,
    #[serde(default = "default_generated_artifacts_mode")]
    pub(super) generated_artifacts_mode: String,
    #[serde(default)]
    pub(super) excludes: Vec<String>,
    #[serde(default)]
    pub(super) auto_excludes: Vec<String>,
    #[serde(default)]
    pub(super) rust_analysis_ran: bool,
    #[serde(default)]
    pub(super) rust_analysis_run: Option<RustAnalysisRunObservation>,
    #[serde(default = "default_true")]
    pub(super) base_pipeline_planned: bool,
    #[serde(default)]
    pub(super) base_pipeline_skip_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestLifecycleEvidenceRefreshResult {
    pub(super) manifest: serde_json::Value,
    pub(super) artifact_reads: ManifestEvidenceArtifactReadEvents,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestRootWithEvidenceCliInput {
    pub(super) generated: String,
    pub(super) profile: String,
    pub(super) root: String,
    pub(super) output: String,
    #[serde(default)]
    pub(super) commands_run: Vec<ManifestCommandRun>,
    #[serde(default)]
    pub(super) skipped: Vec<ManifestSkippedStep>,
    #[serde(default = "default_true")]
    pub(super) include_tests: bool,
    #[serde(default)]
    pub(super) production: bool,
    #[serde(default = "default_generated_artifacts_mode")]
    pub(super) generated_artifacts_mode: String,
    #[serde(default)]
    pub(super) excludes: Vec<String>,
    #[serde(default)]
    pub(super) auto_excludes: Vec<String>,
    #[serde(default)]
    pub(super) rust_analysis_ran: bool,
    #[serde(default)]
    pub(super) rust_analysis_run: Option<RustAnalysisRunObservation>,
    #[serde(default = "default_true")]
    pub(super) base_pipeline_planned: bool,
    #[serde(default)]
    pub(super) base_pipeline_skip_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestRootWithEvidenceResult {
    pub(super) manifest: serde_json::Value,
    pub(super) artifact_reads: ManifestEvidenceArtifactReadEvents,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestCloseoutUpdateCliInput {
    pub(super) output: String,
    pub(super) producer_performance_path: String,
    #[serde(default)]
    pub(super) rust_analysis: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) companion: ManifestCloseoutCompanionInput,
}

fn default_true() -> bool {
    true
}

fn default_generated_artifacts_mode() -> String {
    "default".to_string()
}
