# Rust Source Health Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Keep tests behavior-focused: one guaranteed happy path, realistic edge cases, and hard-stop paths. Do not add scaffolding tests that only prove files, functions, or modules exist.

**Goal:** Implement the M6 lab-only Rust source health product slice that emits a separate `rust-health.json` artifact from `ra_ap_syntax` syntax facts and review signals.

**Architecture:** Add a new Rust sidecar under `experiments/rust-sidecar/rust-source-health` and a Node wrapper under `_lib/` plus a lab script under `scripts/`. The wrapper owns file discovery, path policy, hashing, UTF-8 decode, skipped-file evidence, final schema validation, summary invariants, and writing `rust-health.json`; the sidecar only reads request JSON from stdin and writes response JSON to stdout. M6 does not touch topology, prefer, quorum, SARIF, markdown audit output, stable commands, or public package defaults.

**Tech Stack:** Node.js ESM, Vitest, Rust 2021, Cargo, `ra_ap_syntax = "=0.0.337"`, `rayon`, `serde`, `serde_json`, `anyhow`, existing `_lib/atomic-write.mjs`.

---

## Design Commitments

- Product-grade vertical slice, not a loose demo.
- Lab-only local script; no stable plugin command and no public default behavior.
- Existing JS/TS `oxc-parser`, tree-sitter WASM, and M2-M5 Rust topology sidecar remain untouched.
- Sidecar never walks the repository and never writes files.
- Wrapper owns raw byte reads, SHA-256 hashing, UTF-8 decoding, path policy, skipped-file evidence, final artifact validation, and atomic writes.
- Fixed Rust parser edition policy for M6: `editionPolicy: "fixed"`, `edition: "2021"`, `editionSource: "m6-policy-default"`.
- Invalid UTF-8 is deterministic: skipped-file reason `invalid-utf8`.
- Signals are syntax-only review prompts, not semantic proof.
- `ra_ap_syntax` is an implementation detail. Internal `pub(crate)` facade/re-exports are allowed; public re-exports and JSON/protocol exposure of `ra_ap_syntax` types are forbidden.
- Shape constructors are centralized. Do not hand-build `Signal`, `ParseError`, `Summary`, or final artifact metadata in multiple modules.
- Per-file Rust parsing and analysis uses an explicit local `rayon` pool; output is re-collected into deterministic sorted structures before serialization.
- Rayon runtime policy is product contract, not incidental implementation detail: configure thread count and worker stack through a dedicated `parallel.rs` boundary.
- Do not share `ra_ap_syntax` ASTs across worker boundaries; parse inside each worker and return plain serializable `FileHealth`.
- Output ordering is deterministic.
- Summary counters must match artifact body.
- Tests must prove behavior and hard stops, not scaffolding accidents.

## File Structure

- Create: `experiments/rust-sidecar/rust-source-health/Cargo.toml`
  - Lab-only Rust sidecar package.
  - Pin `ra_ap_syntax = "=0.0.337"` so parser behavior is reproducible.
- Create: `experiments/rust-sidecar/rust-source-health/src/main.rs`
  - Read stdin, parse request JSON, call analyzer, write response JSON to stdout.
  - Exit non-zero for hard sidecar failures.
- Create: `experiments/rust-sidecar/rust-source-health/src/protocol.rs`
  - Own request/response structs and schema constants.
  - Use only owned project protocol structs; do not expose `ra_ap_syntax` types here.
- Create: `experiments/rust-sidecar/rust-source-health/src/analyzer.rs`
  - Own `ra_ap_syntax` parsing and syntax-only facts/signals.
- Create: `experiments/rust-sidecar/rust-source-health/src/locations.rs`
  - Convert byte offsets to line/column and keep ranges deterministic.
- Create: `experiments/rust-sidecar/rust-source-health/src/signals.rs`
  - Own `Signal` and `ParseError` constructors so syntax-only claim/severity/location shape is not repeated.
- Create: `experiments/rust-sidecar/rust-source-health/src/summary.rs`
  - Own `Summary` construction from `BTreeMap<String, FileHealth>`.
- Create: `experiments/rust-sidecar/rust-source-health/src/parallel.rs`
  - Own Rayon thread-count and worker-stack policy through a local pool, not ad hoc `par_iter()` calls hidden in analyzer code.
- Create: `experiments/rust-sidecar/rust-source-health/tests/health.rs`
  - Rust sidecar behavior coverage for parse success, signals, parse errors, ordering, and schema hard stops.
- Create: `_lib/rust-source-health-schema.mjs`
  - Own JS-side schema constants, final artifact validation, summary invariant checks, and deterministic sorting helpers.
- Create: `_lib/rust-source-health-runner.mjs`
  - Own file collection, hashing, UTF-8 decode, skipped-file evidence, sidecar execution, runtime request construction, final artifact assembly, and atomic write.
- Create: `scripts/run-rust-source-health.mjs`
  - Lab CLI for running the product slice.
- Create: `tests/rust-source-health-schema.test.mjs`
  - Behavior coverage for schema validation, summary invariants, deterministic ordering, and skipped-file evidence.
- Create: `tests/rust-source-health-runner.test.mjs`
  - Behavior coverage for wrapper happy path, invalid UTF-8 skipped file, invalid sidecar output hard stop, timeout hard stop, and boundary no-change guarantee.
- Modify: `tests/README.md`
  - Regenerate after adding test files.

No files under `skills/` should be modified in this plan. M6 starts as a root lab script and sidecar, not as a packaged skill command.

## External API Facts Checked

- `ra_ap_syntax` latest on docs.rs is `0.0.337`.
- `ra_ap_syntax::ast::SourceFile::parse(text, edition)` accepts an explicit `Edition`.
- `ra_ap_syntax::Edition` includes `Edition2021` and `Edition2024`; M6 uses `Edition2021` by policy.

## Task 1: Add The Rust Sidecar Package

**Files:**
- Create: `experiments/rust-sidecar/rust-source-health/Cargo.toml`
- Create: `experiments/rust-sidecar/rust-source-health/src/main.rs`
- Create: `experiments/rust-sidecar/rust-source-health/src/protocol.rs`
- Create: `experiments/rust-sidecar/rust-source-health/src/analyzer.rs`
- Create: `experiments/rust-sidecar/rust-source-health/src/locations.rs`
- Create: `experiments/rust-sidecar/rust-source-health/src/signals.rs`
- Create: `experiments/rust-sidecar/rust-source-health/src/summary.rs`
- Create: `experiments/rust-sidecar/rust-source-health/src/parallel.rs`
- Test: `experiments/rust-sidecar/rust-source-health/tests/health.rs`

- [ ] **Step 1: Create the sidecar package manifest**

Create `experiments/rust-sidecar/rust-source-health/Cargo.toml`:

```toml
[package]
name = "lumin-rust-source-health"
version = "0.0.0-lab.0"
edition = "2021"
license = "MIT"

[dependencies]
anyhow = "1"
ra_ap_syntax = "=0.0.337"
rayon = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Add the protocol structs**

Create `experiments/rust-sidecar/rust-source-health/src/protocol.rs` with these public contracts:

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 1;
pub const POLICY_VERSION: &str = "m6-rust-source-health-syntax-v1";
pub const PARSER_KIND: &str = "ra_ap_syntax";
pub const PARSER_VERSION: &str = "0.0.337";
pub const PARSER_EDITION: &str = "2021";
pub const PARSER_EDITION_POLICY: &str = "fixed";
pub const PARSER_EDITION_SOURCE: &str = "m6-policy-default";
pub const DEFAULT_WORKER_STACK_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct HealthRequest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub root: String,
    pub files: Vec<RequestFile>,
    #[serde(rename = "pathPolicy")]
    pub path_policy: PathPolicy,
    pub parser: ParserRequest,
    pub runtime: RuntimeRequest,
}

#[derive(Debug, Deserialize)]
pub struct RequestFile {
    pub path: String,
    pub sha256: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct PathPolicy {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ParserRequest {
    #[serde(rename = "editionPolicy")]
    pub edition_policy: String,
    pub edition: String,
    #[serde(rename = "editionSource")]
    pub edition_source: String,
}

#[derive(Debug, Deserialize)]
pub struct RuntimeRequest {
    #[serde(rename = "threadCount")]
    pub thread_count: Option<usize>,
    #[serde(rename = "workerStackBytes")]
    pub worker_stack_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub meta: ResponseMeta,
    pub summary: Summary,
    #[serde(rename = "skippedFiles")]
    pub skipped_files: Vec<SkippedFile>,
    pub files: BTreeMap<String, FileHealth>,
}

#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    pub producer: String,
    pub mode: String,
    pub parser: ParserMeta,
    pub policy: PolicyMeta,
    pub runtime: RuntimeMeta,
    pub limits: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ParserMeta {
    pub kind: String,
    pub version: String,
    #[serde(rename = "editionPolicy")]
    pub edition_policy: String,
    pub edition: String,
    #[serde(rename = "editionSource")]
    pub edition_source: String,
}

#[derive(Debug, Serialize)]
pub struct PolicyMeta {
    pub version: String,
    pub thresholds: Thresholds,
}

#[derive(Debug, Serialize)]
pub struct RuntimeMeta {
    #[serde(rename = "threadCount")]
    pub thread_count: usize,
    #[serde(rename = "workerStackBytes")]
    pub worker_stack_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct Thresholds {
    #[serde(rename = "maxFunctionLines")]
    pub max_function_lines: usize,
    #[serde(rename = "maxImplLines")]
    pub max_impl_lines: usize,
}

#[derive(Debug, Serialize, Default)]
pub struct Summary {
    pub files: usize,
    #[serde(rename = "skippedFiles")]
    pub skipped_files: usize,
    #[serde(rename = "parseErrorFiles")]
    pub parse_error_files: usize,
    #[serde(rename = "parseErrors")]
    pub parse_errors: usize,
    pub functions: usize,
    #[serde(rename = "unsafeBlocks")]
    pub unsafe_blocks: usize,
    #[serde(rename = "unsafeFunctions")]
    pub unsafe_functions: usize,
    pub signals: usize,
    #[serde(rename = "signalsByKind")]
    pub signals_by_kind: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct FileHealth {
    pub sha256: String,
    pub facts: Facts,
    pub signals: Vec<Signal>,
    pub parse: ParseStatus,
    pub path: PathMeta,
}

#[derive(Debug, Serialize, Default)]
pub struct Facts {
    pub items: usize,
    pub functions: usize,
    #[serde(rename = "maxFunctionLines")]
    pub max_function_lines: usize,
    #[serde(rename = "unsafeBlocks")]
    pub unsafe_blocks: usize,
    #[serde(rename = "unsafeFunctions")]
    pub unsafe_functions: usize,
}

#[derive(Debug, Serialize)]
pub struct Signal {
    pub kind: String,
    pub severity: String,
    pub claim: String,
    pub location: Location,
}

#[derive(Debug, Serialize)]
pub struct ParseStatus {
    pub ok: bool,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Serialize)]
pub struct ParseError {
    pub message: String,
    pub claim: String,
    pub location: Location,
}

#[derive(Debug, Serialize, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    #[serde(rename = "endLine")]
    pub end_line: usize,
    #[serde(rename = "endColumn")]
    pub end_column: usize,
    #[serde(rename = "byteStart")]
    pub byte_start: usize,
    #[serde(rename = "byteEnd")]
    pub byte_end: usize,
}

#[derive(Debug, Serialize)]
pub struct PathMeta {
    pub classifications: Vec<String>,
    pub suppressed: bool,
}

#[derive(Debug, Serialize)]
pub struct SkippedFile {
    pub path: String,
    pub reason: String,
}
```

- [ ] **Step 3: Add byte range to line/column conversion**

Create `experiments/rust-sidecar/rust-source-health/src/locations.rs`:

```rust
use crate::protocol::Location;

#[derive(Debug, Clone)]
pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self { line_starts }
    }

    pub fn location(&self, byte_start: usize, byte_end: usize) -> Location {
        let (line, column) = self.point(byte_start);
        let (end_line, end_column) = self.point(byte_end);
        Location {
            line,
            column,
            end_line,
            end_column,
            byte_start,
            byte_end,
        }
    }

    fn point(&self, byte_offset: usize) -> (usize, usize) {
        let idx = match self.line_starts.binary_search(&byte_offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = self.line_starts.get(idx).copied().unwrap_or(0);
        (idx + 1, byte_offset.saturating_sub(line_start) + 1)
    }
}
```

- [ ] **Step 4: Add signal and parse-error constructors**

Create `experiments/rust-sidecar/rust-source-health/src/signals.rs`:

```rust
use ra_ap_syntax::TextRange;

use crate::locations::LineIndex;
use crate::protocol::{ParseError, Signal};

pub fn review_signal(kind: &str, line_index: &LineIndex, range: TextRange) -> Signal {
    Signal {
        kind: kind.to_string(),
        severity: "review".to_string(),
        claim: "syntax-only".to_string(),
        location: line_index.location(u32::from(range.start()) as usize, u32::from(range.end()) as usize),
    }
}

pub fn syntax_parse_error(message: String, line_index: &LineIndex, range: TextRange) -> ParseError {
    ParseError {
        message,
        claim: "syntax-only".to_string(),
        location: line_index.location(u32::from(range.start()) as usize, u32::from(range.end()) as usize),
    }
}
```

All `Signal` and `ParseError` construction must go through this module. Do not repeat `"review"` or `"syntax-only"` literals in `analyzer.rs` or tests except when asserting output values.

- [ ] **Step 5: Add summary construction**

Create `experiments/rust-sidecar/rust-source-health/src/summary.rs`:

```rust
use std::collections::BTreeMap;

use crate::protocol::{FileHealth, Summary};

pub fn summarize(files: &BTreeMap<String, FileHealth>) -> Summary {
    let mut summary = Summary::default();
    summary.files = files.len();

    for file in files.values() {
        if !file.parse.ok {
            summary.parse_error_files += 1;
        }
        summary.parse_errors += file.parse.errors.len();
        summary.functions += file.facts.functions;
        summary.unsafe_blocks += file.facts.unsafe_blocks;
        summary.unsafe_functions += file.facts.unsafe_functions;
        summary.signals += file.signals.len();

        for signal in &file.signals {
            *summary
                .signals_by_kind
                .entry(signal.kind.clone())
                .or_insert(0) += 1;
        }
    }

    summary
}
```

`analyzer.rs` must call this helper. Do not keep a second summary implementation in tests or wrapper code; JS has its own independent final artifact invariant checker because it validates serialized output after wrapper-owned skipped files are appended.

- [ ] **Step 6: Add the Rayon runtime boundary**

Create `experiments/rust-sidecar/rust-source-health/src/parallel.rs`:

```rust
use anyhow::{bail, Result};
use rayon::ThreadPool;

use crate::protocol::DEFAULT_WORKER_STACK_BYTES;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub thread_count: Option<usize>,
    pub worker_stack_bytes: usize,
}

pub fn build_pool(config: &RuntimeConfig) -> Result<ThreadPool> {
    if matches!(config.thread_count, Some(0)) {
        bail!("runtime.threadCount must be greater than zero when provided");
    }
    if config.worker_stack_bytes < DEFAULT_WORKER_STACK_BYTES {
        bail!(
            "runtime.workerStackBytes must be at least {}",
            DEFAULT_WORKER_STACK_BYTES
        );
    }

    let mut builder = rayon::ThreadPoolBuilder::new()
        .stack_size(config.worker_stack_bytes);
    if let Some(thread_count) = config.thread_count {
        builder = builder.num_threads(thread_count);
    }
    builder
        .build()
        .map_err(|error| anyhow::anyhow!("failed to build Rayon pool: {error}"))
}
```

This is an explicit Rayon pool, not implicit default Rayon behavior. Use a local pool rather than `build_global()` so tests stay isolated and hidden process-global state does not become part of the product contract. The default worker stack is intentionally explicit because `fallow` hit real stack pressure in deep AST/graph work; M6 should start with the safer stack policy instead of rediscovering that failure.

- [ ] **Step 7: Add the syntax analyzer**

Create `experiments/rust-sidecar/rust-source-health/src/analyzer.rs` with this behavior:

- Parse each file using `SourceFile::parse(text, Edition::Edition2021)`.
- Count syntactic item/function nodes across the file.
- Count unsafe blocks and unsafe functions separately.
- Emit syntax-only signals for `.unwrap()`, `.expect(...)`, `.clone()`, `panic!`, `todo!`, and `unimplemented!`.
- Emit oversized function/impl signals from thresholds.
- Convert every signal and parse error to range locations.
- Sort signals by `byteStart`, then `kind`.

Use this skeleton and fill only the functions named in it:

```rust
use std::collections::BTreeMap;

use ra_ap_syntax::{ast, AstNode, Edition, SourceFile, SyntaxKind, SyntaxNode, TextRange};
use rayon::prelude::*;

use crate::locations::LineIndex;
use crate::signals::{review_signal, syntax_parse_error};
use crate::summary::summarize;
use crate::protocol::{
    Facts, FileHealth, ParseStatus, PathMeta, RequestFile, Signal, Summary, Thresholds,
};

pub fn analyze_files(files: &[RequestFile], thresholds: &Thresholds) -> (BTreeMap<String, FileHealth>, Summary) {
    let analyzed = files
        .par_iter()
        .map(|file| (file.path.clone(), analyze_file(file, thresholds)))
        .collect::<Vec<_>>();
    let out = analyzed.into_iter().collect::<BTreeMap<_, _>>();
    let summary = summarize(&out);
    (out, summary)
}

fn analyze_file(file: &RequestFile, thresholds: &Thresholds) -> FileHealth {
    let line_index = LineIndex::new(&file.text);
    let parsed = SourceFile::parse(&file.text, Edition::Edition2021);
    let root = parsed.tree().syntax().clone();
    let mut signals = Vec::new();
    let facts = collect_facts_and_signals(&root, &line_index, thresholds, &mut signals);
    let mut errors = parsed
        .errors()
        .into_iter()
        .map(|error| syntax_parse_error(error.to_string(), &line_index, error.range()))
        .collect::<Vec<_>>();
    errors.sort_by(|a, b| {
        a.location
            .byte_start
            .cmp(&b.location.byte_start)
            .then(a.message.cmp(&b.message))
    });

    signals.sort_by(|a, b| {
        a.location
            .byte_start
            .cmp(&b.location.byte_start)
            .then(a.kind.cmp(&b.kind))
    });

    FileHealth {
        sha256: file.sha256.clone(),
        facts,
        signals,
        parse: ParseStatus { ok: errors.is_empty(), errors },
        path: PathMeta {
            classifications: classify_path(&file.path),
            suppressed: false,
        },
    }
}

fn collect_facts_and_signals(
    root: &SyntaxNode,
    line_index: &LineIndex,
    thresholds: &Thresholds,
    signals: &mut Vec<Signal>,
) -> Facts {
    let mut facts = Facts::default();
    for node in root.descendants() {
        match node.kind() {
            SyntaxKind::FN => {
                facts.functions += 1;
                if function_is_unsafe(&node) {
                    facts.unsafe_functions += 1;
                }
                let lines = line_span(line_index, node.text_range());
                facts.max_function_lines = facts.max_function_lines.max(lines);
                if lines > thresholds.max_function_lines {
                    signals.push(review_signal("oversized-function", line_index, node.text_range()));
                }
            }
            SyntaxKind::IMPL => {
                let lines = line_span(line_index, node.text_range());
                if lines > thresholds.max_impl_lines {
                    signals.push(review_signal("oversized-impl", line_index, node.text_range()));
                }
            }
            _ if is_unsafe_block_expr(&node) => {
                facts.unsafe_blocks += 1;
                signals.push(review_signal("unsafe-block", line_index, node.text_range()));
            }
            _ => {}
        }
    }
    facts.items = count_items(root);
    collect_syntax_signals(root, line_index, signals);
    facts
}

fn classify_path(path: &str) -> Vec<String> {
    if path.contains("/generated/") || path.ends_with("generated.rs") {
        vec!["generated".to_string()]
    } else if path.contains("/tests/") || path.ends_with("_test.rs") {
        vec!["test".to_string()]
    } else {
        vec!["source".to_string()]
    }
}
```

Add the remaining helper functions in the same file:

- `count_items(root)`: count syntax nodes whose kind is one of `FN`, `STRUCT`, `ENUM`, `TRAIT`, `IMPL`, `MOD`, `CONST`, `STATIC`, `TYPE_ALIAS`.
- `function_is_unsafe(node)`: return true when an `FN` node has a direct child token with `SyntaxKind::UNSAFE_KW`.
- `is_unsafe_block_expr(node)`: return true for a block expression whose direct children include `SyntaxKind::UNSAFE_KW`; do not depend on a speculative unsafe-expression syntax kind.
- `line_span(line_index, range)`: compute inclusive line span from `LineIndex::location`; use `range.end() - 1` for the end point when the range is non-empty so a node ending at the next line start does not count an extra line.
- `collect_syntax_signals(root, line_index, signals)`: inspect syntax shape, not bare identifier occurrences:
  - use `ast::MethodCallExpr` and emit `unwrap-call`, `expect-call`, or `clone-call` only when the method name is exactly `unwrap`, `expect`, or `clone`;
  - use `ast::MacroCall` for `panic!`, `todo!`, and `unimplemented!` when possible; fall back to token scanning only for macro syntax that `ast::MacroCall` cannot cast.
- Do not implement summary calculation in this file; use `crate::summary::summarize`.

Use this `ast::MethodCallExpr` pattern for method-call signals:

```rust
for node in root.descendants() {
    if let Some(call) = ast::MethodCallExpr::cast(node.clone()) {
        if let Some(name_ref) = call.name_ref() {
            match name_ref.text().as_str() {
                "unwrap" => signals.push(review_signal("unwrap-call", line_index, node.text_range())),
                "expect" => signals.push(review_signal("expect-call", line_index, node.text_range())),
                "clone" => signals.push(review_signal("clone-call", line_index, node.text_range())),
                _ => {}
            }
        }
    }
}
```

Do not emit method-call signals for plain identifier declarations or fields such as:

```rust
fn unwrap() {}
let clone = 1;
struct S { expect: bool }
```

Use this unsafe-block helper shape and adjust only if `ra_ap_syntax = "=0.0.337"` proves a different CST shape in the fixture test:

```rust
fn is_unsafe_block_expr(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::BLOCK_EXPR
        && node
            .children_with_tokens()
            .any(|child| child.kind() == SyntaxKind::UNSAFE_KW)
}
```

- [ ] **Step 8: Add `main.rs`**

Create `experiments/rust-sidecar/rust-source-health/src/main.rs`:

```rust
mod analyzer;
mod locations;
mod parallel;
mod protocol;
mod signals;
mod summary;

use std::io::{self, Read};

use anyhow::{bail, Context, Result};
use analyzer::analyze_files;
use parallel::{build_pool, RuntimeConfig};
use protocol::{
    DEFAULT_WORKER_STACK_BYTES, HealthRequest, HealthResponse, ParserMeta, PolicyMeta,
    ResponseMeta, RuntimeMeta, SCHEMA_VERSION, PARSER_EDITION, PARSER_EDITION_POLICY,
    PARSER_EDITION_SOURCE, PARSER_KIND, PARSER_VERSION, POLICY_VERSION, Thresholds,
};

const MAX_FUNCTION_LINES: usize = 80;
const MAX_IMPL_LINES: usize = 200;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: HealthRequest = serde_json::from_str(&input).context("invalid request JSON")?;
    validate_request(&request)?;

    let thresholds = Thresholds {
        max_function_lines: MAX_FUNCTION_LINES,
        max_impl_lines: MAX_IMPL_LINES,
    };
    let runtime = RuntimeConfig {
        thread_count: request.runtime.thread_count,
        worker_stack_bytes: request.runtime.worker_stack_bytes,
    };
    let pool = build_pool(&runtime)?;
    let thread_count = pool.current_num_threads();
    let (files, summary) = pool.install(|| analyze_files(&request.files, &thresholds));

    let response = HealthResponse {
        schema_version: SCHEMA_VERSION,
        meta: ResponseMeta {
            producer: "rust-source-health".to_string(),
            mode: "syntax-only".to_string(),
            parser: ParserMeta {
                kind: PARSER_KIND.to_string(),
                version: PARSER_VERSION.to_string(),
                edition_policy: PARSER_EDITION_POLICY.to_string(),
                edition: PARSER_EDITION.to_string(),
                edition_source: PARSER_EDITION_SOURCE.to_string(),
            },
            policy: PolicyMeta {
                version: POLICY_VERSION.to_string(),
                thresholds,
            },
            runtime: RuntimeMeta {
                thread_count,
                worker_stack_bytes: runtime.worker_stack_bytes,
            },
            limits: vec![
                "syntax-only".to_string(),
                "no-type-info".to_string(),
                "no-trait-solving".to_string(),
                "no-borrow-check".to_string(),
            ],
        },
        summary,
        skipped_files: Vec::new(),
        files,
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

fn validate_request(request: &HealthRequest) -> Result<()> {
    if request.schema_version != SCHEMA_VERSION {
        bail!("unsupported schemaVersion {}", request.schema_version);
    }
    if request.parser.edition_policy != PARSER_EDITION_POLICY
        || request.parser.edition != PARSER_EDITION
        || request.parser.edition_source != PARSER_EDITION_SOURCE
    {
        bail!("unsupported parser edition policy");
    }
    if request.runtime.worker_stack_bytes < DEFAULT_WORKER_STACK_BYTES {
        bail!(
            "runtime.workerStackBytes must be at least {}",
            DEFAULT_WORKER_STACK_BYTES
        );
    }
    Ok(())
}
```

- [ ] **Step 9: Add Rust sidecar behavior tests**

Create `experiments/rust-sidecar/rust-source-health/tests/health.rs` with these cases:

- happy path: one source file with two functions produces parse-ok file facts and sorted signals;
- realistic edge case: `.unwrap()`, `.clone()`, `unsafe`, and `todo!` produce syntax-only review signals with byte ranges;
- unsafe fixture: `fn main() { unsafe { do_thing(); } }` produces `facts.unsafeBlocks === 1` and `signalsByKind["unsafe-block"] === 1`;
- negative syntax edge case: `fn unwrap() {}`, `let clone = 1;`, and `struct S { expect: bool }` do not produce `unwrap-call`, `clone-call`, or `expect-call`;
- syntax edge case: malformed Rust records parse errors and still returns exit 0 through the sidecar;
- summary invariant edge case: returned summary includes correct `signalsByKind`, `parseErrorFiles`, and `parseErrors` values;
- runtime edge case: `runtime.workerStackBytes` below `16777216` exits non-zero and does not emit valid artifact JSON;
- hard stop: unsupported `schemaVersion` exits non-zero and does not emit valid artifact JSON.

Use a helper that launches the compiled test binary through Cargo's integration test environment:

```rust
use std::io::Write;
use std::process::{Command, Stdio};

fn run_sidecar(input: &serde_json::Value) -> (std::process::ExitStatus, String, String) {
    let bin = env!("CARGO_BIN_EXE_lumin-rust-source-health");
    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sidecar");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(input.to_string().as_bytes())
        .expect("write request");
    let output = child.wait_with_output().expect("wait sidecar");
    (
        output.status,
        String::from_utf8(output.stdout).expect("stdout utf8"),
        String::from_utf8(output.stderr).expect("stderr utf8"),
    )
}
```

The hard-stop test should assert behavior, not binary presence:

```rust
#[test]
fn rejects_unsupported_schema_without_valid_artifact() {
    let request = serde_json::json!({
        "schemaVersion": 999,
        "root": "C:/repo",
        "files": [],
        "pathPolicy": { "include": ["**/*.rs"], "exclude": [] },
        "parser": {
            "editionPolicy": "fixed",
            "edition": "2021",
            "editionSource": "m6-policy-default"
        },
        "runtime": {
            "threadCount": 1,
            "workerStackBytes": 16777216
        }
    });
    let (status, stdout, stderr) = run_sidecar(&request);
    assert!(!status.success());
    assert!(stdout.trim().is_empty());
    assert!(stderr.contains("unsupported schemaVersion"));
}
```

- [ ] **Step 10: Verify Rust sidecar behavior**

Run:

```bash
cargo test --manifest-path experiments/rust-sidecar/rust-source-health/Cargo.toml
```

Expected:

- Rust integration tests pass.
- Cargo creates `Cargo.lock` under `experiments/rust-sidecar/rust-source-health/`.

- [ ] **Step 11: Commit sidecar package**

Run:

```bash
git add experiments/rust-sidecar/rust-source-health
git commit -m "Add Rust source health sidecar"
```

## Task 2: Add JS Artifact Schema And Invariants

**Files:**
- Create: `_lib/rust-source-health-schema.mjs`
- Test: `tests/rust-source-health-schema.test.mjs`

- [ ] **Step 1: Create schema module**

Create `_lib/rust-source-health-schema.mjs`:

```js
export const RUST_SOURCE_HEALTH_SCHEMA_VERSION = 1;
export const RUST_SOURCE_HEALTH_POLICY_VERSION = 'm6-rust-source-health-syntax-v1';
export const RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES = 16 * 1024 * 1024;
export const RUST_SOURCE_HEALTH_PARSER = Object.freeze({
  kind: 'ra_ap_syntax',
  version: '0.0.337',
  editionPolicy: 'fixed',
  edition: '2021',
  editionSource: 'm6-policy-default',
});

export function sortRustHealthArtifact(artifact) {
  const files = Object.fromEntries(
    Object.entries(artifact.files ?? {})
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([file, value]) => [
        file,
        {
          ...value,
          signals: [...(value.signals ?? [])].sort(compareSignals),
        },
      ]),
  );
  const skippedFiles = [...(artifact.skippedFiles ?? [])]
    .sort((left, right) => String(left.path).localeCompare(String(right.path)));
  return { ...artifact, skippedFiles, files };
}

function compareSignals(left, right) {
  return (
    Number(left?.location?.byteStart ?? 0) -
      Number(right?.location?.byteStart ?? 0) ||
    String(left?.kind ?? '').localeCompare(String(right?.kind ?? ''))
  );
}

function stableObject(value = {}) {
  return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => left.localeCompare(right)),
  );
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function isSha256(value) {
  return typeof value === 'string' && /^sha256:[a-f0-9]{64}$/i.test(value);
}

function isLocation(value) {
  return isPlainObject(value) &&
    Number.isInteger(value.line) &&
    value.line > 0 &&
    Number.isInteger(value.column) &&
    value.column > 0 &&
    Number.isInteger(value.endLine) &&
    value.endLine > 0 &&
    Number.isInteger(value.endColumn) &&
    value.endColumn > 0 &&
    Number.isInteger(value.byteStart) &&
    value.byteStart >= 0 &&
    Number.isInteger(value.byteEnd) &&
    value.byteEnd >= value.byteStart;
}

export function summarizeRustHealthArtifact(artifact) {
  const fileEntries = Object.values(artifact.files ?? {});
  const signals = fileEntries.flatMap((entry) => entry.signals ?? []);
  const parseErrors = fileEntries.flatMap((entry) => entry.parse?.errors ?? []);
  const signalsByKind = {};
  for (const signal of signals) {
    const kind = String(signal.kind ?? '');
    signalsByKind[kind] = (signalsByKind[kind] ?? 0) + 1;
  }
  return {
    files: fileEntries.length,
    skippedFiles: Array.isArray(artifact.skippedFiles) ? artifact.skippedFiles.length : 0,
    parseErrorFiles: fileEntries.filter((entry) => entry.parse?.ok === false).length,
    parseErrors: parseErrors.length,
    functions: fileEntries.reduce((sum, entry) => sum + Number(entry.facts?.functions ?? 0), 0),
    unsafeBlocks: fileEntries.reduce((sum, entry) => sum + Number(entry.facts?.unsafeBlocks ?? 0), 0),
    unsafeFunctions: fileEntries.reduce((sum, entry) => sum + Number(entry.facts?.unsafeFunctions ?? 0), 0),
    signals: signals.length,
    signalsByKind,
  };
}

export function rustHealthInvariantProblems(artifact) {
  const expected = summarizeRustHealthArtifact(artifact);
  const actual = artifact.summary ?? {};
  const problems = [];
  for (const [key, value] of Object.entries(expected)) {
    if (key === 'signalsByKind') {
      if (
        JSON.stringify(stableObject(actual.signalsByKind ?? {})) !==
        JSON.stringify(stableObject(value))
      ) {
        problems.push('summary.signalsByKind mismatch');
      }
    } else if (actual[key] !== value) {
      problems.push(`summary.${key} expected ${value} but found ${actual[key]}`);
    }
  }
  return problems;
}

function isIsoTimestamp(value) {
  return typeof value === 'string' && !Number.isNaN(Date.parse(value));
}

function validateRustHealthArtifactShape(artifact, { requireWrapperMeta }) {
  const problems = [];
  if (artifact?.schemaVersion !== RUST_SOURCE_HEALTH_SCHEMA_VERSION) {
    problems.push('schemaVersion mismatch');
  }
  if (artifact?.meta?.producer !== 'rust-source-health') {
    problems.push('meta.producer mismatch');
  }
  if (artifact?.meta?.mode !== 'syntax-only') {
    problems.push('meta.mode mismatch');
  }
  if (
    !Number.isInteger(artifact?.meta?.runtime?.threadCount) ||
    artifact.meta.runtime.threadCount <= 0
  ) {
    problems.push('meta.runtime.threadCount invalid');
  }
  if (
    !Number.isInteger(artifact?.meta?.runtime?.workerStackBytes) ||
    artifact.meta.runtime.workerStackBytes < RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES
  ) {
    problems.push('meta.runtime.workerStackBytes invalid');
  }
  const limits = artifact?.meta?.limits;
  for (const limit of ['syntax-only', 'no-type-info', 'no-trait-solving', 'no-borrow-check']) {
    if (!Array.isArray(limits) || !limits.includes(limit)) {
      problems.push(`meta.limits missing ${limit}`);
    }
  }
  if (artifact?.meta?.policy?.version !== RUST_SOURCE_HEALTH_POLICY_VERSION) {
    problems.push('policy.version mismatch');
  }
  if (artifact?.meta?.parser?.kind !== RUST_SOURCE_HEALTH_PARSER.kind) {
    problems.push('parser.kind mismatch');
  }
  if (artifact?.meta?.parser?.version !== RUST_SOURCE_HEALTH_PARSER.version) {
    problems.push('parser.version mismatch');
  }
  if (artifact?.meta?.parser?.editionPolicy !== RUST_SOURCE_HEALTH_PARSER.editionPolicy) {
    problems.push('parser.editionPolicy mismatch');
  }
  if (artifact?.meta?.parser?.edition !== RUST_SOURCE_HEALTH_PARSER.edition) {
    problems.push('parser.edition mismatch');
  }
  if (artifact?.meta?.parser?.editionSource !== RUST_SOURCE_HEALTH_PARSER.editionSource) {
    problems.push('parser.editionSource mismatch');
  }
  if (requireWrapperMeta) {
    if (!isIsoTimestamp(artifact?.meta?.generated)) {
      problems.push('meta.generated invalid');
    }
    if (typeof artifact?.meta?.sidecar?.sourceCommit !== 'string' || artifact.meta.sidecar.sourceCommit.length === 0) {
      problems.push('meta.sidecar.sourceCommit missing');
    }
    if (!isSha256(artifact?.meta?.sidecar?.binarySha256)) {
      problems.push('meta.sidecar.binarySha256 invalid');
    }
    if (!isPlainObject(artifact?.meta?.input?.pathPolicy)) {
      problems.push('meta.input.pathPolicy missing');
    }
  }
  if (!isPlainObject(artifact?.files)) {
    problems.push('files must be an object');
  }
  if (!Array.isArray(artifact?.skippedFiles)) {
    problems.push('skippedFiles must be an array');
  }
  for (const [filePath, file] of Object.entries(artifact?.files ?? {})) {
    if (!isSha256(file?.sha256)) {
      problems.push(`files.${filePath}.sha256 invalid`);
    }
    if (!isPlainObject(file?.facts)) {
      problems.push(`files.${filePath}.facts missing`);
    }
    if (!Array.isArray(file?.signals)) {
      problems.push(`files.${filePath}.signals must be an array`);
    }
    if (!isPlainObject(file?.parse) || typeof file.parse.ok !== 'boolean') {
      problems.push(`files.${filePath}.parse invalid`);
    }
    if (!isPlainObject(file?.path) || !Array.isArray(file.path.classifications)) {
      problems.push(`files.${filePath}.path invalid`);
    }
    for (const signal of file?.signals ?? []) {
      if (typeof signal.kind !== 'string' || signal.kind.length === 0) {
        problems.push(`files.${filePath}.signal.kind invalid`);
      }
      if (signal.severity !== 'review') {
        problems.push(`files.${filePath}.signal.severity mismatch`);
      }
      if (signal.claim !== 'syntax-only') {
        problems.push(`files.${filePath}.signal.claim mismatch`);
      }
      if (!isLocation(signal.location)) {
        problems.push(`files.${filePath}.signal.location invalid`);
      }
    }
    for (const parseError of file?.parse?.errors ?? []) {
      if (typeof parseError.message !== 'string' || parseError.message.length === 0) {
        problems.push(`files.${filePath}.parse.error.message invalid`);
      }
      if (parseError.claim !== 'syntax-only') {
        problems.push(`files.${filePath}.parse.error.claim mismatch`);
      }
      if (!isLocation(parseError.location)) {
        problems.push(`files.${filePath}.parse.error.location invalid`);
      }
    }
  }
  problems.push(...rustHealthInvariantProblems(artifact));
  return problems;
}

export function validateRustHealthSidecarArtifact(artifact) {
  return validateRustHealthArtifactShape(artifact, { requireWrapperMeta: false });
}

export function validateRustHealthFinalArtifact(artifact) {
  return validateRustHealthArtifactShape(artifact, { requireWrapperMeta: true });
}

export const validateRustHealthArtifact = validateRustHealthFinalArtifact;
```

- [ ] **Step 2: Add schema behavior tests**

Create `tests/rust-source-health-schema.test.mjs`:

```js
import { describe, expect, it } from 'vitest';

import {
  sortRustHealthArtifact,
  summarizeRustHealthArtifact,
  validateRustHealthFinalArtifact,
  validateRustHealthSidecarArtifact,
} from '../_lib/rust-source-health-schema.mjs';

function artifact(overrides = {}) {
  return {
    schemaVersion: 1,
    meta: {
      producer: 'rust-source-health',
      mode: 'syntax-only',
      generated: '2026-06-16T10:00:00.000Z',
      sidecar: {
        sourceCommit: 'abc123',
        binarySha256: `sha256:${'b'.repeat(64)}`,
      },
      input: {
        pathPolicy: { include: ['**/*.rs'], exclude: ['target/**', 'vendor/**'] },
      },
      runtime: { threadCount: 2, workerStackBytes: 16777216 },
      limits: ['syntax-only', 'no-type-info', 'no-trait-solving', 'no-borrow-check'],
      policy: { version: 'm6-rust-source-health-syntax-v1', thresholds: {} },
      parser: {
        kind: 'ra_ap_syntax',
        version: '0.0.337',
        editionPolicy: 'fixed',
        edition: '2021',
        editionSource: 'm6-policy-default',
      },
    },
    summary: {
      files: 1,
      skippedFiles: 1,
      parseErrorFiles: 0,
      parseErrors: 0,
      functions: 2,
      unsafeBlocks: 1,
      unsafeFunctions: 1,
      signals: 2,
      signalsByKind: { 'clone-call': 1, 'unwrap-call': 1 },
    },
    skippedFiles: [{ path: 'target/generated.rs', reason: 'excluded-by-path-policy' }],
    files: {
      'src/lib.rs': {
        sha256: `sha256:${'a'.repeat(64)}`,
        facts: { functions: 2, unsafeBlocks: 1, unsafeFunctions: 1 },
        signals: [
          {
            kind: 'unwrap-call',
            severity: 'review',
            claim: 'syntax-only',
            location: { line: 2, column: 3, endLine: 2, endColumn: 11, byteStart: 20, byteEnd: 28 },
          },
          {
            kind: 'clone-call',
            severity: 'review',
            claim: 'syntax-only',
            location: { line: 3, column: 3, endLine: 3, endColumn: 10, byteStart: 40, byteEnd: 47 },
          },
        ],
        parse: { ok: true, errors: [] },
        path: { classifications: ['source'], suppressed: false },
      },
    },
    ...overrides,
  };
}

describe('Rust source health schema', () => {
  it('accepts a complete artifact whose summary matches the body', () => {
    expect(validateRustHealthFinalArtifact(artifact())).toEqual([]);
  });

  it('accepts sidecar artifact before wrapper provenance is injected', () => {
    const value = artifact();
    delete value.meta.generated;
    delete value.meta.sidecar;
    delete value.meta.input;
    expect(validateRustHealthSidecarArtifact(value)).toEqual([]);
    expect(validateRustHealthFinalArtifact(value)).toContain('meta.generated invalid');
  });

  it('rejects summary counts that do not match artifact body', () => {
    const value = artifact({
      summary: { ...artifact().summary, signals: 99 },
    });
    expect(validateRustHealthFinalArtifact(value)).toContain(
      'summary.signals expected 2 but found 99',
    );
  });

  it('sorts file keys, skipped files, and signals deterministically', () => {
    const sorted = sortRustHealthArtifact({
      ...artifact(),
      skippedFiles: [
        { path: 'z.rs', reason: 'invalid-utf8' },
        { path: 'a.rs', reason: 'excluded-by-path-policy' },
      ],
      files: {
        'z.rs': {
          signals: [
            {
              kind: 'z',
              severity: 'review',
              claim: 'syntax-only',
              location: { line: 1, column: 10, endLine: 1, endColumn: 11, byteStart: 9, byteEnd: 10 },
            },
          ],
        },
        'a.rs': {
          signals: [
            {
              kind: 'late',
              severity: 'review',
              claim: 'syntax-only',
              location: { line: 1, column: 21, endLine: 1, endColumn: 25, byteStart: 20, byteEnd: 24 },
            },
            {
              kind: 'early',
              severity: 'review',
              claim: 'syntax-only',
              location: { line: 1, column: 2, endLine: 1, endColumn: 7, byteStart: 1, byteEnd: 6 },
            },
          ],
        },
      },
    });
    expect(Object.keys(sorted.files)).toEqual(['a.rs', 'z.rs']);
    expect(sorted.skippedFiles.map((file) => file.path)).toEqual(['a.rs', 'z.rs']);
    expect(sorted.files['a.rs'].signals.map((signal) => signal.kind)).toEqual([
      'early',
      'late',
    ]);
  });
});
```

- [ ] **Step 3: Run schema tests**

Run:

```bash
node node_modules/vitest/vitest.mjs run tests/rust-source-health-schema.test.mjs
```

Expected:

- 1 test file passes.
- Failures, if any, are about artifact body/summary behavior.

- [ ] **Step 4: Commit schema module**

Run:

```bash
git add _lib/rust-source-health-schema.mjs tests/rust-source-health-schema.test.mjs
git commit -m "Add Rust source health artifact schema"
```

## Task 3: Add The Node Wrapper And Lab CLI

**Files:**
- Create: `_lib/rust-source-health-runner.mjs`
- Create: `scripts/run-rust-source-health.mjs`
- Test: `tests/rust-source-health-runner.test.mjs`

- [ ] **Step 1: Create runner module**

Create `_lib/rust-source-health-runner.mjs` with these exported functions:

- `collectRustSourceHealthInput({ root, include, exclude })`
- `runRustSourceHealthSidecar({ binary, input, timeoutMs })`
- `buildFinalRustHealthArtifact({ sidecarArtifact, skippedFiles, pathPolicy, sidecarSourceCommit, binarySha256 })`
- `writeRustHealthArtifact({ output, artifact })`
- `runRustSourceHealth({ root, output, binary, sidecarSourceCommit, timeoutMs, threadCount, workerStackBytes })`

The runner must:

- collect only `.rs` files under root;
- skip paths whose normalized root-relative POSIX path has a `target` or `vendor` segment;
- not follow symlinked directories or symlinked files in M6;
- normalize every included path to a root-relative POSIX-slash path;
- reject any collected path that is absolute or contains `..` after normalization;
- hash raw bytes before decode;
- hash the sidecar binary at `binary` with SHA-256 and pass that value as `binarySha256` into `buildFinalRustHealthArtifact`;
- never accept a user-supplied binary SHA from the CLI;
- construct `input.runtime` with `threadCount` and `workerStackBytes`;
- default `workerStackBytes` to `16777216`;
- omit `threadCount` only when the caller did not provide it, letting Rayon choose CPU defaults;
- record invalid UTF-8 as skipped-file reason `invalid-utf8`;
- never pass invalid UTF-8 files to the sidecar;
- call the sidecar with stdin JSON only;
- treat invalid JSON stdout, timeout, and non-zero exit as hard failures;
- validate sidecar artifact with `validateRustHealthSidecarArtifact`;
- append wrapper-owned `skippedFiles`;
- inject `meta.generated`, `meta.sidecar.sourceCommit`, `meta.sidecar.binarySha256`, and `meta.input.pathPolicy`;
- recompute `artifact.summary = summarizeRustHealthArtifact(artifact)` after wrapper-owned `skippedFiles` and metadata are appended;
- sort deterministically;
- validate final artifact and summary invariants with `validateRustHealthFinalArtifact`;
- write with `atomicWrite`.

Use the existing `_lib/atomic-write.mjs` for writes and `crypto.createHash('sha256')` for hashing.

Use this path segment predicate, not raw substring checks:

```js
function hasPathSegment(relativePath, segment) {
  return (
    relativePath === segment ||
    relativePath.startsWith(`${segment}/`) ||
    relativePath.endsWith(`/${segment}`) ||
    relativePath.includes(`/${segment}/`)
  );
}

function isExcludedByPathPolicy(relativePath) {
  return hasPathSegment(relativePath, 'target') ||
    hasPathSegment(relativePath, 'vendor');
}
```

Use this final assembly order:

```js
const artifact = sortRustHealthArtifact({
  ...sidecarArtifact,
  skippedFiles: [...(sidecarArtifact.skippedFiles ?? []), ...skippedFiles],
  meta: {
    ...sidecarArtifact.meta,
    generated: new Date().toISOString(),
    sidecar: { sourceCommit: sidecarSourceCommit, binarySha256 },
    input: { pathPolicy },
  },
});

artifact.summary = summarizeRustHealthArtifact(artifact);

const problems = validateRustHealthFinalArtifact(artifact);
if (problems.length > 0) {
  throw new Error(`invalid final rust-health artifact: ${problems.join('; ')}`);
}
```

- [ ] **Step 2: Add CLI script**

Create `scripts/run-rust-source-health.mjs`:

```js
#!/usr/bin/env node
import { parseArgs } from 'node:util';

import { runRustSourceHealth } from '../_lib/rust-source-health-runner.mjs';

const { values } = parseArgs({
  options: {
    root: { type: 'string' },
    output: { type: 'string' },
    'rust-source-health-bin': { type: 'string' },
    'sidecar-source-commit': { type: 'string' },
    'timeout-ms': { type: 'string', default: '60000' },
    threads: { type: 'string' },
    'worker-stack-bytes': { type: 'string', default: '16777216' },
  },
});

if (!values.root) throw new Error('--root is required');
if (!values.output) throw new Error('--output is required');
if (!values['rust-source-health-bin']) throw new Error('--rust-source-health-bin is required');
if (!values['sidecar-source-commit']) throw new Error('--sidecar-source-commit is required');

const timeoutMs = Number(values['timeout-ms']);
if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
  throw new Error('--timeout-ms must be a positive number');
}
const threadCount = values.threads === undefined ? undefined : Number(values.threads);
if (
  threadCount !== undefined &&
  (!Number.isInteger(threadCount) || threadCount <= 0)
) {
  throw new Error('--threads must be a positive integer');
}
const workerStackBytes = Number(values['worker-stack-bytes']);
if (!Number.isInteger(workerStackBytes) || workerStackBytes < 16777216) {
  throw new Error('--worker-stack-bytes must be an integer >= 16777216');
}

const result = await runRustSourceHealth({
  root: values.root,
  output: values.output,
  binary: values['rust-source-health-bin'],
  sidecarSourceCommit: values['sidecar-source-commit'],
  timeoutMs,
  threadCount,
  workerStackBytes,
});

console.log(`[rust-source-health] wrote ${result.output}`);
console.log(`[rust-source-health] files=${result.artifact.summary.files} skipped=${result.artifact.summary.skippedFiles} signals=${result.artifact.summary.signals}`);
```

- [ ] **Step 3: Add wrapper behavior tests**

Create `tests/rust-source-health-runner.test.mjs` with fake sidecar helpers modeled after `tests/rust-topology-scanner-bridge.test.mjs`.

Required behavior cases:

- happy path:
  - temp root contains `src/lib.rs`;
  - fake sidecar echoes a valid artifact body;
  - fake sidecar request includes `runtime.workerStackBytes: 16777216`;
  - wrapper writes `rust-health.json`;
  - output has sidecar provenance, path policy, deterministic summary, wrapper-computed binary SHA-256, and no topology fields.
- invalid UTF-8:
  - temp root contains one valid `.rs` and one invalid `.rs` byte sequence;
  - fake sidecar receives only the valid file;
  - final artifact has skipped-file reason `invalid-utf8`;
  - summary skipped count matches skipped array length.
- path policy:
  - symlinked `.rs` files are not followed;
  - sidecar input paths are root-relative POSIX slash paths;
  - no absolute path or `..` path is accepted into sidecar input.
  - `target/generated.rs` and `vendor/lib.rs` are skipped;
  - `src/targeted.rs` and `src/vendor_name.rs` are not skipped.
- final assembly:
  - wrapper-owned skipped files are appended before summary recompute;
  - final validator requires `meta.generated`, `meta.sidecar`, and `meta.input.pathPolicy`.
- runtime policy:
  - `--threads 2` reaches sidecar input as `runtime.threadCount: 2`;
  - `--worker-stack-bytes 8388608` is rejected before running the sidecar.
- invalid sidecar output:
  - fake sidecar writes malformed JSON;
  - wrapper throws and does not write a partial artifact.
- timeout hard stop:
  - fake sidecar hangs;
  - wrapper throws timeout and does not write a partial artifact.
- boundary:
  - run M6 wrapper in a temp repo;
  - assert it writes only the requested `rust-health.json` path and does not create `topology.json`, SARIF, quorum, or prefer artifacts.

- [ ] **Step 4: Run wrapper tests**

Run:

```bash
node node_modules/vitest/vitest.mjs run tests/rust-source-health-schema.test.mjs tests/rust-source-health-runner.test.mjs
```

Expected:

- 2 test files pass.
- Invalid output and timeout tests fail loudly if the wrapper swallows sidecar errors.

- [ ] **Step 5: Commit wrapper and CLI**

Run:

```bash
git add _lib/rust-source-health-runner.mjs scripts/run-rust-source-health.mjs tests/rust-source-health-runner.test.mjs
git commit -m "Add Rust source health runner"
```

## Task 4: Integrate Verification And Docs

**Files:**
- Modify: `tests/README.md`
- Modify: `docs/lab/m6-rust-source-health-design-2026-06-16.md` only if implementation discovers a concrete contract correction.

- [ ] **Step 1: Regenerate test docs**

Run:

```bash
node scripts/update-test-doc.mjs
```

Expected:

- `tests/README.md` lists `rust-source-health-schema.test.mjs` and `rust-source-health-runner.test.mjs`.
- The descriptions mention behavior, not file/module existence.

- [ ] **Step 2: Run focused M6 verification**

Run:

```bash
cargo test --manifest-path experiments/rust-sidecar/rust-source-health/Cargo.toml
node node_modules/vitest/vitest.mjs run tests/rust-source-health-schema.test.mjs tests/rust-source-health-runner.test.mjs
node scripts/run-syntax-check.mjs
node scripts/update-test-doc.mjs --check
node scripts/check-doc-script-refs.mjs
git diff --check
```

Expected:

- Rust sidecar tests pass.
- Node behavior tests pass.
- Syntax, test docs, doc refs, and whitespace checks pass.

- [ ] **Step 3: Run one real local smoke**

Build the sidecar:

```bash
cargo build --release --manifest-path experiments/rust-sidecar/rust-source-health/Cargo.toml
```

Run the lab script against this repo:

```bash
node scripts/run-rust-source-health.mjs \
  --root . \
  --output baselines/m6-rust-source-health-local/rust-health.json \
  --rust-source-health-bin experiments/rust-sidecar/rust-source-health/target/release/lumin-rust-source-health \
  --sidecar-source-commit $(git rev-parse HEAD)
```

PowerShell equivalent:

```powershell
$commit = git rev-parse HEAD
node scripts/run-rust-source-health.mjs `
  --root . `
  --output baselines/m6-rust-source-health-local/rust-health.json `
  --rust-source-health-bin experiments/rust-sidecar/rust-source-health/target/release/lumin-rust-source-health.exe `
  --sidecar-source-commit $commit
```

Expected:

- `baselines/m6-rust-source-health-local/rust-health.json` exists.
- `summary.files` is greater than 0 if the repo contains Rust files.
- `summary.skippedFiles === skippedFiles.length`.
- No topology, prefer, quorum, SARIF, or markdown audit artifacts are created by this script.

- [ ] **Step 4: Add a short baseline note**

Create `baselines/m6-rust-source-health-local/README.md`:

````md
# M6 Rust Source Health Local Smoke

Date: 2026-06-16

Command:

```text
<paste exact command>
```

Result:

- artifact: `baselines/m6-rust-source-health-local/rust-health.json`
- parser: `ra_ap_syntax`
- parser edition policy: `fixed / 2021 / m6-policy-default`
- semantic claims: none
- topology/prefer/quorum artifacts changed: no
````

Replace `<paste exact command>` with the exact command used. Do not claim speed wins from this smoke.

- [ ] **Step 5: Commit verification docs**

Run:

```bash
git add tests/README.md baselines/m6-rust-source-health-local
git commit -m "Record Rust source health smoke"
```

## Task 5: Final Branch Verification

**Files:**
- No new planned files.

- [ ] **Step 1: Run final focused checks**

Run:

```bash
cargo test --manifest-path experiments/rust-sidecar/rust-source-health/Cargo.toml
cargo build --release --manifest-path experiments/rust-sidecar/rust-source-health/Cargo.toml
node node_modules/vitest/vitest.mjs run tests/rust-source-health-schema.test.mjs tests/rust-source-health-runner.test.mjs
node scripts/run-syntax-check.mjs
node scripts/update-test-doc.mjs --check
node scripts/check-doc-script-refs.mjs
git diff --check
```

Expected:

- All commands pass.

- [ ] **Step 2: Check boundary drift**

Run:

```bash
git diff --name-only origin/main...HEAD
```

Expected changed paths are limited to:

- `AGENTS.md`
- `canonical/index.md`
- `canonical/rust-debt.txt`
- `canonical/rust-source-health.md`
- `docs/lab/m6-rust-source-health-design-2026-06-16.md`
- `docs/superpowers/plans/2026-06-16-rust-source-health.md`
- `experiments/rust-sidecar/rust-source-health/**`
- `_lib/rust-source-health-schema.mjs`
- `_lib/rust-source-health-runner.mjs`
- `scripts/run-rust-source-health.mjs`
- `tests/rust-source-health-schema.test.mjs`
- `tests/rust-source-health-runner.test.mjs`
- `tests/README.md`
- `baselines/m6-rust-source-health-local/**`

If `measure-topology.mjs`, topology sidecar files, prefer/quorum files, stable skill files, or public package files appear, stop and inspect. Those are outside M6.

- [ ] **Step 3: Commit final cleanup if needed**

If final checks required doc or fixture updates:

```bash
git add <specific changed files>
git commit -m "Finalize Rust source health implementation"
```

If no files changed, do not create an empty commit.

## Self-Review Checklist

- Design coverage:
  - separate `rust-source-health` sidecar: Task 1
  - Node wrapper owns file policy/hashing/decode/write: Task 3
  - `rust-health.json` artifact: Tasks 2 and 3
  - fixed edition policy: Tasks 1 and 2
  - parser crate version, sidecar provenance, and per-file source hashes: Tasks 1 and 3
  - per-file Rayon parallelism with deterministic re-collection: Task 1
  - `ra_ap_syntax` hidden behind internal sidecar implementation, no public re-export: Task 1
  - syntax-only caveats: Tasks 1 and 4
  - invalid UTF-8 skipped-file evidence: Task 3
  - strict artifact structure validation, deterministic ordering, and summary invariants: Task 2
  - behavior and hard-stop tests: Tasks 1, 2, 3
  - no topology/prefer/quorum changes: Tasks 3 and 5
- Placeholder scan:
  - No task uses vague "handle it later" language.
  - No task asks for file/function existence tests.
- Type consistency:
  - `schemaVersion`, `policy.version`, `editionPolicy`, `editionSource`, `skippedFiles`, `unsafeBlocks`, and `unsafeFunctions` use the same names as the design.
  - `parser.version` means the `ra_ap_syntax` crate version, not the sidecar package version.
  - `files[path].sha256` is preserved from wrapper input to final artifact.
  - sidecar and final artifact validators intentionally differ: wrapper metadata is required only for final artifacts.
  - `target` and `vendor` checks are path segment checks, not substring checks.
  - Rust serde renames match the JSON contract.
  - Node invariant names match artifact summary fields.

## Execution Options

After this plan is reviewed, choose one:

1. **Subagent-Driven**: use `superpowers:subagent-driven-development`, one fresh worker per task, with review between tasks.
2. **Inline Execution**: use `superpowers:executing-plans`, execute tasks in this session with checkpoints.
