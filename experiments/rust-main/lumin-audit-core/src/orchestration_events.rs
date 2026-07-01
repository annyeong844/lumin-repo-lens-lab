use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::artifact_measurement::measure_artifact_sizes;
use crate::orchestration_plan::AuditProfile;

pub const ORCHESTRATION_LEDGER_SCHEMA_VERSION: &str = "lumin-audit-orchestration-ledger.v1";
pub const PRODUCER_PERFORMANCE_SCHEMA_VERSION: &str = "producer-performance.v1";
pub const PRODUCER_PERFORMANCE_RUNTIME_INPUT_SCHEMA_VERSION: &str =
    "lumin-audit-producer-performance-runtime.v1";
const PRODUCER_PHASE_TIMING_SCHEMA_VERSION: &str = "producer-phase-timing.v1";
const LARGEST_ARTIFACT_READ_LIMIT: usize = 10;

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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerPerformanceRuntimeInput {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    pub output: String,
    pub profile: String,
    pub scan_range: LedgerScanRange,
    pub cache: LedgerCache,
    pub generated_artifacts: LedgerGeneratedArtifacts,
    pub artifact_reads: ArtifactReadSummary,
    #[serde(default)]
    pub artifacts_produced: Vec<String>,
    #[serde(default)]
    pub commands_run: Vec<RuntimeCommandRun>,
    #[serde(default)]
    pub skipped: Vec<RuntimeSkippedRun>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandRun {
    pub step: String,
    pub status: String,
    #[serde(default)]
    pub ms: Option<u64>,
    #[serde(default)]
    pub memory: Option<ProducerMemory>,
    #[serde(default)]
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSkippedRun {
    pub step: String,
    pub reason: String,
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
    pub largest: Vec<ArtifactSizeEntry>,
    #[serde(default)]
    pub by_name: BTreeMap<String, ArtifactSizeBytes>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSizeEntry {
    pub name: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSizeBytes {
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum LedgerEvent {
    Producer(Box<ProducerLedgerEvent>),
    Skipped(Box<SkippedLedgerEvent>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

pub fn build_producer_performance_artifact_from_runtime(
    input: ProducerPerformanceRuntimeInput,
) -> Result<ProducerPerformanceArtifact> {
    validate_required("generated", &input.generated)?;
    validate_required("root", &input.root)?;
    validate_required("output", &input.output)?;
    validate_required("profile", &input.profile)?;
    if input.schema_version != PRODUCER_PERFORMANCE_RUNTIME_INPUT_SCHEMA_VERSION {
        bail!(
            "producer-performance-runtime-artifact: unsupported schemaVersion '{}'",
            input.schema_version
        );
    }

    let output = PathBuf::from(&input.output);
    let mut artifact_reads = input.artifact_reads;
    let events = runtime_events(
        &output,
        &mut artifact_reads,
        &input.commands_run,
        &input.skipped,
    )?;
    let artifacts = measure_artifact_sizes(&output, &input.artifacts_produced);

    build_producer_performance_artifact(OrchestrationLedger {
        schema_version: ORCHESTRATION_LEDGER_SCHEMA_VERSION.to_string(),
        generated: input.generated,
        root: input.root,
        output: input.output,
        profile: input.profile,
        scan_range: input.scan_range,
        cache: input.cache,
        generated_artifacts: input.generated_artifacts,
        artifact_reads,
        artifacts,
        events,
    })
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

fn runtime_events(
    output: &Path,
    artifact_reads: &mut ArtifactReadSummary,
    commands_run: &[RuntimeCommandRun],
    skipped: &[RuntimeSkippedRun],
) -> Result<Vec<LedgerEvent>> {
    let mut events = Vec::new();
    for command in commands_run {
        validate_required("commandsRun[].step", &command.step)?;
        validate_required("commandsRun[].status", &command.status)?;
        let phase_timing = read_phase_timing(output, &command.step, artifact_reads);
        events.push(LedgerEvent::Producer(Box::new(ProducerLedgerEvent {
            name: command.step.clone(),
            status: command.status.clone(),
            wall_ms: command.ms,
            phases: phase_timing
                .as_ref()
                .and_then(|timing| (!timing.phases.is_empty()).then(|| timing.phases.clone())),
            counters: phase_timing.and_then(|timing| timing.counters),
            memory: command.memory.clone(),
            stderr_snippet: command.stderr.clone(),
        })));
    }
    for skipped_run in skipped {
        validate_required("skipped[].step", &skipped_run.step)?;
        validate_required("skipped[].reason", &skipped_run.reason)?;
        events.push(LedgerEvent::Skipped(Box::new(SkippedLedgerEvent {
            name: skipped_run.step.clone(),
            reason: skipped_run.reason.clone(),
        })));
    }
    Ok(events)
}

struct PhaseTiming {
    phases: Vec<Value>,
    counters: Option<Value>,
}

fn read_phase_timing(
    output: &Path,
    producer: &str,
    artifact_reads: &mut ArtifactReadSummary,
) -> Option<PhaseTiming> {
    let path = producer_phase_timing_path(output, producer);
    let started = Instant::now();
    let raw = fs::read_to_string(&path).ok()?;
    let read_ms = elapsed_ms(started);
    let parse_started = Instant::now();
    let parsed = match serde_json::from_str::<Value>(&raw) {
        Ok(parsed) => {
            record_artifact_read(
                artifact_reads,
                output,
                &path,
                raw.len() as u64,
                read_ms,
                elapsed_ms(parse_started),
                true,
            );
            parsed
        }
        Err(_) => {
            record_artifact_read(
                artifact_reads,
                output,
                &path,
                raw.len() as u64,
                read_ms,
                0,
                false,
            );
            return None;
        }
    };

    if parsed.get("schemaVersion").and_then(Value::as_str)
        != Some(PRODUCER_PHASE_TIMING_SCHEMA_VERSION)
    {
        return None;
    }

    let phases = parsed
        .get("phases")
        .and_then(Value::as_array)
        .map(|phases| {
            phases
                .iter()
                .filter_map(|phase| {
                    let name = phase.get("name").and_then(Value::as_str)?;
                    let wall_ms = rounded_non_negative_u64(phase.get("wallMs")?)?;
                    Some(json!({ "name": name, "wallMs": wall_ms }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let counters = parsed
        .get("counters")
        .and_then(Value::as_object)
        .map(|counters| {
            let mut sanitized = Map::new();
            for (name, value) in counters {
                if let Some(number) = rounded_non_negative_u64(value) {
                    sanitized.insert(name.clone(), json!(number));
                }
            }
            sanitized
        })
        .filter(|counters| !counters.is_empty())
        .map(Value::Object);

    Some(PhaseTiming { phases, counters })
}

fn producer_phase_timing_path(output: &Path, producer: &str) -> PathBuf {
    output
        .join(".producer-phases")
        .join(format!("{}.json", safe_producer_file_name(producer)))
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

fn rounded_non_negative_u64(value: &Value) -> Option<u64> {
    let number = value.as_f64()?;
    number.is_finite().then(|| number.max(0.0).round() as u64)
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

#[derive(Debug, Clone, Default)]
struct ArtifactReadMetricEntry {
    read_count: u64,
    total_bytes: u64,
    total_read_ms: u64,
    total_json_parse_ms: u64,
    parse_failure_count: u64,
}

fn record_artifact_read(
    summary: &mut ArtifactReadSummary,
    root: &Path,
    file_path: &Path,
    bytes: u64,
    read_ms: u64,
    json_parse_ms: u64,
    ok: bool,
) {
    let mut by_name = artifact_read_entries(&summary.by_name);
    let name = artifact_metric_name(root, file_path);
    let entry = by_name.entry(name).or_default();
    entry.read_count += 1;
    entry.total_bytes += bytes;
    entry.total_read_ms += read_ms;
    entry.total_json_parse_ms += json_parse_ms;
    if !ok {
        entry.parse_failure_count += 1;
        summary.parse_failure_count += 1;
    }

    summary.total_read_count += 1;
    summary.total_read_bytes += bytes;
    summary.total_read_ms += read_ms;
    summary.total_json_parse_ms += json_parse_ms;
    refresh_artifact_read_projections(summary, by_name);
}

fn artifact_read_entries(value: &Value) -> BTreeMap<String, ArtifactReadMetricEntry> {
    let Some(object) = value.as_object() else {
        return BTreeMap::new();
    };
    object
        .iter()
        .map(|(name, value)| {
            (
                name.clone(),
                ArtifactReadMetricEntry {
                    read_count: number_field(value, "readCount"),
                    total_bytes: number_field(value, "totalBytes"),
                    total_read_ms: number_field(value, "totalReadMs"),
                    total_json_parse_ms: number_field(value, "totalJsonParseMs"),
                    parse_failure_count: number_field(value, "parseFailureCount"),
                },
            )
        })
        .collect()
}

fn number_field(value: &Value, field: &str) -> u64 {
    value.get(field).and_then(Value::as_u64).unwrap_or(0)
}

fn refresh_artifact_read_projections(
    summary: &mut ArtifactReadSummary,
    entries: BTreeMap<String, ArtifactReadMetricEntry>,
) {
    let mut by_name = Map::new();
    for (name, entry) in &entries {
        by_name.insert(
            name.clone(),
            json!({
                "readCount": entry.read_count,
                "totalBytes": entry.total_bytes,
                "totalReadMs": entry.total_read_ms,
                "totalJsonParseMs": entry.total_json_parse_ms,
                "parseFailureCount": entry.parse_failure_count,
            }),
        );
    }
    summary.by_name = Value::Object(by_name);

    let mut largest_reads = entries
        .iter()
        .map(|(name, entry)| {
            json!({
                "name": name,
                "bytes": entry.total_bytes,
                "readCount": entry.read_count,
            })
        })
        .collect::<Vec<_>>();
    largest_reads.sort_by(|left, right| {
        number_field(right, "bytes")
            .cmp(&number_field(left, "bytes"))
            .then_with(|| string_field(left, "name").cmp(&string_field(right, "name")))
    });
    largest_reads.truncate(LARGEST_ARTIFACT_READ_LIMIT);
    summary.largest_reads = largest_reads;

    let mut slowest_json_parses = entries
        .iter()
        .filter(|(_, entry)| entry.total_json_parse_ms > 0)
        .map(|(name, entry)| {
            json!({
                "name": name,
                "jsonParseMs": entry.total_json_parse_ms,
                "readCount": entry.read_count,
            })
        })
        .collect::<Vec<_>>();
    slowest_json_parses.sort_by(|left, right| {
        number_field(right, "jsonParseMs")
            .cmp(&number_field(left, "jsonParseMs"))
            .then_with(|| string_field(left, "name").cmp(&string_field(right, "name")))
    });
    slowest_json_parses.truncate(LARGEST_ARTIFACT_READ_LIMIT);
    summary.slowest_json_parses = slowest_json_parses;
}

fn string_field(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn artifact_metric_name(root: &Path, file_path: &Path) -> String {
    if let Ok(relative) = file_path.strip_prefix(root) {
        let text = path_to_posix(relative);
        if !text.is_empty() {
            return text;
        }
    }
    file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn path_to_posix(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
