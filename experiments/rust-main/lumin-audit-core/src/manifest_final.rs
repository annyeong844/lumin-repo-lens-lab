use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

use crate::artifact_registry::{
    collect_produced_artifacts, collect_produced_artifacts_for_manifest,
};
use crate::manifest_companion::{
    build_manifest_companion_update, ManifestCompanionUpdate, ManifestCompanionUpdateInput,
};
use crate::orchestration_result::{summarize_orchestration_result, OrchestrationResultSummary};
use crate::producer_performance::{summarize_producer_performance, ProducerPerformanceSummary};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFinalSummaryUpdate {
    pub performance: ProducerPerformanceSummary,
    pub orchestration: OrchestrationResultSummary,
    pub artifacts_produced: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestArtifactsProducedUpdate {
    pub artifacts_produced: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCloseoutCompanionInput {
    #[serde(default)]
    pub topology_mermaid_path: Option<String>,
    #[serde(default)]
    pub audit_summary_path: Option<String>,
    #[serde(default)]
    pub review_pack_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCloseoutUpdate {
    pub performance: ProducerPerformanceSummary,
    pub orchestration: OrchestrationResultSummary,
    pub artifacts_produced: Vec<String>,
    #[serde(flatten)]
    pub companion: ManifestCompanionUpdate,
}

pub fn build_manifest_artifacts_produced_update(
    output: &Path,
    rust_analysis: Option<&Value>,
) -> Result<ManifestArtifactsProducedUpdate> {
    Ok(ManifestArtifactsProducedUpdate {
        artifacts_produced: collect_produced_artifacts_for_manifest(output, rust_analysis)?,
    })
}

pub fn build_manifest_closeout_update(
    output: &Path,
    producer_performance: &Value,
    rust_analysis: Option<&Value>,
    companion: ManifestCloseoutCompanionInput,
) -> Result<ManifestCloseoutUpdate> {
    Ok(ManifestCloseoutUpdate {
        performance: summarize_producer_performance(producer_performance),
        orchestration: summarize_orchestration_result(producer_performance),
        artifacts_produced: collect_produced_artifacts_for_manifest(output, rust_analysis)?,
        companion: build_manifest_companion_update(ManifestCompanionUpdateInput {
            topology_mermaid_path: companion.topology_mermaid_path,
            audit_summary_path: companion.audit_summary_path,
            review_pack_path: companion.review_pack_path,
        })?,
    })
}

pub fn build_manifest_final_summary_update(
    output: &Path,
    producer_performance: &Value,
    rust_analysis_usable: bool,
) -> Result<ManifestFinalSummaryUpdate> {
    Ok(ManifestFinalSummaryUpdate {
        performance: summarize_producer_performance(producer_performance),
        orchestration: summarize_orchestration_result(producer_performance),
        artifacts_produced: collect_produced_artifacts(output, rust_analysis_usable)?,
    })
}

pub fn build_manifest_final_summary_update_for_rust_analysis(
    output: &Path,
    producer_performance: &Value,
    rust_analysis: Option<&Value>,
) -> Result<ManifestFinalSummaryUpdate> {
    Ok(ManifestFinalSummaryUpdate {
        performance: summarize_producer_performance(producer_performance),
        orchestration: summarize_orchestration_result(producer_performance),
        artifacts_produced: collect_produced_artifacts_for_manifest(output, rust_analysis)?,
    })
}
