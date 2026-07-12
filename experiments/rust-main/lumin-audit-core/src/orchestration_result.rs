use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

const ORCHESTRATION_EXAMPLE_LIMIT: usize = 5;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationResultSummary {
    pub artifact: &'static str,
    pub schema_version: Value,
    pub summary_owner: &'static str,
    pub execution_owner: &'static str,
    pub source_status: OrchestrationSourceStatus,
    pub status: OrchestrationExecutionStatus,
    pub executed_step_count: u64,
    pub ok_count: u64,
    pub failed_step_count: u64,
    pub failed_required_count: u64,
    pub failed_optional_count: u64,
    pub failed_other_count: u64,
    pub skipped_step_count: u64,
    pub observed_status_counts: BTreeMap<String, u64>,
    pub required_failure_examples: Vec<OrchestrationStepExample>,
    pub optional_failure_examples: Vec<OrchestrationStepExample>,
    pub skipped_examples: Vec<OrchestrationStepExample>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrchestrationSourceStatus {
    Available,
    InvalidShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrchestrationExecutionStatus {
    Complete,
    Degraded,
    FailedRequired,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationStepExample {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrchestrationResultArtifactInput {
    #[serde(default)]
    schema_version: Value,
    producers: Vec<OrchestrationProducerInput>,
    skipped: Vec<OrchestrationSkippedInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrchestrationProducerInput {
    name: String,
    status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrchestrationSkippedInput {
    name: String,
    reason: String,
}

pub fn summarize_orchestration_result(artifact: &Value) -> OrchestrationResultSummary {
    let schema_version = field_or_null(artifact, "schemaVersion");
    let Ok(input) = OrchestrationResultArtifactInput::deserialize(artifact) else {
        return unavailable_summary(schema_version);
    };

    let mut observed_status_counts = BTreeMap::new();
    let mut ok_count = 0;
    let mut failed_required_count = 0;
    let mut failed_optional_count = 0;
    let mut failed_other_count = 0;
    let mut required_failure_examples = Vec::new();
    let mut optional_failure_examples = Vec::new();

    for producer in &input.producers {
        let status = producer.status.clone();
        *observed_status_counts.entry(status.clone()).or_insert(0) += 1;
        match status.as_str() {
            "ok" => ok_count += 1,
            "failed-required" => {
                failed_required_count += 1;
                push_producer_example(&mut required_failure_examples, producer, status, None);
            }
            "failed-optional" => {
                failed_optional_count += 1;
                push_producer_example(&mut optional_failure_examples, producer, status, None);
            }
            status if status.starts_with("failed") => {
                failed_other_count += 1;
                push_producer_example(
                    &mut optional_failure_examples,
                    producer,
                    status.to_string(),
                    None,
                );
            }
            _ => {}
        }
    }

    let mut skipped_examples = Vec::new();
    for skipped_step in &input.skipped {
        push_skipped_example(
            &mut skipped_examples,
            skipped_step,
            "skipped".to_string(),
            Some(skipped_step.reason.clone()),
        );
    }

    let failed_step_count = failed_required_count + failed_optional_count + failed_other_count;
    let status = if failed_required_count > 0 {
        OrchestrationExecutionStatus::FailedRequired
    } else if failed_step_count > 0 {
        OrchestrationExecutionStatus::Degraded
    } else {
        OrchestrationExecutionStatus::Complete
    };

    OrchestrationResultSummary {
        artifact: "producer-performance.json",
        schema_version: input.schema_version,
        summary_owner: "lumin-audit-core",
        execution_owner: "audit-repo.mjs",
        source_status: OrchestrationSourceStatus::Available,
        status,
        executed_step_count: input.producers.len() as u64,
        ok_count,
        failed_step_count,
        failed_required_count,
        failed_optional_count,
        failed_other_count,
        skipped_step_count: input.skipped.len() as u64,
        observed_status_counts,
        required_failure_examples,
        optional_failure_examples,
        skipped_examples,
    }
}

fn unavailable_summary(schema_version: Value) -> OrchestrationResultSummary {
    OrchestrationResultSummary {
        artifact: "producer-performance.json",
        schema_version,
        summary_owner: "lumin-audit-core",
        execution_owner: "audit-repo.mjs",
        source_status: OrchestrationSourceStatus::InvalidShape,
        status: OrchestrationExecutionStatus::Unavailable,
        executed_step_count: 0,
        ok_count: 0,
        failed_step_count: 0,
        failed_required_count: 0,
        failed_optional_count: 0,
        failed_other_count: 0,
        skipped_step_count: 0,
        observed_status_counts: BTreeMap::new(),
        required_failure_examples: Vec::new(),
        optional_failure_examples: Vec::new(),
        skipped_examples: Vec::new(),
    }
}

fn field_or_null(value: &Value, key: &str) -> Value {
    value.get(key).cloned().unwrap_or(Value::Null)
}

fn push_producer_example(
    examples: &mut Vec<OrchestrationStepExample>,
    value: &OrchestrationProducerInput,
    status: String,
    reason: Option<String>,
) {
    if examples.len() >= ORCHESTRATION_EXAMPLE_LIMIT {
        return;
    }
    examples.push(OrchestrationStepExample {
        name: value.name.clone(),
        status,
        reason,
    });
}

fn push_skipped_example(
    examples: &mut Vec<OrchestrationStepExample>,
    value: &OrchestrationSkippedInput,
    status: String,
    reason: Option<String>,
) {
    if examples.len() >= ORCHESTRATION_EXAMPLE_LIMIT {
        return;
    }
    examples.push(OrchestrationStepExample {
        name: value.name.clone(),
        status,
        reason,
    });
}
