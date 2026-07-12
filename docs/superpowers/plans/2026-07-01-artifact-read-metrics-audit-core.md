# Artifact Read Metrics Audit-Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move ordinary audit artifact-read metric aggregation from JS into `lumin-audit-core` while keeping ordinary JSON file read/parse semantics JS-owned.

**Architecture:** Add a focused Rust `artifact_read_metrics` module that receives raw JS read observations and emits the existing `artifact-read-metrics.v1` summary. Reuse that module from `orchestration_events.rs` for phase-sidecar read merges, then replace the JS summary math with a thin raw-event collector that calls audit-core.

**Tech Stack:** Rust 2024, `serde`, `serde_json`, `anyhow`, existing `lumin-audit-core` CLI wrappers, Node MJS compatibility wrappers.

---

## File Structure

- Create `experiments/rust-main/lumin-audit-core/src/artifact_read_metrics.rs`
  - Owns raw read event request shape, metric-name normalization, read counter aggregation, largest/slowest projections, and summary updates.
- Create `experiments/rust-main/lumin-audit-core/tests/artifact_read_metrics.rs`
  - Product behavior tests for JS contract parity and CLI output.
- Modify `experiments/rust-main/lumin-audit-core/src/lib.rs`
  - Add `pub mod artifact_read_metrics;`.
  - Preserve any existing local changes such as `pub mod pre_write_routing;`.
- Modify `experiments/rust-main/lumin-audit-core/src/cli.rs`
  - Add `artifact-read-metrics-summary --input <path|->`.
- Modify `experiments/rust-main/lumin-audit-core/src/orchestration_events.rs`
  - Import the shared `ArtifactReadSummary` and `record_artifact_read` helpers.
  - Remove duplicated local artifact-read summary math.
- Modify `_lib/audit-manifest.mjs`
  - Export `buildArtifactReadMetricsSummary(request)`.
- Create `_lib/artifact-read-metrics.mjs`
  - Provide `createArtifactReadMetrics({ rootDir, largestLimit })` as a thin raw-event collector whose `summary()` calls audit-core.
- Modify `_lib/artifacts.mjs`
  - Remove the JS summary math export; keep `readJsonFile()` and `loadIfExists()` read/parse behavior unchanged.
- Modify `audit-repo.mjs`
  - Import `createArtifactReadMetrics` from `_lib/artifact-read-metrics.mjs`.
- Mirror modified JS files under `skills/lumin-repo-lens-lab/_engine/...`.
- Modify `canonical/audit-core.md`
  - Register `artifact_read_metrics.rs` as owner.
  - Narrow the remaining JS-owned boundary to ordinary JSON read/parse events.

Current worktree caveat: `experiments/rust-main/lumin-audit-core/src/lib.rs` and `experiments/rust-main/lumin-audit-core/src/pre_write_routing.rs` may already contain unrelated local work. Do not delete or revert those lines.

---

### Task 1: Add Rust Artifact-Read Metrics Core

**Files:**
- Create: `experiments/rust-main/lumin-audit-core/src/artifact_read_metrics.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/artifact_read_metrics.rs`

- [ ] **Step 1: Create the focused Rust module**

Create `experiments/rust-main/lumin-audit-core/src/artifact_read_metrics.rs` with this shape:

```rust
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const ARTIFACT_READ_EVENTS_SCHEMA_VERSION: &str =
    "lumin-audit-artifact-read-events.v1";
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
pub struct ArtifactReadMetricEntry {
    read_count: u64,
    total_bytes: u64,
    total_read_ms: u64,
    total_json_parse_ms: u64,
    parse_failure_count: u64,
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
    let limit = request
        .largest_limit
        .unwrap_or(DEFAULT_LARGEST_READ_LIMIT);
    let mut summary = ArtifactReadSummary::empty();
    for read in request.reads {
        let path = read.file_path.unwrap_or_else(|| "unknown".to_string());
        record_artifact_read(
            &mut summary,
            root.as_deref(),
            Path::new(&path),
            rounded_non_negative_u64(&read.bytes),
            rounded_non_negative_u64(&read.read_ms),
            rounded_non_negative_u64(&read.json_parse_ms),
            read.ok.unwrap_or(true),
            limit,
        );
    }
    Ok(summary)
}

pub fn record_artifact_read(
    summary: &mut ArtifactReadSummary,
    root: Option<&Path>,
    file_path: &Path,
    bytes: u64,
    read_ms: u64,
    json_parse_ms: u64,
    ok: bool,
    largest_limit: usize,
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
    refresh_artifact_read_projections(summary, by_name, largest_limit);
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
            .then_with(|| string_field(left, "name").cmp(&string_field(right, "name")))
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
            .then_with(|| string_field(left, "name").cmp(&string_field(right, "name")))
    });
    slowest_json_parses.truncate(largest_limit);
    summary.slowest_json_parses = slowest_json_parses;
}

fn string_field<'a>(value: &'a Value, field: &str) -> &'a str {
    value.get(field).and_then(Value::as_str).unwrap_or("")
}
```

- [ ] **Step 2: Add product behavior tests**

Create `experiments/rust-main/lumin-audit-core/tests/artifact_read_metrics.rs`:

```rust
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::artifact_read_metrics::{
    summarize_artifact_read_events, ArtifactReadMetricsRequest,
};

fn request(value: Value) -> anyhow::Result<ArtifactReadMetricsRequest> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn summarizes_successful_and_failed_json_reads_like_js_contract() -> anyhow::Result<()> {
    let request = request(json!({
        "schemaVersion": "lumin-audit-artifact-read-events.v1",
        "rootDir": "C:/repo/.audit",
        "largestLimit": 2,
        "reads": [
            {
                "filePath": "C:/repo/.audit/symbols.json",
                "bytes": 10.4,
                "readMs": 1.4,
                "jsonParseMs": 2.5,
                "ok": true
            },
            {
                "filePath": "C:/repo/.audit/symbols.json",
                "bytes": "9.6",
                "readMs": -3,
                "jsonParseMs": "bad",
                "ok": false
            },
            {
                "filePath": "C:/repo/.audit/triage.json",
                "bytes": 3,
                "readMs": 4,
                "jsonParseMs": 5,
                "ok": true
            }
        ]
    }))?;

    let summary = serde_json::to_value(summarize_artifact_read_events(request)?)?;

    assert_eq!(summary["schemaVersion"], "artifact-read-metrics.v1");
    assert_eq!(summary["measurement"], "audit-repo-orchestrator-json-reads");
    assert_eq!(summary["totalReadCount"], 3);
    assert_eq!(summary["totalReadBytes"], 23);
    assert_eq!(summary["totalReadMs"], 5);
    assert_eq!(summary["totalJsonParseMs"], 8);
    assert_eq!(summary["parseFailureCount"], 1);
    assert_eq!(summary["byName"]["symbols.json"]["readCount"], 2);
    assert_eq!(summary["byName"]["symbols.json"]["totalBytes"], 20);
    assert_eq!(summary["byName"]["symbols.json"]["parseFailureCount"], 1);
    assert_eq!(summary["largestReads"][0]["name"], "symbols.json");
    assert_eq!(summary["slowestJsonParses"][0]["name"], "triage.json");
    Ok(())
}

#[test]
fn normalizes_paths_relative_to_root_and_uses_basename_outside_root() -> anyhow::Result<()> {
    let request = request(json!({
        "schemaVersion": "lumin-audit-artifact-read-events.v1",
        "rootDir": "C:/repo/.audit",
        "reads": [
            {
                "filePath": "C:/repo/.audit/.producer-phases/triage-repo.mjs.json",
                "bytes": 1,
                "readMs": 0,
                "jsonParseMs": 0,
                "ok": true
            },
            {
                "filePath": "D:/elsewhere/symbols.json",
                "bytes": 1,
                "readMs": 0,
                "jsonParseMs": 0,
                "ok": true
            }
        ]
    }))?;

    let summary = serde_json::to_value(summarize_artifact_read_events(request)?)?;

    assert_eq!(
        summary["byName"][".producer-phases/triage-repo.mjs.json"]["readCount"],
        1
    );
    assert_eq!(summary["byName"]["symbols.json"]["readCount"], 1);
    Ok(())
}

#[test]
fn cli_emits_artifact_read_summary_json() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("reads.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-audit-artifact-read-events.v1",
            "rootDir": temp.path().to_string_lossy(),
            "reads": [
                {
                    "filePath": temp.path().join("manifest.json").to_string_lossy(),
                    "bytes": 7,
                    "readMs": 1,
                    "jsonParseMs": 2,
                    "ok": true
                }
            ]
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("artifact-read-metrics-summary")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["schemaVersion"], "artifact-read-metrics.v1");
    assert_eq!(stdout["summary"], Value::Null);
    assert_eq!(stdout["byName"]["manifest.json"]["readCount"], 1);
    Ok(())
}

#[test]
fn rejects_wrong_request_schema_version() -> anyhow::Result<()> {
    let request = request(json!({
        "schemaVersion": "old",
        "reads": []
    }))?;

    let error = summarize_artifact_read_events(request)
        .expect_err("wrong artifact-read schema should hard-stop");

    assert!(error.to_string().contains("unsupported schemaVersion"));
    Ok(())
}
```

- [ ] **Step 3: Run the new Rust behavior tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test artifact_read_metrics
```

Expected after Task 1 implementation: PASS.

---

### Task 2: Wire the Rust CLI Command

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/lib.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli.rs`

- [ ] **Step 1: Export the module without touching unrelated local changes**

In `experiments/rust-main/lumin-audit-core/src/lib.rs`, add:

```rust
pub mod artifact_read_metrics;
```

Keep existing local entries such as:

```rust
pub mod pre_write_routing;
```

- [ ] **Step 2: Add CLI imports**

In `experiments/rust-main/lumin-audit-core/src/cli.rs`, add:

```rust
use lumin_audit_core::artifact_read_metrics::{
    summarize_artifact_read_events, ArtifactReadMetricsRequest,
};
```

- [ ] **Step 3: Add the command to usage and dispatch**

In `USAGE`, add this line near the other artifact commands:

```text
       lumin-audit-core artifact-read-metrics-summary --input <path|->
```

In `run()`, add:

```rust
Some("artifact-read-metrics-summary") => run_artifact_read_metrics_summary(args.collect()),
```

- [ ] **Step 4: Add the command handler**

Add this function near the producer-performance runtime handler:

```rust
fn run_artifact_read_metrics_summary(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("artifact-read-metrics-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("artifact-read-metrics-summary: missing --input <path|->")?;
    let input_json = read_json_input(&input, "artifact-read-metrics-summary")?;
    let request = serde_json::from_value::<ArtifactReadMetricsRequest>(input_json)
        .context("artifact-read-metrics-summary: invalid request shape")?;
    let summary = summarize_artifact_read_events(request)?;
    write_stdout_json(&summary)
}
```

- [ ] **Step 5: Verify CLI tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test artifact_read_metrics
```

Expected: PASS, including `cli_emits_artifact_read_summary_json`.

---

### Task 3: Reuse the Shared Rust Summary in Producer Performance

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_events.rs`
- Modify: `experiments/rust-main/lumin-audit-core/tests/orchestration_events.rs`

- [ ] **Step 1: Import shared artifact-read types and helpers**

In `orchestration_events.rs`, add:

```rust
use crate::artifact_read_metrics::{
    record_artifact_read, ArtifactReadSummary, DEFAULT_LARGEST_READ_LIMIT,
};
```

Remove the local `ArtifactReadSummary` struct and local `ArtifactReadMetricEntry`,
`record_artifact_read`, `artifact_read_entries`, `number_field`,
`refresh_artifact_read_projections`, and `string_field` definitions from
`orchestration_events.rs`.

- [ ] **Step 2: Update phase-sidecar read calls**

Replace both current local calls:

```rust
record_artifact_read(
    artifact_reads,
    output,
    &path,
    raw.len() as u64,
    read_ms,
    elapsed_ms(parse_started),
    true,
);
```

with:

```rust
record_artifact_read(
    artifact_reads,
    Some(output),
    &path,
    raw.len() as u64,
    read_ms,
    elapsed_ms(parse_started),
    true,
    DEFAULT_LARGEST_READ_LIMIT,
);
```

Do the same for the malformed phase sidecar branch with `ok: false`.

- [ ] **Step 3: Run existing producer-performance tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test orchestration_events
```

Expected: PASS. This proves phase sidecar reads still merge into ordinary
artifact-read summaries.

---

### Task 4: Replace JS Summary Math With a Thin Audit-Core Wrapper

**Files:**
- Modify: `_lib/audit-manifest.mjs`
- Create: `_lib/artifact-read-metrics.mjs`
- Modify: `_lib/artifacts.mjs`
- Modify: `audit-repo.mjs`
- Mirror: `skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs`
- Mirror: `skills/lumin-repo-lens-lab/_engine/lib/artifact-read-metrics.mjs`
- Mirror: `skills/lumin-repo-lens-lab/_engine/lib/artifacts.mjs`
- Mirror: `skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`

- [ ] **Step 1: Add the audit-core wrapper export**

In `_lib/audit-manifest.mjs`, add:

```js
export function buildArtifactReadMetricsSummary(input) {
  return runAuditCoreJson([
    'artifact-read-metrics-summary',
    '--input', '-',
  ], 'buildArtifactReadMetricsSummary', {
    input: JSON.stringify(input ?? {}),
  });
}
```

Mirror the same export in
`skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs`.

- [ ] **Step 2: Create the thin JS collector**

Create `_lib/artifact-read-metrics.mjs`:

```js
import { buildArtifactReadMetricsSummary } from './audit-manifest.mjs';

export const ARTIFACT_READ_EVENTS_SCHEMA_VERSION = 'lumin-audit-artifact-read-events.v1';

export function createArtifactReadMetrics({ rootDir, largestLimit = 10 } = {}) {
  const reads = [];

  function observeRead(record) {
    reads.push({
      filePath: record?.filePath ?? 'unknown',
      bytes: record?.bytes ?? 0,
      readMs: record?.readMs ?? 0,
      jsonParseMs: record?.jsonParseMs ?? 0,
      ok: record?.ok !== false,
    });
  }

  function summary() {
    return buildArtifactReadMetricsSummary({
      schemaVersion: ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
      rootDir,
      largestLimit,
      reads,
    });
  }

  return { observeRead, summary };
}
```

Create the same file at
`skills/lumin-repo-lens-lab/_engine/lib/artifact-read-metrics.mjs`.

- [ ] **Step 3: Remove JS summary math from artifacts helper**

In `_lib/artifacts.mjs`, delete:

```js
export const ARTIFACT_READ_METRICS_SCHEMA_VERSION = 'artifact-read-metrics.v1';
export function createArtifactReadMetrics({ rootDir, largestLimit = 10 } = {}) {
  ...
}
```

Keep `loadIfExists()`, `readJsonFile()`, and `producerMetaBase()` unchanged.

Apply the same edit to `skills/lumin-repo-lens-lab/_engine/lib/artifacts.mjs`.

- [ ] **Step 4: Update audit-repo imports**

In `audit-repo.mjs`, change:

```js
import {
  createArtifactReadMetrics,
  loadIfExists as loadArtifact,
} from './_lib/artifacts.mjs';
```

to:

```js
import { createArtifactReadMetrics } from './_lib/artifact-read-metrics.mjs';
import { loadIfExists as loadArtifact } from './_lib/artifacts.mjs';
```

Apply the same edit to
`skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`, adjusting the
relative import paths already used in that mirror file.

- [ ] **Step 5: Search for old JS owner references**

Run:

```powershell
rg -n "createArtifactReadMetrics|ARTIFACT_READ_METRICS_SCHEMA_VERSION" _lib audit-repo.mjs skills\lumin-repo-lens-lab\_engine
```

Expected:

- `createArtifactReadMetrics` is defined only in `artifact-read-metrics.mjs`;
- audit-repo imports it from that file;
- `ARTIFACT_READ_METRICS_SCHEMA_VERSION` no longer appears in JS.

---

### Task 5: Update Canonical Ownership

**Files:**
- Modify: `canonical/audit-core.md`
- Optional if packaging allow-list exists for canonical/spec files: inspect `scripts/build-skill.mjs`

- [ ] **Step 1: Update scope paragraph**

In `canonical/audit-core.md`, change the scope so `lumin-audit-core` owns:

```text
ordinary artifact-read metric summary projection from JS-supplied raw read observations
```

Do not claim Rust owns ordinary JSON reading or parsing.

- [ ] **Step 2: Narrow remaining JS-owned boundary**

Replace the remaining exception:

```text
ordinary artifact-read measurement
```

with:

```text
ordinary artifact JSON read/parse events
```

- [ ] **Step 3: Add module owner row**

Add a canonical module row:

```markdown
| `experiments/rust-main/lumin-audit-core/src/artifact_read_metrics.rs` | `artifact-read-metrics.v1` summary projection from JS-supplied raw read events, metric-name normalization, read counter aggregation, largest/slowest read projections, and shared phase-sidecar read metric updates | ordinary JSON file reading/parsing, parse-failure log text, producer artifact meaning, blind-zone interpretation |
```

- [ ] **Step 4: Update `orchestration_events.rs` owner row**

Change the `orchestration_events.rs` row so it says it owns producer-performance
construction and phase timing reads, but reuses `artifact_read_metrics.rs` for
artifact-read summary math.

- [ ] **Step 5: Verify no stale docs contradict the new owner**

Run:

```powershell
rg -n "ordinary artifact-read measurement|createArtifactReadMetrics|artifact-read-metrics.v1" canonical docs\superpowers\specs docs\superpowers\plans
```

Expected: stale owner text appears only in historical specs/plans or is updated
in current canonical docs.

---

### Task 6: Verification and Commit

**Files:**
- All files changed above

- [ ] **Step 1: Run Rust audit-core tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core
```

Expected: PASS.

- [ ] **Step 2: Run Rust clippy**

Run:

```powershell
cargo clippy --manifest-path experiments\Cargo.toml -p lumin-audit-core --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 3: Run Rust format check**

Run:

```powershell
cargo fmt --manifest-path experiments\Cargo.toml --all -- --check
```

Expected: PASS.

- [ ] **Step 4: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected: no errors. CRLF warnings may appear if Git reports them; do not
rewrite unrelated files to silence them.

- [ ] **Step 5: Confirm JS summary math was removed**

Run:

```powershell
rg -n "totalReadCount|largestReads|slowestJsonParses|parseFailureCount" _lib\artifacts.mjs _lib\artifact-read-metrics.mjs audit-repo.mjs
```

Expected:

- `_lib/artifacts.mjs` has no artifact-read summary math;
- `_lib/artifact-read-metrics.mjs` only stores raw events and calls audit-core;
- `audit-repo.mjs` still observes reads at the same load sites.

- [ ] **Step 6: Commit only owned files**

Before committing, check:

```powershell
git status --short
```

Do not stage unrelated local changes. If `pre_write_routing.rs` or other
pre-existing local files are still unrelated to this task, leave them out.

Commit:

```powershell
git add `
  docs\superpowers\plans\2026-07-01-artifact-read-metrics-audit-core.md `
  experiments\rust-main\lumin-audit-core\src\artifact_read_metrics.rs `
  experiments\rust-main\lumin-audit-core\tests\artifact_read_metrics.rs `
  experiments\rust-main\lumin-audit-core\src\cli.rs `
  experiments\rust-main\lumin-audit-core\src\orchestration_events.rs `
  experiments\rust-main\lumin-audit-core\src\lib.rs `
  _lib\audit-manifest.mjs `
  _lib\artifact-read-metrics.mjs `
  _lib\artifacts.mjs `
  audit-repo.mjs `
  skills\lumin-repo-lens-lab\_engine\lib\audit-manifest.mjs `
  skills\lumin-repo-lens-lab\_engine\lib\artifact-read-metrics.mjs `
  skills\lumin-repo-lens-lab\_engine\lib\artifacts.mjs `
  skills\lumin-repo-lens-lab\_engine\producers\audit-repo.mjs `
  canonical\audit-core.md
git commit -m "Move artifact read metrics into audit core"
```

If `lib.rs` contains unrelated pre-existing changes, inspect the staged diff
before committing and confirm the only `lib.rs` line added for this task is
`pub mod artifact_read_metrics;`.

---

## Plan Self-Review

- Spec coverage: Tasks 1-4 cover typed request shape, path normalization,
  deterministic summaries, JS wrapper thinning, and producer-performance API
  preservation. Task 5 covers canonical ownership. Task 6 covers verification.
- Scope: The plan intentionally keeps ordinary JSON read/parse behavior in JS
  and does not touch `blindZones`.
- Dirty worktree safety: The plan explicitly preserves unrelated
  `pre_write_routing` work.
- Placeholder scan: no TBD/TODO/fill-in placeholders remain.
