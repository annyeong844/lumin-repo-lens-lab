use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerPerformanceSummary {
    pub artifact: &'static str,
    pub schema_version: Value,
    pub producer_count: u64,
    pub ok_count: u64,
    pub failed_count: u64,
    pub skipped_count: u64,
    pub total_wall_ms: u64,
    pub artifact_count: u64,
    pub total_artifact_bytes: u64,
    pub artifact_read_count: u64,
    pub total_artifact_read_bytes: u64,
    pub total_json_parse_ms: u64,
    pub phase_support_count: u64,
    pub largest_artifacts: Value,
    pub max_observed_orchestrator_rss_bytes: u64,
}

pub fn summarize_producer_performance(artifact: &Value) -> ProducerPerformanceSummary {
    let summary = artifact.get("summary");
    ProducerPerformanceSummary {
        artifact: "producer-performance.json",
        schema_version: field_or_null(artifact, "schemaVersion"),
        producer_count: number_or_zero(summary, "producerCount"),
        ok_count: number_or_zero(summary, "okCount"),
        failed_count: number_or_zero(summary, "failedCount"),
        skipped_count: number_or_zero(summary, "skippedCount"),
        total_wall_ms: number_or_zero(summary, "totalWallMs"),
        artifact_count: number_or_zero(summary, "artifactCount"),
        total_artifact_bytes: number_or_zero(summary, "totalArtifactBytes"),
        artifact_read_count: number_or_zero(summary, "artifactReadCount"),
        total_artifact_read_bytes: number_or_zero(summary, "totalArtifactReadBytes"),
        total_json_parse_ms: number_or_zero(summary, "totalJsonParseMs"),
        phase_support_count: number_or_zero(summary, "phaseSupportCount"),
        largest_artifacts: artifact
            .get("artifacts")
            .and_then(|artifacts| artifacts.get("largest"))
            .cloned()
            .unwrap_or_else(|| json!([])),
        max_observed_orchestrator_rss_bytes: number_or_zero(
            summary,
            "maxObservedOrchestratorRssBytes",
        ),
    }
}

fn field_or_null(value: &Value, key: &str) -> Value {
    value.get(key).cloned().unwrap_or(Value::Null)
}

fn number_or_zero(value: Option<&Value>, key: &str) -> u64 {
    value
        .and_then(|value| value.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}
