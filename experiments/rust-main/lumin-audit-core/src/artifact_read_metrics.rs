use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const ARTIFACT_READ_EVENTS_SCHEMA_VERSION: &str = "lumin-audit-artifact-read-events.v1";
pub const ARTIFACT_READ_METRICS_SCHEMA_VERSION: &str = "artifact-read-metrics.v1";
pub const ARTIFACT_READ_MEASUREMENT: &str = "audit-repo-orchestrator-json-reads";
pub const DEFAULT_LARGEST_READ_LIMIT: usize = 10;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactReadMetricsRequest {
    pub schema_version: String,
    #[serde(default)]
    pub root_dir: Option<String>,
    #[serde(default)]
    pub largest_limit: Option<usize>,
    #[serde(default)]
    pub reads: Vec<ArtifactReadObservation>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactReadObservation {
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub bytes: Value,
    #[serde(default)]
    pub read_ms: Value,
    #[serde(default)]
    pub json_parse_ms: Value,
    #[serde(default)]
    pub ok: Option<bool>,
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

impl ArtifactReadSummary {
    pub fn empty() -> Self {
        Self {
            schema_version: ARTIFACT_READ_METRICS_SCHEMA_VERSION.to_string(),
            measurement: ARTIFACT_READ_MEASUREMENT.to_string(),
            total_read_count: 0,
            total_read_bytes: 0,
            total_read_ms: 0,
            total_json_parse_ms: 0,
            parse_failure_count: 0,
            largest_reads: Vec::new(),
            slowest_json_parses: Vec::new(),
            by_name: Value::Object(Map::new()),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ArtifactReadMetricEntry {
    read_count: u64,
    total_bytes: u64,
    total_read_ms: u64,
    total_json_parse_ms: u64,
    parse_failure_count: u64,
}

pub(crate) struct ArtifactReadRecord<'a> {
    pub root: Option<&'a Path>,
    pub file_path: &'a Path,
    pub bytes: u64,
    pub read_ms: u64,
    pub json_parse_ms: u64,
    pub ok: bool,
    pub largest_limit: usize,
}

pub fn summarize_artifact_read_events(
    request: ArtifactReadMetricsRequest,
) -> Result<ArtifactReadSummary> {
    if request.schema_version != ARTIFACT_READ_EVENTS_SCHEMA_VERSION {
        bail!(
            "artifact-read-metrics-summary: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let root = request.root_dir.as_deref().map(PathBuf::from);
    let limit = request.largest_limit.unwrap_or(DEFAULT_LARGEST_READ_LIMIT);
    let mut summary = ArtifactReadSummary::empty();
    for read in request.reads {
        let path = read.file_path.unwrap_or_else(|| "unknown".to_string());
        record_artifact_read(
            &mut summary,
            ArtifactReadRecord {
                root: root.as_deref(),
                file_path: Path::new(&path),
                bytes: rounded_non_negative_u64(&read.bytes),
                read_ms: rounded_non_negative_u64(&read.read_ms),
                json_parse_ms: rounded_non_negative_u64(&read.json_parse_ms),
                ok: read.ok.unwrap_or(true),
                largest_limit: limit,
            },
        );
    }
    Ok(summary)
}

pub(crate) fn record_artifact_read(
    summary: &mut ArtifactReadSummary,
    record: ArtifactReadRecord<'_>,
) {
    let mut by_name = artifact_read_entries(&summary.by_name);
    let name = artifact_metric_name(record.root, record.file_path);
    let entry = by_name.entry(name).or_default();
    entry.read_count += 1;
    entry.total_bytes += record.bytes;
    entry.total_read_ms += record.read_ms;
    entry.total_json_parse_ms += record.json_parse_ms;
    if !record.ok {
        entry.parse_failure_count += 1;
        summary.parse_failure_count += 1;
    }

    summary.total_read_count += 1;
    summary.total_read_bytes += record.bytes;
    summary.total_read_ms += record.read_ms;
    summary.total_json_parse_ms += record.json_parse_ms;
    refresh_artifact_read_projections(summary, by_name, record.largest_limit);
}

fn artifact_metric_name(root: Option<&Path>, file_path: &Path) -> String {
    let Some(root) = root else {
        return basename(file_path);
    };
    let Ok(relative) = file_path.strip_prefix(root) else {
        return basename(file_path);
    };
    let value = relative.to_string_lossy().replace('\\', "/");
    if value.is_empty() {
        basename(file_path)
    } else {
        value
    }
}

fn basename(file_path: &Path) -> String {
    file_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn rounded_non_negative_u64(value: &Value) -> u64 {
    let number = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        Value::Bool(true) => Some(1.0),
        Value::Bool(false) => Some(0.0),
        _ => None,
    };
    number
        .filter(|number| number.is_finite())
        .map(|number| number.max(0.0).round() as u64)
        .unwrap_or(0)
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
    largest_limit: usize,
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
            .then_with(|| string_field(left, "name").cmp(string_field(right, "name")))
    });
    largest_reads.truncate(largest_limit);
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
            .then_with(|| string_field(left, "name").cmp(string_field(right, "name")))
    });
    slowest_json_parses.truncate(largest_limit);
    summary.slowest_json_parses = slowest_json_parses;
}

fn string_field<'a>(value: &'a Value, field: &str) -> &'a str {
    value.get(field).and_then(Value::as_str).unwrap_or("")
}
