use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

use crate::artifact_registry::collect_produced_artifacts;
use crate::orchestration_result::{summarize_orchestration_result, OrchestrationResultSummary};
use crate::producer_performance::{summarize_producer_performance, ProducerPerformanceSummary};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFinalSummaryUpdate {
    pub performance: ProducerPerformanceSummary,
    pub orchestration: OrchestrationResultSummary,
    pub artifacts_produced: Vec<String>,
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
