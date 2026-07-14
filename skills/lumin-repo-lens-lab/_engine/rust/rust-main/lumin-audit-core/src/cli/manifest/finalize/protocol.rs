use serde::{Deserialize, Serialize};

use lumin_audit_core::artifact_read_metrics::ArtifactReadMetricsRequest;
use lumin_audit_core::manifest_final::{ManifestCloseoutCompanionInput, ManifestCloseoutUpdate};
use lumin_audit_core::orchestration_events::{
    ProducerPerformanceAuditRunContext, ProducerPerformanceRuntimeObservations, RuntimeCommandRun,
    RuntimeSkippedRun,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestWriteCliInput {
    pub(super) manifest: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestWriteResult {
    pub(super) manifest_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestCloseoutWriteCliInput {
    pub(super) manifest: serde_json::Value,
    pub(super) output: String,
    pub(super) producer_performance_path: String,
    #[serde(default)]
    pub(super) rust_analysis: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) companion: ManifestCloseoutCompanionInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ManifestCloseoutWriteResult {
    pub(super) manifest_path: String,
    pub(super) closeout_update: ManifestCloseoutUpdate,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FinalizeAuditRunCliInput {
    pub(super) manifest: serde_json::Value,
    pub(super) context: ProducerPerformanceAuditRunContext,
    pub(super) observations: ProducerPerformanceRuntimeObservations,
    #[serde(default)]
    pub(super) rust_analysis: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) companion: ManifestCloseoutCompanionInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FinalizeAuditRunResult {
    pub(super) producer_performance_path: String,
    pub(super) manifest_path: String,
    pub(super) closeout_update: ManifestCloseoutUpdate,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FinalizeAuditRunWithCompanionsCliInput {
    pub(super) manifest: serde_json::Value,
    pub(super) context: ProducerPerformanceAuditRunContext,
    pub(super) artifact_read_events: ArtifactReadMetricsRequest,
    #[serde(default)]
    pub(super) commands_run: Vec<RuntimeCommandRun>,
    #[serde(default)]
    pub(super) skipped: Vec<RuntimeSkippedRun>,
    #[serde(default)]
    pub(super) rust_analysis: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) companions: FinalizeAuditRunCompanionPlan,
    #[serde(default)]
    pub(super) companion_policy: Option<FinalizeAuditRunCompanionPolicy>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FinalizeAuditRunCompanionPlan {
    #[serde(default)]
    pub(super) topology_mermaid: bool,
    #[serde(default)]
    pub(super) audit_summary: bool,
    #[serde(default)]
    pub(super) review_pack: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FinalizeAuditRunCompanionPolicy {
    #[serde(default)]
    pub(super) base_pipeline_planned: bool,
    #[serde(default)]
    pub(super) base_pipeline_skip_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FinalizeAuditRunWithCompanionsResult {
    pub(super) producer_performance_path: String,
    pub(super) manifest_path: String,
    pub(super) topology_mermaid_path: Option<String>,
    pub(super) audit_summary_path: Option<String>,
    pub(super) review_pack_path: Option<String>,
    pub(super) audit_summary_preview: Option<String>,
    pub(super) artifacts_produced_count: usize,
    pub(super) blind_zones: Vec<serde_json::Value>,
    pub(super) blind_zones_summary: String,
    pub(super) closeout_update: ManifestCloseoutUpdate,
}
