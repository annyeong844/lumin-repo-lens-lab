# Rust Rank Fixes Producer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `fix-plan.json` construction and four-tier fix ranking from `rank-fixes.mjs` / `_lib/ranking.mjs` into `lumin-audit-core` while keeping JS package-export policy as a wrapper-supplied fact.

**Architecture:** Add a Rust `rank_fixes` artifact producer that consumes already-produced JSON artifacts plus `publicDeepImportRiskByFile`, computes finding evidence/tier projection, and returns the existing `fix-plan.json` shape. Keep `rank-fixes.mjs` as a compatibility wrapper that loads artifacts, computes a shallow public-risk file map using current JS package helpers, calls `lumin-audit-core rank-fixes-artifact`, writes `fix-plan.json`, and prints the existing summary.

**Tech Stack:** Rust (`serde`, `serde_json`, `anyhow`, `BTreeMap`/`BTreeSet` for deterministic projection), Node ESM wrapper, existing `lumin-audit-core` result-file bridge, focused Cargo and Node compatibility checks.

---

## File Structure

- Create: `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs`
  - Owns request validation, finding flattening, tier predicate, support evidence merge, deterministic `fix-plan.json` artifact projection.
- Create: `experiments/rust-main/lumin-audit-core/src/cli/rank_fixes.rs`
  - Owns CLI argument parsing for `rank-fixes-artifact` and result-file/stdout writes.
- Create: `experiments/rust-main/lumin-audit-core/tests/rank_fixes.rs`
  - Focused Rust product behavior and CLI tests.
- Modify: `experiments/rust-main/lumin-audit-core/src/lib.rs`
  - Export `rank_fixes`.
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/mod.rs`
  - Dispatch `rank-fixes-artifact`.
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/usage.rs`
  - Document CLI usage.
- Modify: `_lib/audit-core.mjs`
  - Add `rank-fixes-artifact` to audit-core contract probes and result-file requirements.
- Modify: `rank-fixes.mjs`
  - Replace ranking/projection logic with wrapper request construction and Rust call.
- Modify: `canonical/audit-core.md`
  - Register `rank_fixes.rs` as `fix-plan.json` owner.
- Modify: `scripts/build-skill.mjs` and generated skill package mirrors only if required by the existing package source-copy lists.

## Task 1: Wire The Rust Producer Surface

**Files:**
- Create: `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs`
- Create: `experiments/rust-main/lumin-audit-core/src/cli/rank_fixes.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/lib.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/mod.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/usage.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/rank_fixes.rs`

- [ ] **Step 1: Add the public Rust request/artifact skeleton**

Add `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs` with these public constants and shapes:

```rust
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const RANK_FIXES_REQUEST_SCHEMA_VERSION: &str =
    "lumin-rank-fixes-producer-request.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesRequest {
    pub schema_version: String,
    pub root: String,
    pub generated: String,
    pub artifacts: RankFixesArtifacts,
    #[serde(default)]
    pub public_deep_import_risk_by_file: BTreeMap<String, PublicDeepImportRisk>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesArtifacts {
    pub dead_classify: Value,
    #[serde(default)]
    pub runtime_evidence: Option<Value>,
    #[serde(default)]
    pub staleness: Option<Value>,
    #[serde(default)]
    pub symbols: Option<Value>,
    #[serde(default)]
    pub export_action_safety: Option<Value>,
    #[serde(default)]
    pub call_graph: Option<Value>,
    #[serde(default)]
    pub entry_surface: Option<Value>,
    #[serde(default)]
    pub module_reachability: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDeepImportRisk {
    #[serde(default)]
    pub risk: Option<bool>,
    #[serde(flatten)]
    pub detail: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesArtifact {
    pub meta: Value,
    pub summary: Value,
    pub safe_fixes: Vec<Value>,
    pub safe_fix_groups: Vec<Value>,
    pub review_fixes: Vec<Value>,
    pub degraded: Vec<Value>,
    pub muted: Vec<Value>,
}

pub fn build_rank_fixes_artifact(request: RankFixesRequest) -> Result<RankFixesArtifact> {
    if request.schema_version != RANK_FIXES_REQUEST_SCHEMA_VERSION {
        bail!(
            "rank-fixes-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.public_deep_import_risk_by_file.is_empty() {
        bail!("rank-fixes-artifact: missing publicDeepImportRiskByFile");
    }
    let inputs = serde_json::json!({
        "dead-classify.json": true,
        "runtime-evidence.json": request.artifacts.runtime_evidence.is_some(),
        "staleness.json": request.artifacts.staleness.is_some(),
        "symbols.json": request.artifacts.symbols.is_some(),
        "export-action-safety.json": request.artifacts.export_action_safety.is_some(),
        "call-graph.json": request.artifacts.call_graph.is_some(),
        "entry-surface.json": request.artifacts.entry_surface.is_some(),
        "module-reachability.json": request.artifacts.module_reachability.is_some()
    });
    Ok(RankFixesArtifact {
        meta: serde_json::json!({
            "generated": request.generated,
            "root": request.root,
            "tool": "rank-fixes.mjs",
            "executionOwner": "lumin-audit-core",
            "inputs": inputs,
            "resolverBlindness": Value::Null,
            "topUnresolvedSpecifiers": []
        }),
        summary: serde_json::json!({
            "SAFE_FIX": 0,
            "REVIEW_FIX": 0,
            "DEGRADED": 0,
            "MUTED": 0,
            "total": 0,
            "safeFixGroups": 0
        }),
        safe_fixes: Vec::new(),
        safe_fix_groups: Vec::new(),
        review_fixes: Vec::new(),
        degraded: Vec::new(),
        muted: Vec::new(),
    })
}
```

- [ ] **Step 2: Add the CLI module**

Create `experiments/rust-main/lumin-audit-core/src/cli/rank_fixes.rs`:

```rust
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::rank_fixes::{build_rank_fixes_artifact, RankFixesRequest};

pub(super) fn run_rank_fixes_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output: Option<PathBuf> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("rank-fixes-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("rank-fixes-artifact: missing --input <path|->")?;
    let json = read_json_input(&input, "rank-fixes-artifact")?;
    let request = serde_json::from_value::<RankFixesRequest>(json)
        .context("rank-fixes-artifact: invalid request shape")?;
    let artifact = build_rank_fixes_artifact(request)?;
    if let Some(path) = result_output {
        write_json_file(&path, &artifact)
    } else {
        write_stdout_json(&artifact)
    }
}
```

- [ ] **Step 3: Wire library and CLI dispatch**

Add to `experiments/rust-main/lumin-audit-core/src/lib.rs`:

```rust
pub mod rank_fixes;
```

Add to `experiments/rust-main/lumin-audit-core/src/cli/mod.rs`:

```rust
mod rank_fixes;
use rank_fixes::*;
```

Add the dispatch arm:

```rust
Some("rank-fixes-artifact") => run_rank_fixes_artifact(args.collect()),
```

Add to `experiments/rust-main/lumin-audit-core/src/cli/usage.rs`:

```text
       lumin-audit-core rank-fixes-artifact --input <path|-> [--result-output <path>]
```

- [ ] **Step 4: Add the first CLI contract test**

Create `experiments/rust-main/lumin-audit-core/tests/rank_fixes.rs`:

```rust
use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

#[test]
fn cli_rank_fixes_artifact_rejects_missing_input() -> Result<()> {
    let output = Command::new(audit_core_bin())
        .arg("rank-fixes-artifact")
        .output()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.status.success());
    assert!(text.contains("rank-fixes-artifact: missing --input <path|->"), "{text}");
    Ok(())
}

#[test]
fn cli_rank_fixes_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-rank-fixes-producer-request.v1",
            "root": "C:/repo",
            "generated": "2026-07-03T00:00:00.000Z",
            "artifacts": {
                "deadClassify": {
                    "proposal_C_remove_symbol": [],
                    "proposal_A_demote_to_internal": [],
                    "proposal_B_review": [],
                    "proposal_remove_export_specifier": [],
                    "proposal_DEGRADED_unprocessed": [],
                    "excludedCandidates": []
                }
            },
            "publicDeepImportRiskByFile": { "__sentinel__": { "risk": false } }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("rank-fixes-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stdout.is_empty());
    let artifact: serde_json::Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(artifact["meta"]["tool"], "rank-fixes.mjs");
    assert_eq!(artifact["summary"]["total"], 0);
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
```

- [ ] **Step 5: Verify Task 1**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test rank_fixes
```

Expected: `cli_rank_fixes_artifact_rejects_missing_input` passes, and the result-file test passes.

## Task 2: Port Finding Flattening, Identity, And Action Evidence

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs`
- Modify: `experiments/rust-main/lumin-audit-core/tests/rank_fixes.rs`

- [ ] **Step 1: Add identity and flattening test cases**

Append tests covering:

```rust
#[test]
fn rank_fixes_materializes_safe_review_degraded_and_muted_findings() -> Result<()> {
    let artifact = build_rank_fixes_artifact(rank_request(json!({
        "proposal_C_remove_symbol": [
            { "file": "src/safe.ts", "line": 1, "symbol": "Safe", "kind": "FunctionDeclaration", "action": "" }
        ],
        "proposal_A_demote_to_internal": [],
        "proposal_B_review": [
            { "file": "src/review.ts", "line": 2, "symbol": "Review", "kind": "FunctionDeclaration", "action": "" }
        ],
        "proposal_remove_export_specifier": [],
        "proposal_DEGRADED_unprocessed": [
            { "file": "src/bounded.ts", "line": 3, "symbol": "Bounded", "kind": "FunctionDeclaration", "action": "classification incomplete" }
        ],
        "excludedCandidates": [
            { "file": "src/public.ts", "line": 4, "symbol": "Public", "kind": "FunctionDeclaration", "reason": "publicApi_FP23" }
        ]
    }), Some(json!({
        "findings": [
            {
                "id": "dead-export:src/safe.ts:Safe:1",
                "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                "actionBlockers": []
            },
            {
                "id": "dead-export:src/review.ts:Review:2",
                "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                "actionBlockers": []
            }
        ]
    })), None, None))?;
    assert_eq!(artifact.summary["SAFE_FIX"], 1);
    assert_eq!(artifact.summary["REVIEW_FIX"], 1);
    assert_eq!(artifact.summary["DEGRADED"], 1);
    assert_eq!(artifact.summary["MUTED"], 1);
    assert_eq!(artifact.safe_fixes[0]["finding"]["id"], "dead-export:src/safe.ts:Safe:1");
    assert_eq!(artifact.muted[0]["tier"], "MUTED");
    Ok(())
}
```

Add the helper in the test file:

```rust
fn rank_request(
    dead_classify: serde_json::Value,
    export_action_safety: Option<serde_json::Value>,
    symbols: Option<serde_json::Value>,
    extra_public_risk: Option<serde_json::Value>,
) -> lumin_audit_core::rank_fixes::RankFixesRequest {
    let mut public_deep_import_risk_by_file = serde_json::Map::new();
    public_deep_import_risk_by_file.insert("__sentinel__".to_string(), json!({ "risk": false }));
    for file in [
        "src/safe.ts",
        "src/review.ts",
        "src/bounded.ts",
        "src/public.ts",
    ] {
        public_deep_import_risk_by_file.insert(file.to_string(), json!({ "risk": false }));
    }
    if let Some(serde_json::Value::Object(extra)) = extra_public_risk {
        public_deep_import_risk_by_file.extend(extra);
    }
    serde_json::from_value(json!({
        "schemaVersion": "lumin-rank-fixes-producer-request.v1",
        "root": "C:/repo",
        "generated": "2026-07-03T00:00:00.000Z",
        "artifacts": {
            "deadClassify": dead_classify,
            "exportActionSafety": export_action_safety,
            "symbols": symbols
        },
        "publicDeepImportRiskByFile": public_deep_import_risk_by_file
    })).expect("valid request")
}
```

- [ ] **Step 2: Implement canonical finding identity and flattening**

Add internal helpers in `rank_fixes.rs`:

```rust
#[derive(Debug, Clone)]
struct FindingRecord {
    value: Value,
    id: String,
    key: String,
    file: String,
    symbol: String,
    line: Value,
    bucket: String,
    excluded_reason: Option<String>,
}

fn normalize_path(value: &Value) -> String {
    value.as_str().unwrap_or_default().replace('\\', "/").trim_start_matches("./").to_string()
}

fn line_key(value: &Value) -> String {
    match value {
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn finding_id(file: &str, symbol: &str, line: &Value) -> String {
    format!("dead-export:{file}:{symbol}:{}", line_key(line))
}

fn lookup_key(file: &str, symbol: &str, line: &Value) -> String {
    format!("{file}|{symbol}|{}", line_key(line))
}
```

Implement `flatten_bucket(dead_classify, field, bucket)` and `flatten_excluded(dead_classify)` by cloning each proposal object to `Value::Object`, adding:

```rust
object.insert("id".to_string(), Value::String(id.clone()));
object.insert("bucket".to_string(), Value::String(bucket.to_string()));
```

For excluded records, set:

```rust
object.insert("bucket".to_string(), Value::String("excluded".to_string()));
object.insert(
    "action".to_string(),
    Value::String(format!("Policy-excluded: {}", reason)),
);
object.insert("_excludeReason".to_string(), Value::String(reason));
```

- [ ] **Step 3: Implement action-safety merge**

Build `actionById` from either `exportActionSafety.byId` or `exportActionSafety.findings[]`:

```rust
fn action_by_id(export_action_safety: Option<&Value>) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    if let Some(by_id) = export_action_safety.and_then(|value| value.get("byId")).and_then(Value::as_object) {
        for (id, record) in by_id {
            map.insert(id.clone(), record.clone());
        }
    }
    if let Some(records) = export_action_safety
        .and_then(|value| value.get("findings"))
        .and_then(Value::as_array)
    {
        for record in records {
            if let Some(id) = record.get("id").and_then(Value::as_str) {
                map.insert(id.to_string(), record.clone());
            }
        }
    }
    map
}
```

When flattening ordinary findings, if an action record exists, copy `safeAction`,
`actionBlockers`, and `localUseProof` into `finding`.

- [ ] **Step 4: Verify Task 2**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test rank_fixes
```

Expected: Task 1 and Task 2 tests pass.

## Task 3: Port The Four-Tier Predicate

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs`
- Modify: `experiments/rust-main/lumin-audit-core/tests/rank_fixes.rs`

- [ ] **Step 1: Add direct predicate parity tests**

Add tests for the checked tier order:

```rust
#[test]
fn rank_predicate_blocks_safe_fix_for_soft_taint_and_unknown_public_risk() -> Result<()> {
    let artifact = build_rank_fixes_artifact(rank_request(json!({
        "proposal_C_remove_symbol": [
            {
                "file": "src/safe.ts",
                "line": 1,
                "symbol": "Safe",
                "kind": "FunctionDeclaration",
                "action": "",
                "taintedBy": [{ "kind": "parse-errors-elsewhere", "file": "src/other.ts" }]
            },
            { "file": "src/missing-risk.ts", "line": 2, "symbol": "UnknownRisk", "kind": "FunctionDeclaration", "action": "" }
        ],
        "proposal_A_demote_to_internal": [],
        "proposal_B_review": [],
        "proposal_remove_export_specifier": [],
        "proposal_DEGRADED_unprocessed": [],
        "excludedCandidates": []
    }), Some(json!({
        "findings": [
            {
                "id": "dead-export:src/safe.ts:Safe:1",
                "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                "actionBlockers": []
            },
            {
                "id": "dead-export:src/missing-risk.ts:UnknownRisk:2",
                "safeAction": { "kind": "demote_export_declaration", "proofComplete": true, "actionBlockers": [] },
                "actionBlockers": []
            }
        ]
    })), None, None))?;
    assert_eq!(artifact.summary["SAFE_FIX"], 0);
    assert_eq!(artifact.summary["REVIEW_FIX"], 2);
    assert!(
        artifact.review_fixes.iter().any(|entry| entry["reason"].as_str().unwrap_or("").contains("parse-errors-elsewhere"))
    );
    assert!(
        artifact.review_fixes.iter().any(|entry| entry["reason"].as_str().unwrap_or("").contains("public-deep-import-risk"))
    );
    Ok(())
}
```

- [ ] **Step 2: Implement `TierResult` and evidence extraction**

In `rank_fixes.rs`, add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier {
    SafeFix,
    ReviewFix,
    Degraded,
    Muted,
}

impl Tier {
    fn as_str(self) -> &'static str {
        match self {
            Tier::SafeFix => "SAFE_FIX",
            Tier::ReviewFix => "REVIEW_FIX",
            Tier::Degraded => "DEGRADED",
            Tier::Muted => "MUTED",
        }
    }
}

#[derive(Debug, Default)]
struct Evidence {
    runtime: Option<Value>,
    staleness: Option<Value>,
    resolver: Option<Value>,
    contract: Value,
    entry_surface: Value,
    policy: Value,
}

struct TierResult {
    tier: Tier,
    reason: String,
    confidence: Option<&'static str>,
    confidence_detail: Option<&'static str>,
    blocked_promotion: bool,
    blocked_by: Vec<Value>,
}
```

Add helper functions mirroring `_lib/ranking.mjs` names, with these concrete
outputs:

- `policy_exclusion_result(policy)` returns `MUTED` with
  `policy-excluded: <reason|unknown>` when `policy.excluded === true`.
- `runtime_contradiction_result(runtime)` returns `DEGRADED` with
  `runtime-executed (<hits> hits)` when `runtime.status === "executed"`.
- `blocking_taint_result(finding)` returns `DEGRADED` for
  `unresolved-spec-could-match` and defining-file parse-error blocking taints.
- `missing_safe_action_result(finding)` returns `REVIEW_FIX` with
  `action-blockers: <joined actionBlockers>` when selected `actionBlockers[]` is non-empty and
  `REVIEW_FIX` with `missing-safe-action-proof` when proof is absent.
- `declaration_dependency_result(finding)` returns `REVIEW_FIX` when a
  declaration export dependency would not be preserved by the selected action.
- `bucket_b_result(finding)` returns `REVIEW_FIX` for ordinary `B` bucket
  findings.
- `html_entrypoint_blind_zone_result(entrySurface)` returns `REVIEW_FIX` with
  `blockedPromotion: true` and the existing `blockedBy` object.
- `public_deep_import_risk_result(contract)` returns `REVIEW_FIX` for
  `risk === true` and for missing per-file risk facts.
- `safe_fix_result(finding, runtime, staleness, support)` returns `SAFE_FIX`
  only after all previous blockers are absent.
- `weaker_evidence_review_result(finding, runtime, taints)` returns
  `REVIEW_FIX` with structured `blockedBy` when soft taints or weak runtime
  status prevent `SAFE_FIX`.

Keep `actionBlockers` as `REVIEW_FIX`, not `DEGRADED`.

- [ ] **Step 3: Preserve structured soft-taint blockers**

Implement generated-artifact and resolver blind-zone blocker projection as
`Vec<Value>` by copying the checked fields from each taint object:

```rust
const GENERATED_ARTIFACT_MISSING_RELEVANT: &str = "generated-artifact-missing-relevant";
const RESOLVER_BLIND_ZONE_RELEVANT: &str = "resolver-blind-zone-relevant";
```

For generated artifact taints, default `reason` to
`"workspace-generated-artifact-missing"` when the taint lacks `reason`, matching
JS `GENERATED_ARTIFACT_MISSING_REASON`.

- [ ] **Step 4: Verify Task 3**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test rank_fixes
```

Expected: tier predicate tests pass.

## Task 4: Port Support Evidence And Summary Projection

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs`
- Modify: `experiments/rust-main/lumin-audit-core/tests/rank_fixes.rs`

- [ ] **Step 1: Add support evidence tests**

Add tests for:

```rust
#[test]
fn entry_unreachable_support_requires_complete_unbounded_private_file() -> Result<()> {
    let artifact = build_rank_fixes_artifact(rank_request_with_support_artifacts())?;
    assert_eq!(artifact.summary["SAFE_FIX"], 1);
    let safe = &artifact.safe_fixes[0];
    assert_eq!(safe["confidence"], "medium");
    assert_eq!(safe["confidenceDetail"], "medium_with_evidence");
    assert_eq!(safe["finding"]["supportedBy"][0]["kind"], "entry-unreachable");
    Ok(())
}

#[test]
fn call_graph_support_requires_bounded_member_stats() -> Result<()> {
    let artifact = build_rank_fixes_artifact(rank_request_with_call_graph_without_bounded_stats())?;
    assert_eq!(artifact.summary["SAFE_FIX"], 1);
    assert!(artifact.safe_fixes[0]["finding"]["supportedBy"].as_array().map_or(true, |items| {
        !items.iter().any(|item| item["kind"] == "call-graph-no-observed-callers")
    }));
    Ok(())
}
```

Implement helpers in the test file using minimal JSON artifacts:

```rust
fn module_reachability_for_unreachable(file: &str) -> serde_json::Value {
    json!({
        "meta": {
            "completenessBySubmodule": { ".": "high" },
            "supports": {
                "runtimeReachableFiles": true,
                "typeReachableFiles": true,
                "boundedOutFiles": true
            }
        },
        "runtimeReachableFiles": [],
        "typeReachableFiles": [],
        "boundedOutFiles": [],
        "unreachableFiles": [file]
    })
}
```

- [ ] **Step 2: Implement support evidence helpers**

Add:

```rust
fn with_evidence_support(finding: Value, request: &RankFixesRequest) -> Value
fn entry_unreachable_support(finding: &Value, request: &RankFixesRequest) -> Option<Value>
fn call_graph_no_observed_callers_support(finding: &Value, request: &RankFixesRequest) -> Option<Value>
fn html_entry_surface_blind_zone_for_file(file: &str, entry_surface: Option<&Value>) -> Option<Value>
```

Rules:

- Use normalized slash paths.
- Treat missing support flags as absence.
- Use strict `< 0.10` for bounded-out ratio.
- Do not add `entry-unreachable` when the file is bounded out, reachable,
  an entry file, dynamically opaque, or public-risk unknown/true.
- Do not add call graph support without bounded member-call stats.

- [ ] **Step 3: Implement summary and safeFixGroups**

Collect scored entries into `BTreeMap<Tier, Vec<Value>>`. Each scored entry must
have:

```json
{
  "finding": {},
  "evidence": {},
  "tier": "SAFE_FIX",
  "reason": "safe-action + static-graph-clean + bucket-C + no-runtime + no-staleness"
}
```

Add optional `confidence`, `confidenceDetail`, `blockedPromotion`, and
`blockedBy` only when present.

Sort each tier list by:

```rust
fn sort_key(score: &Value) -> (String, i64, String)
```

Build `safeFixGroups` from `safeFixes` only. Group key is
`file|safeAction.kind`, with fields:

```json
{
  "file": "src/foo.ts",
  "actionKind": "demote_export_declaration",
  "count": 2,
  "symbols": ["A", "B"],
  "lines": [1, 2]
}
```

Sort groups by descending `count`, then `file`, then `actionKind`.

- [ ] **Step 4: Verify Task 4**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test rank_fixes
```

Expected: support evidence and summary projection tests pass.

## Task 5: Convert `rank-fixes.mjs` To A Thin Wrapper

**Files:**
- Modify: `rank-fixes.mjs`
- Modify: `_lib/audit-core.mjs`
- Test: `tests/test-rank-fixes.mjs`

- [ ] **Step 1: Add audit-core bridge exports if missing**

If `_lib/audit-core.mjs` does not already export a generic result-file runner,
add:

```js
export function runAuditCoreJsonResultFile(args, label, options = {}) {
  const command = auditCoreBinary();
  const tempDir = mkdtempSync(path.join(tmpdir(), 'lumin-audit-core-result-'));
  const resultPath = path.join(tempDir, 'result.json');
  try {
    execFileSync(command, [...args, '--result-output', resultPath], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
      ...options,
    });
    return JSON.parse(readFileSync(resultPath, 'utf8'));
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}
```

If the export already exists, reuse it and do not duplicate the helper.

- [ ] **Step 2: Add contract probes**

In `_lib/audit-core.mjs`, add to `AUDIT_CORE_CONTRACT_PROBES`:

```js
[
  ['rank-fixes-artifact'],
  'rank-fixes-artifact: missing --input <path|->',
],
```

Add to `RESULT_FILE_REQUIRED_SUBCOMMANDS`:

```js
'rank-fixes-artifact',
```

If the result-file contract probe has a table of minimal valid fixtures, add a
`rank-fixes-artifact` fixture that asserts the result has `meta`, `summary`,
`safeFixes`, `reviewFixes`, `degraded`, and `muted`.

- [ ] **Step 3: Replace ranking logic in `rank-fixes.mjs`**

Keep only:

```js
#!/usr/bin/env node
import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { parseCliArgs } from './_lib/cli.mjs';
import { loadIfExists as loadArtifact } from './_lib/artifacts.mjs';
import { runAuditCoreJsonResultFile } from './_lib/audit-core.mjs';
import { findNearestPackageInfo, getPublicDeepImportRisk } from './_lib/package-exports.mjs';

const { root, output } = parseCliArgs();
const ROOT = path.resolve(root);
const OUT = path.resolve(output);
const loadIfExists = (name) => loadArtifact(OUT, name, { tag: 'rank-fixes' });

const deadClassify = loadIfExists('dead-classify.json');
if (!deadClassify) {
  console.error('[rank-fixes] dead-classify.json is required. Run classify-dead-exports.mjs first.');
  process.exit(1);
}

const artifacts = {
  deadClassify,
  runtimeEvidence: loadIfExists('runtime-evidence.json'),
  staleness: loadIfExists('staleness.json'),
  symbols: loadIfExists('symbols.json'),
  exportActionSafety: loadIfExists('export-action-safety.json'),
  callGraph: loadIfExists('call-graph.json'),
  entrySurface: loadIfExists('entry-surface.json'),
  moduleReachability: loadIfExists('module-reachability.json'),
};
```

Add shallow file collection:

```js
function collectDeadClassifyFiles(deadClassify) {
  const files = new Set(['__sentinel__']);
  const fields = [
    'proposal_C_remove_symbol',
    'proposal_A_demote_to_internal',
    'proposal_B_review',
    'proposal_remove_export_specifier',
    'proposal_DEGRADED_unprocessed',
    'excludedCandidates',
  ];
  for (const field of fields) {
    for (const item of deadClassify?.[field] ?? []) {
      if (typeof item?.file === 'string' && item.file.length > 0) files.add(item.file);
    }
  }
  return [...files].sort();
}

function publicDeepImportRiskByFile(files) {
  const result = {};
  for (const file of files) {
    if (file === '__sentinel__') {
      result[file] = { risk: false, reason: 'sentinel' };
      continue;
    }
    const packageInfo = findNearestPackageInfo(ROOT, file);
    result[file] = packageInfo?.packageJson
      ? getPublicDeepImportRisk(packageInfo.packageJson, packageInfo.relFileFromPkgRoot)
      : { risk: null, reason: 'package-json-absent', relFileFromPkgRoot: file };
  }
  return result;
}
```

Build request and call Rust:

```js
const artifact = runAuditCoreJsonResultFile([
  'rank-fixes-artifact',
  '--input',
  '-',
], 'rank-fixes', {
  input: JSON.stringify({
    schemaVersion: 'lumin-rank-fixes-producer-request.v1',
    root: ROOT,
    generated: new Date().toISOString(),
    artifacts,
    publicDeepImportRiskByFile: publicDeepImportRiskByFile(collectDeadClassifyFiles(deadClassify)),
  }),
});

const outPath = path.join(OUT, 'fix-plan.json');
writeFileSync(outPath, JSON.stringify(artifact, null, 2));
```

Keep the existing console summary output shape:

```js
console.log('\n══════ fix-plan ranking ══════');
console.log(`  SAFE_FIX    : ${artifact.summary.SAFE_FIX}  (auto-fix candidates)`);
console.log(`  REVIEW_FIX  : ${artifact.summary.REVIEW_FIX}  (human review recommended)`);
console.log(`  DEGRADED    : ${artifact.summary.DEGRADED}  (evidence insufficient — not a hard warning)`);
console.log(`  MUTED       : ${artifact.summary.MUTED}  (policy-excluded — not a finding)`);
console.log(`  total       : ${artifact.summary.total}`);
console.log(`\n[rank-fixes] saved → ${outPath}`);
```

- [ ] **Step 4: Verify Task 5**

Run:

```powershell
node tests/test-rank-fixes.mjs
```

Expected: focused Node rank-fixes compatibility suite passes.

## Task 6: Canonical, Packaging, And Skill Surface Updates

**Files:**
- Modify: `canonical/audit-core.md`
- Modify: `scripts/build-skill.mjs` if package source lists need the new module
- Modify: `skills/lumin-repo-lens-lab/_engine/rust` only through the established package build process if this repo expects generated package diffs in source
- Modify: `docs/lumin-wiki/log.md` only if wiki pages are changed

- [ ] **Step 1: Register Rust owner**

In `canonical/audit-core.md`, add `fix-plan.json` ownership to the scope paragraph:

```text
`fix-plan.json` artifact construction from already-produced dead-classify,
runtime, staleness, symbol, action-safety, call-graph, entry-surface, module
reachability, and JS-supplied public-contract facts,
```

Add row to Canonical Rust Modules:

```md
| `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs` | `fix-plan.json` artifact construction from already-produced dead-classify/action-safety/runtime/staleness/symbol/call-graph/entry-surface/module-reachability artifacts plus JS-supplied public deep-import risk facts: finding flattening, tier predicate, support evidence merge, deterministic summary projection, and safe-fix grouping | JS/TS source parsing, dead-export classification, edit-action proof, package export/public-surface interpretation, SARIF emission |
```

- [ ] **Step 2: Update package source lists only if needed**

Run:

```powershell
rg "rust-main.*lumin-audit-core|rank_fixes|unused_deps|module_reachability" scripts build-skill.mjs skills -n
```

If `scripts/build-skill.mjs` explicitly lists Rust source files, add
`rank_fixes.rs` and `cli/rank_fixes.rs`. If it copies directories recursively,
do not add special cases.

- [ ] **Step 3: Verify packaged fallback workspace**

Run:

```powershell
cargo check --manifest-path skills/lumin-repo-lens-lab/_engine/rust/Cargo.toml --locked -p lumin-audit-core
```

Expected: packaged fallback workspace checks. If generated package source is stale, run the established package build command once and include the generated Rust source mirror changes.

## Task 7: Final Focused Verification And Commit

**Files:**
- All files changed by Tasks 1-6

- [ ] **Step 1: Run focused Rust checks**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test rank_fixes
cargo clippy --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --all-targets -- -D warnings
```

Expected: both pass.

- [ ] **Step 2: Run focused JS compatibility check**

Run:

```powershell
node tests/test-rank-fixes.mjs
```

Expected: focused rank-fixes compatibility passes. Do not run the full Node umbrella unless explicitly requested.

- [ ] **Step 3: Run package fallback check**

Run:

```powershell
cargo check --manifest-path skills/lumin-repo-lens-lab/_engine/rust/Cargo.toml --locked -p lumin-audit-core
```

Expected: packaged fallback compiles.

- [ ] **Step 4: Run diff hygiene**

Run:

```powershell
git diff --check
git status -sb
```

Expected: no whitespace errors. Status lists only intentional changed files.

- [ ] **Step 5: Commit**

Run:

```powershell
git add experiments/rust-main/lumin-audit-core/src/rank_fixes.rs `
        experiments/rust-main/lumin-audit-core/src/cli/rank_fixes.rs `
        experiments/rust-main/lumin-audit-core/src/lib.rs `
        experiments/rust-main/lumin-audit-core/src/cli/mod.rs `
        experiments/rust-main/lumin-audit-core/src/cli/usage.rs `
        experiments/rust-main/lumin-audit-core/tests/rank_fixes.rs `
        _lib/audit-core.mjs `
        rank-fixes.mjs `
        canonical/audit-core.md `
        scripts/build-skill.mjs `
        skills/lumin-repo-lens-lab/_engine/rust
git commit -m "Migrate rank fixes projection to audit-core"
```

If `scripts/build-skill.mjs` or generated skill files are unchanged, omit them
from `git add`.

## Self-Review Checklist

- Spec coverage:
  - Request schema and CLI: Task 1.
  - JS wrapper boundary and public-risk shallow superset: Task 5.
  - Finding identity, duplicate, and excluded precedence: Task 2.
  - Tier predicate and absence semantics: Task 3.
  - Support evidence, HTML blind zone, call graph, safeFixGroups: Task 4.
  - Canonical/package docs: Task 6.
  - Focused verification only: Task 7.
- No full Node suite is required by this plan.
- No Rust JS/TS parser, package export resolver, dead-classify migration, edit safety migration, or SARIF migration is introduced.
