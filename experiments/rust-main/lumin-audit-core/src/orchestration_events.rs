use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::orchestration_plan::AuditProfile;

pub const ORCHESTRATION_LEDGER_SCHEMA_VERSION: &str = "lumin-audit-orchestration-ledger.v1";
pub const PRODUCER_PERFORMANCE_SCHEMA_VERSION: &str = "producer-performance.v1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationLedger {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    pub output: String,
    pub profile: String,
    pub scan_range: LedgerScanRange,
    pub cache: LedgerCache,
    pub generated_artifacts: LedgerGeneratedArtifacts,
    pub artifact_reads: ArtifactReadSummary,
    pub artifacts: ArtifactSizeSummary,
    pub events: Vec<LedgerEvent>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerScanRange {
    pub include_tests: bool,
    pub production: bool,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub auto_excludes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerCache {
    pub no_incremental: bool,
    pub cache_root: String,
    pub clear_incremental_cache: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerGeneratedArtifacts {
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactReadSummary {
    pub schema_version: String,
    pub measurement: String,
    pub total_read_count: u64,
    pub total_read_bytes: u64,
    pub total_read_ms: u64,
    pub total_json_parse_ms: u64,
    pub parse_failure_count: u64,
    #[serde(default)]
    pub largest_reads: Vec<Value>,
    #[serde(default)]
    pub slowest_json_parses: Vec<Value>,
    #[serde(default)]
    pub by_name: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSizeSummary {
    pub produced_count: u64,
    pub total_bytes: u64,
    #[serde(default)]
    pub largest: Vec<Value>,
    #[serde(default)]
    pub by_name: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum LedgerEvent {
    Producer(Box<ProducerLedgerEvent>),
    Skipped(Box<SkippedLedgerEvent>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerLedgerEvent {
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub wall_ms: Option<u64>,
    #[serde(default)]
    pub phases: Option<Vec<Value>>,
    #[serde(default)]
    pub counters: Option<Value>,
    #[serde(default)]
    pub memory: Option<ProducerMemory>,
    #[serde(default)]
    pub stderr_snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedLedgerEvent {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerMemory {
    pub before: MemorySnapshot,
    pub after: MemorySnapshot,
    pub delta: MemorySnapshot,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySnapshot {
    pub rss_bytes: i64,
    #[serde(default)]
    pub heap_total_bytes: i64,
    #[serde(default)]
    pub heap_used_bytes: i64,
    #[serde(default)]
    pub external_bytes: i64,
    #[serde(default)]
    pub array_buffers_bytes: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerPerformanceArtifact {
    pub schema_version: &'static str,
    pub generated: String,
    pub root: String,
    pub output: String,
    pub profile: AuditProfile,
    pub scan_range: LedgerScanRange,
    pub cache: LedgerCache,
    pub generated_artifacts: LedgerGeneratedArtifacts,
    pub summary: ProducerPerformanceArtifactSummary,
    pub memory: ProducerPerformanceMemoryPolicy,
    pub artifacts: ArtifactSizeSummary,
    pub artifact_reads: ArtifactReadSummary,
    pub producers: Vec<ProducerPerformanceEntry>,
    pub skipped: Vec<SkippedPerformanceEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerPerformanceArtifactSummary {
    pub producer_count: usize,
    pub ok_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub total_wall_ms: u64,
    pub artifact_count: u64,
    pub total_artifact_bytes: u64,
    pub artifact_read_count: u64,
    pub total_artifact_read_bytes: u64,
    pub total_json_parse_ms: u64,
    pub max_observed_orchestrator_rss_bytes: i64,
    pub phase_support_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerPerformanceMemoryPolicy {
    pub measurement: &'static str,
    pub child_peak_rss_available: bool,
    pub note: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerPerformanceEntry {
    pub name: String,
    pub status: String,
    pub wall_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phases: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counters: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<ProducerMemory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedPerformanceEntry {
    pub name: String,
    pub status: &'static str,
    pub reason: String,
}

pub fn build_producer_performance_artifact(
    ledger: OrchestrationLedger,
) -> Result<ProducerPerformanceArtifact> {
    validate_required("generated", &ledger.generated)?;
    validate_required("root", &ledger.root)?;
    validate_required("output", &ledger.output)?;
    validate_required("profile", &ledger.profile)?;
    if ledger.schema_version != ORCHESTRATION_LEDGER_SCHEMA_VERSION {
        bail!(
            "producer-performance-artifact: unsupported ledger schemaVersion '{}'",
            ledger.schema_version
        );
    }
    let profile = AuditProfile::parse(&ledger.profile)?;

    let mut producers = Vec::new();
    let mut skipped = Vec::new();
    for event in ledger.events {
        match event {
            LedgerEvent::Producer(event) => {
                let event = *event;
                validate_required("producer event name", &event.name)?;
                validate_required("producer event status", &event.status)?;
                producers.push(ProducerPerformanceEntry {
                    name: event.name,
                    status: event.status,
                    wall_ms: event.wall_ms,
                    phases: event.phases.filter(|phases| !phases.is_empty()),
                    counters: event.counters,
                    memory: event.memory,
                    stderr_snippet: event.stderr_snippet,
                });
            }
            LedgerEvent::Skipped(event) => {
                let event = *event;
                validate_required("skipped event name", &event.name)?;
                skipped.push(SkippedPerformanceEntry {
                    name: event.name,
                    status: "skipped",
                    reason: event.reason,
                });
            }
        }
    }

    let summary = ProducerPerformanceArtifactSummary {
        producer_count: producers.len(),
        ok_count: producers
            .iter()
            .filter(|entry| entry.status == "ok")
            .count(),
        failed_count: producers
            .iter()
            .filter(|entry| entry.status.starts_with("failed"))
            .count(),
        skipped_count: skipped.len(),
        total_wall_ms: producers
            .iter()
            .map(|entry| entry.wall_ms.unwrap_or(0))
            .sum(),
        artifact_count: ledger.artifacts.produced_count,
        total_artifact_bytes: ledger.artifacts.total_bytes,
        artifact_read_count: ledger.artifact_reads.total_read_count,
        total_artifact_read_bytes: ledger.artifact_reads.total_read_bytes,
        total_json_parse_ms: ledger.artifact_reads.total_json_parse_ms,
        max_observed_orchestrator_rss_bytes: max_observed_rss(&producers),
        phase_support_count: producers
            .iter()
            .filter(|entry| {
                entry
                    .phases
                    .as_ref()
                    .is_some_and(|phases| !phases.is_empty())
            })
            .count(),
    };

    Ok(ProducerPerformanceArtifact {
        schema_version: PRODUCER_PERFORMANCE_SCHEMA_VERSION,
        generated: ledger.generated,
        root: ledger.root,
        output: ledger.output,
        profile,
        scan_range: ledger.scan_range,
        cache: ledger.cache,
        generated_artifacts: ledger.generated_artifacts,
        summary,
        memory: ProducerPerformanceMemoryPolicy {
            measurement: "orchestrator-process-snapshots",
            child_peak_rss_available: false,
            note: "Memory snapshots are measured in the audit-repo orchestrator before and after each child producer; they do not measure child process peak RSS.",
        },
        artifacts: ledger.artifacts,
        artifact_reads: ledger.artifact_reads,
        producers,
        skipped,
    })
}

fn max_observed_rss(entries: &[ProducerPerformanceEntry]) -> i64 {
    entries
        .iter()
        .filter_map(|entry| entry.memory.as_ref())
        .flat_map(|memory| [memory.before.rss_bytes, memory.after.rss_bytes])
        .max()
        .unwrap_or(0)
}

fn validate_required(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("producer-performance-artifact: {field} must be a non-empty string");
    }
    Ok(())
}
