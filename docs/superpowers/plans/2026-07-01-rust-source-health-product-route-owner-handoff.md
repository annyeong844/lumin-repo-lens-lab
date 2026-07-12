# Rust Source-Health Product Route Owner Handoff Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the unified Rust analyzer's Rust syntax/product path from the full in-memory JS-shaped source-health lane to the compact Rust source-health owner, without losing product-visible evidence.

**Architecture:** `rust-source-health` remains the owner of Rust syntax facts, clone groups, unused-definition evidence, compact cache, and compact projections. `lumin-rust-analyzer` consumes a typed syntax phase abstraction that can read either the existing full `HealthResponse` or the compact `CompactAnalysisResponse`; the product artifact then projects the same Rust evidence without serializing raw AST/signals. The first implementation keeps a full diagnostic mode, but the product route moves to compact by default once field parity is verified.

**Tech Stack:** Rust 2021, Cargo workspace under `experiments/`, `serde`, project-owned Rust protocol types, `lumin-rust-source-health`, `lumin-rust-analyzer`.

---

## File Structure

- Modify: `canonical/rust-source-health.md`
  - Add the compact analyzer handoff rule: unified analyzer product mode consumes compact Rust source-health evidence; full mode remains diagnostic only.
- Modify: `experiments/rust-sidecar/rust-source-health/src/driver/analysis.rs`
  - Make `CompactAnalysisResponse` a public library response type or move it to protocol ownership before exposing it.
- Modify: `experiments/rust-sidecar/rust-source-health/src/wrapper/request.rs`
  - Export `analyze_root_compact(...)` as a public library API.
- Modify: `experiments/rust-sidecar/rust-source-health/src/wrapper.rs`
  - Re-export `analyze_root_compact`.
- Modify: `experiments/rust-sidecar/rust-source-health/src/lib.rs`
  - Re-export the compact response API.
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/compact.rs`
  - Add compact signal by-kind, muted-by-reason, and capped signal examples if analyzer product projection still needs those fields.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/cli.rs`
  - Add typed syntax/source-health profile options.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/cli/analyze.rs`
  - Parse `--source-health-profile compact|full`, `--cache-root`, `--no-incremental`, and `--clear-incremental-cache`.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/cli/usage.rs`
  - Document the new flags.
- Create: `experiments/rust-main/lumin-rust-analyzer/src/syntax_phase.rs`
  - Own the typed adapter between full `HealthResponse` and compact `CompactAnalysisResponse`.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/main.rs`
  - Run `analyze_root_compact(...)` for compact product mode and `analyze_root(...)` only for explicit full diagnostic mode.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_artifact/build.rs`
  - Accept the syntax phase adapter instead of directly taking `&HealthResponse`.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_artifact/phases/syntax.rs`
  - Build syntax phase brief from the adapter.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_artifact/phases/syntax/summary.rs`
  - Read clone generation policy/summary from compact or full source-health response.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_files/merge.rs`
  - Iterate syntax files through the adapter.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/policy/syntax.rs`
  - Add product projection from `CompactFileHealth` while preserving product-visible counts and capped examples.
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_summary/syntax.rs`
  - Read summary fields from the adapter.
- Modify: `experiments/rust-main/lumin-rust-analyzer/tests/integration/*.rs` and support modules as needed
  - Add product behavior checks for compact default, full diagnostic mode, and incremental visibility.

---

### Task 1: Canonical Owner Update

**Files:**
- Modify: `canonical/rust-source-health.md`

- [ ] **Step 1: Add the product route contract**

Add a short section near the compact cache/product summary paragraphs:

```markdown
`lumin-rust-analyzer` product mode consumes the compact Rust source-health
library response by default. The full raw `HealthResponse` remains a diagnostic
mode for compatibility and deep inspection only. Compact product mode must
preserve product-visible syntax summaries, function clone summaries, unused
definition summaries, parse/skipped evidence, signal summary evidence, and
opaque-surface examples through typed Rust protocol fields. If a compact lane
omits raw evidence, the product artifact must either expose the corresponding
summary/example projection or mark the raw lane as available only through the
compatibility CLI.
```

- [ ] **Step 2: Verify no banned helper names were introduced**

Run:

```powershell
rg -n "makeSignal|buildSignal|signalForRange|makeParseError|buildParseError|toLocation|rangeToLocation|buildSummary|countSummary|summarizeSignals|createThreadPool|globalThreadPool" canonical experiments\rust-sidecar experiments\rust-main\lumin-rust-analyzer
```

Expected: no new production matches beyond canonical text and existing allowed owners.

---

### Task 2: Expose Compact Source-Health API

**Files:**
- Modify: `experiments/rust-sidecar/rust-source-health/src/driver/analysis.rs`
- Modify: `experiments/rust-sidecar/rust-source-health/src/wrapper/request.rs`
- Modify: `experiments/rust-sidecar/rust-source-health/src/wrapper.rs`
- Modify: `experiments/rust-sidecar/rust-source-health/src/lib.rs`

- [ ] **Step 1: Make the compact response available to dependent crates**

In `driver/analysis.rs`, expose the response type without exposing parser internals:

```rust
pub struct CompactAnalysisResponse {
    pub schema_version: u32,
    pub meta: ResponseMeta,
    pub summary: Summary,
    pub function_clone_groups: crate::protocol::AstFunctionCloneGroups,
    pub unused_definition_analysis: RustUnusedDefinitionAnalysis,
    pub skipped_files: Vec<SkippedFile>,
    pub files: BTreeMap<String, CompactFileHealth>,
}
```

- [ ] **Step 2: Export the compact root analyzer**

In `wrapper/request.rs`, change:

```rust
pub(crate) fn analyze_root_compact(
    options: RustSourceHealthOptions,
) -> Result<CompactAnalysisResponse> {
```

to:

```rust
pub fn analyze_root_compact(options: RustSourceHealthOptions) -> Result<CompactAnalysisResponse> {
```

In `wrapper.rs`, export it:

```rust
pub use request::{analyze_root, analyze_root_compact, RustSourceHealthOptions};
```

In `lib.rs`, export it:

```rust
pub use wrapper::{analyze_root, analyze_root_compact, run_cli, RustSourceHealthOptions};
```

- [ ] **Step 3: Verify source-health compiles**

Run:

```powershell
cargo check --locked -p lumin-rust-source-health
```

Expected: exit 0.

---

### Task 3: Preserve Compact Signal Product Evidence

**Files:**
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/compact.rs`

- [ ] **Step 1: Add compact signal summary details if missing**

`CompactSignalSummary` must carry enough evidence to rebuild analyzer product
syntax projections without raw `signals[]`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactSignalSummary {
    pub total: usize,
    pub review: usize,
    pub muted: usize,
    pub by_kind: BTreeMap<SignalKind, usize>,
    pub muted_by_reason: BTreeMap<SignalMuteReason, usize>,
    pub review_signal_sample_limit: usize,
    pub review_signal_examples: Vec<CompactSignalExample>,
    pub muted_signal_sample_limit: usize,
    pub muted_signal_examples: Vec<CompactSignalExample>,
}
```

Use a project-owned compact example shape:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactSignalExample {
    pub kind: SignalKind,
    pub severity: Severity,
    pub mute_reason: Option<SignalMuteReason>,
    pub location: Location,
}
```

- [ ] **Step 2: Populate compact examples from `FileSignalSummary` plus raw signals**

If `FileSignalSummary` does not retain by-kind/example detail, build the compact
projection before raw signals are pruned. The compact protocol must not restore
full raw `signals[]`; it should keep capped examples only.

- [ ] **Step 3: Verify compact source-health artifact still has bounded output**

Run:

```powershell
cargo test --locked -p lumin-rust-source-health --test integration compact_cli_omits_raw_signal_arrays_by_default
```

Expected: the compact artifact omits `files[*].signals` and includes signal
summary/example projection fields only when evidence exists.

---

### Task 4: Add Analyzer Syntax Phase Adapter

**Files:**
- Create: `experiments/rust-main/lumin-rust-analyzer/src/syntax_phase.rs`
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/main.rs`

- [ ] **Step 1: Create the adapter enum**

```rust
use lumin_rust_source_health::driver::CompactAnalysisResponse;
use lumin_rust_source_health::protocol::{CompactFileHealth, HealthResponse, SkippedFile, Summary};

pub(crate) enum SyntaxPhase<'a> {
    Full(&'a HealthResponse),
    Compact(&'a CompactAnalysisResponse),
}

impl<'a> SyntaxPhase<'a> {
    pub(crate) fn summary(&self) -> &Summary {
        match self {
            Self::Full(response) => &response.summary,
            Self::Compact(response) => &response.summary,
        }
    }

    pub(crate) fn skipped_files(&self) -> &[SkippedFile] {
        match self {
            Self::Full(response) => &response.skipped_files,
            Self::Compact(response) => &response.skipped_files,
        }
    }

    pub(crate) fn is_compact(&self) -> bool {
        matches!(self, Self::Compact(_))
    }
}

pub(crate) enum SyntaxFile<'a> {
    Full(&'a lumin_rust_source_health::protocol::FileHealth),
    Compact(&'a CompactFileHealth),
}
```

Add iterator methods after checking exact lifetime needs in `product_files`.

- [ ] **Step 2: Register the module**

In `main.rs`:

```rust
mod syntax_phase;
```

- [ ] **Step 3: Verify analyzer still compiles with the unused adapter stub**

Run:

```powershell
cargo check --locked -p lumin-rust-analyzer
```

Expected: exit 0 after either using the adapter in later tasks or marking no
dead code by wiring it immediately.

---

### Task 5: Add Analyzer CLI Source-Health Profile

**Files:**
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/cli.rs`
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/cli/analyze.rs`
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/cli/usage.rs`

- [ ] **Step 1: Add typed profile/options**

```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SourceHealthProfile {
    Compact,
    Full,
}
```

Add these fields to `Options`:

```rust
pub(crate) source_health_profile: SourceHealthProfile,
pub(crate) source_health_cache_root: Option<PathBuf>,
pub(crate) source_health_incremental_enabled: bool,
pub(crate) source_health_clear_incremental_cache: bool,
```

- [ ] **Step 2: Parse flags**

Add support in `cli/analyze.rs`:

```rust
"--source-health-profile" => {
    let value = take_string(&mut args, "--source-health-profile")?;
    source_health_profile = SourceHealthProfile::parse(&value)?;
}
"--cache-root" => source_health_cache_root = Some(take_path(&mut args, "--cache-root")?),
"--no-incremental" => source_health_incremental_enabled = false,
"--clear-incremental-cache" => source_health_clear_incremental_cache = true,
```

Default:

```rust
let mut source_health_profile = SourceHealthProfile::Compact;
let mut source_health_cache_root = None;
let mut source_health_incremental_enabled = true;
let mut source_health_clear_incremental_cache = false;
```

- [ ] **Step 3: Update usage**

Usage must mention:

```text
[--source-health-profile compact|full] [--cache-root <path>] [--no-incremental] [--clear-incremental-cache]
```

- [ ] **Step 4: Verify unknown/invalid flag behavior**

Run:

```powershell
cargo test --locked -p lumin-rust-analyzer --test integration cli_usage -- --nocapture
```

Expected: usage tests pass and invalid profile exits as usage error.

---

### Task 6: Route Analyzer Product Mode Through Compact Source-Health

**Files:**
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/main.rs`
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_artifact/build.rs`
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_summary/syntax.rs`
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_artifact/phases/syntax/summary.rs`

- [ ] **Step 1: Select full or compact source-health**

In `run_unified_analyzer`, branch on `options.source_health_profile`:

```rust
let syntax = match options.source_health_profile {
    SourceHealthProfile::Compact => {
        let compact = analyze_root_compact(RustSourceHealthOptions {
            root: root.clone(),
            source_commit: options.source_commit.clone(),
            thread_count: options.thread_count,
            worker_stack_bytes: options.worker_stack_bytes,
            retain_raw_name_refs: false,
            retain_raw_signals: false,
            retain_raw_ast_lanes: false,
            cache_root: options.source_health_cache_root.clone(),
            incremental_enabled: options.source_health_incremental_enabled,
            clear_incremental_cache: options.source_health_clear_incremental_cache,
        })?;
        SyntaxPhaseOwned::Compact(compact)
    }
    SourceHealthProfile::Full => {
        let full = analyze_root(RustSourceHealthOptions {
            root: root.clone(),
            source_commit: options.source_commit.clone(),
            thread_count: options.thread_count,
            worker_stack_bytes: options.worker_stack_bytes,
            retain_raw_name_refs: false,
            retain_raw_signals: true,
            retain_raw_ast_lanes: true,
            cache_root: None,
            incremental_enabled: false,
            clear_incremental_cache: false,
        })?;
        SyntaxPhaseOwned::Full(full)
    }
};
```

Use an owned enum if borrowed lifetimes make direct branching awkward.

- [ ] **Step 2: Change product artifact builders to accept syntax phase**

Replace direct `&HealthResponse` parameters where product projection only needs
summary, files, skipped files, function clone groups, and unused-definition
summary.

- [ ] **Step 3: Keep semantic targeting honest**

`oracle_targeting::targeted_oracle_paths(...)` currently accepts
`&HealthResponse`. Add a compact-compatible path source only if it can preserve
the same selected Rust file paths. If compact files do not retain enough
targeting evidence, keep targeted semantic mode on full syntax until compact
targeting is explicitly implemented and artifact-visible.

- [ ] **Step 4: Verify analyzer tests**

Run:

```powershell
cargo test --locked -p lumin-rust-analyzer --test integration artifact_contract -- --nocapture
```

Expected: product artifact contract passes in compact default mode.

---

### Task 7: Product File Projection From Compact Syntax

**Files:**
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/policy/syntax.rs`
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_files/merge.rs`
- Modify: `experiments/rust-main/lumin-rust-analyzer/src/product_files/model/collection.rs`

- [ ] **Step 1: Add compact file projection**

Add a sibling to `product_syntax_file(file: &FileHealth)`:

```rust
pub(crate) fn product_compact_syntax_file(file: &CompactFileHealth) -> ProductSyntaxFile<'_> {
    // Build the same ProductSyntaxFileProjection fields from compact protocol
    // fields. Raw signal arrays and raw AST lanes remain omitted.
}
```

The compact projection must preserve:

- `sha256`
- `facts`
- `parse`
- `path`
- signal summary counts/by-kind/muted-by-reason
- capped review/muted signal examples if compact protocol carries them
- AST opaque summary
- capped review opaque examples from `CompactAstSummary`

- [ ] **Step 2: Insert compact syntax files**

Extend `ProductFiles` insertion to accept `SyntaxFile::Compact(file)` and call
the compact projection.

- [ ] **Step 3: Verify no raw AST lanes leak into product files**

Run:

```powershell
cargo test --locked -p lumin-rust-analyzer --test integration artifact_contract -- --nocapture
```

Expected: product files contain compact summaries and examples, not raw
`ast.definitions[]`, raw `signals[]`, or raw function body fingerprint arrays.

---

### Task 8: Behavioral Verification

**Files:**
- Test-only modifications under `experiments/rust-main/lumin-rust-analyzer/tests/integration/`
- Test support modifications under `experiments/rust-main/lumin-rust-analyzer/tests/support/`

- [ ] **Step 1: Add compact default artifact behavior check**

The test must run `lumin-rust-analyzer` without `--source-health-profile`.
Expected artifact:

- `meta.producer == "lumin-rust-analyzer"`
- `phases.syntax.meta.producer == "rust-source-health"`
- `artifactRefs.syntax.artifact == "rust-source-health"`
- `summary.syntaxFunctionCloneCandidateGenerationMode == "bounded-retrieval"`
- no raw per-file signal arrays
- no raw per-file AST lanes
- compact incremental metadata exists under the syntax/source-health phase if
  exposed by the adapter

- [ ] **Step 2: Add full diagnostic mode behavior check**

Run with:

```text
--source-health-profile full
```

Expected:

- artifact still validates
- raw lane availability remains documented through `artifactRefs`
- no compact cache metadata is claimed for full diagnostic mode

- [ ] **Step 3: Add focused cache behavior check**

Run the analyzer twice with the same `--cache-root`.
Expected:

- cold run reports changed files
- warm run reports reused files
- both runs have equivalent product summaries for files, function clone counts,
  unused-definition counts, and semantic summary counts

- [ ] **Step 4: Run focused Rust verification**

Run:

```powershell
cargo fmt --package lumin-rust-source-health --package lumin-rust-analyzer --check
cargo clippy --locked -p lumin-rust-source-health -p lumin-rust-analyzer --all-targets -- -D warnings
cargo test --locked -p lumin-rust-source-health --test integration -- --nocapture
cargo test --locked -p lumin-rust-analyzer --test integration -- --nocapture
```

Expected: all commands exit 0.

---

### Task 9: Commit Slice

**Files:**
- All files changed by Tasks 1-8.

- [ ] **Step 1: Review diff scope**

Run:

```powershell
git status --short --branch
git diff --stat
```

Expected: only Rust source-health/analyzer/canonical/test files for this slice.

- [ ] **Step 2: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected: exit 0.

- [ ] **Step 3: Commit**

Run:

```powershell
git add canonical/rust-source-health.md experiments/rust-sidecar/rust-source-health experiments/rust-main/lumin-rust-analyzer
git commit -m "Route analyzer through compact rust source health"
```

- [ ] **Step 4: Push**

Run:

```powershell
git push origin main
```

Expected: remote `main` advances.

---

## Self-Review

- Spec coverage: implements the first handoff from the replacement matrix:
  Rust source-health product route, while preserving full diagnostic mode.
- Scope: does not delete JS/TS `.mjs` owners and does not change JS/TS language
  behavior.
- Data loss: compact mode must preserve product-visible summaries and capped
  examples. Raw arrays are omitted only when `artifactRefs`/phase metadata make
  the compatibility full lane visible.
- Rust-only necessity: compact cache and bounded product projection are Rust
  necessities because Rust source-health owns Rust parser output and raw AST
  lanes were too large for practical product use.
- Verification: Rust-only commands are listed. No Node command is required for
  this slice.
