# Rust Pre-Write Name Cues Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (- [ ]) syntax for tracking.

**Goal:** Add an on-demand Rust pre-write name lookup that turns real syntax definition and impl-method evidence into TS/JS-compatible SAFE, REVIEW, and MUTED cue cards without enlarging the normal unified analyzer artifact.

**Architecture:** Keep one lumin-rust-analyzer package and add an explicit pre-write command. Build a borrowed candidate view over typed HealthResponse, then materialize only matched evidence into an owned advisory. Preserve the legacy analyzer command and artifact.

**Tech Stack:** Rust 2021, existing workspace serde, serde_json, anyhow, lumin-rust-common, lumin-rust-source-health, and Rust CI. Node is not run.

---

## Execution Rules

Repository AGENTS.md overrides generic TDD instructions. Implement the complete
production path first, then prove it through the real CLI. Do not use fake
analyzer responses, mock commands, scaffold-only tests, timeouts, repository
caps, or new policy values.

Before Rust edits:

~~~powershell
Get-Location
Get-Content -Raw .\AGENTS.md
Get-Content -Raw .\canonical\rust-debt.txt
Get-Content -Raw .\canonical\rust-source-health.md
git status --short --branch
~~~

Expected: experimental repository, branch rust-prewrite-impl-method-cues,
unrelated JS/TS changes untouched.

### Task 1: Lock Canonical Ownership

**Files:**

- Modify: canonical/rust-source-health.md
- Modify: canonical/pre-write-gate.md

- [ ] **Step 1: Document the source-health consumer boundary**

Add after the AST fact shape section:

~~~markdown
### Rust pre-write consumer

lumin-rust-analyzer pre-write may consume HealthResponse::files[*].ast in
memory to answer declared Rust name intents. The analyzer owns intent, lookup,
cue, and advisory policy. rust-source-health remains the owner of raw AST
extraction and path classification.

The normal unified artifact must not embed a repository-wide definition or
impl-method index. The pre-write consumer builds a borrowed view and serializes
only matched advisory evidence. Impl methods remain separate owner evidence
and must not be promoted into definition-lane SAFE cues.
~~~

- [ ] **Step 2: Document the Rust name-lane protocol**

Add to canonical/pre-write-gate.md:

~~~markdown
### Rust name-lane migration

lumin-rust-analyzer pre-write runs rust-source-health only. It does not run
Cargo metadata or Cargo check because the current cargo/rustc oracle can verify
diagnostics and clean build scope but cannot prove that an unreferenced name is
absent.

The Rust command accepts the five-key intent transport. Missing arrays default
with warnings and malformed present fields hard-stop. Non-empty shape, file,
dependency, and planned-type-escape lanes remain visible as unsupported.
taskId is typed; unknown extras hard-stop instead of entering a Value map.
~~~

- [ ] **Step 3: Check and commit canonical changes**

~~~powershell
git diff --check -- canonical\rust-source-health.md canonical\pre-write-gate.md
git add -- canonical\rust-source-health.md canonical\pre-write-gate.md
git commit -m "Define Rust pre-write name lane"
~~~

Expected: only the two canonical files are staged.

### Task 2: Implement The Complete Production Slice

**Files:**

- Modify: experiments/rust-main/lumin-rust-analyzer/src/cli.rs
- Modify: experiments/rust-main/lumin-rust-analyzer/src/main.rs
- Create: experiments/rust-main/lumin-rust-analyzer/src/prewrite.rs
- Create: experiments/rust-main/lumin-rust-analyzer/src/prewrite/intent.rs
- Create: experiments/rust-main/lumin-rust-analyzer/src/prewrite/index.rs
- Create: experiments/rust-main/lumin-rust-analyzer/src/prewrite/tokens.rs
- Create: experiments/rust-main/lumin-rust-analyzer/src/prewrite/lookup.rs
- Create: experiments/rust-main/lumin-rust-analyzer/src/prewrite/cues.rs
- Create: experiments/rust-main/lumin-rust-analyzer/src/prewrite/artifact.rs

- [ ] **Step 1: Add backward-compatible command parsing**

Keep existing analyzer Options and add:

~~~rust
#[derive(Debug)]
pub(crate) enum Command {
    Analyze(Options),
    PreWrite(PreWriteOptions),
}

#[derive(Debug)]
pub(crate) struct PreWriteOptions {
    pub(crate) root: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) source_commit: String,
    pub(crate) intent: PathBuf,
    pub(crate) thread_count: Option<usize>,
    pub(crate) worker_stack_bytes: usize,
}
~~~

Change parse_args to return Result<CliAction<Command>>. Parse pre-write only
when it is the first argument. Its flags are --root, --output,
--source-commit, --intent, --threads, and --worker-stack-bytes; output defaults
to stdout. Existing invocations retain current defaults and flags.

- [ ] **Step 2: Implement typed intent normalization**

In intent.rs use optional raw fields to detect absence:

~~~rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawIntent {
    names: Option<Vec<NameInput>>,
    shapes: Option<Vec<ShapeIntent>>,
    files: Option<Vec<String>>,
    dependencies: Option<Vec<DependencyInput>>,
    planned_type_escapes: Option<Vec<PlannedTypeEscape>>,
    task_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NameInput {
    Name(String),
    Declaration(NameDeclaration),
}

pub(super) fn load(path: &Path) -> Result<NormalizedIntent>;
~~~

Define typed NameDeclaration, ShapeIntent, DependencyInput,
DependencyDeclaration, PlannedTypeEscape, and EscapeKind. Port non-empty
string, shape hash, planned escape, alias, missing-array warning, and taskId
validation from _lib/pre-write-intent.mjs. Use usage_error for malformed
transport. Preserve valid unsupported-lane entries. Do not use
serde_json::Value or json!.

- [ ] **Step 3: Build a borrowed candidate index**

In index.rs define:

~~~rust
pub(super) enum CandidateLane { Definition, ImplMethod }

pub(super) struct Candidate<'a> {
    pub(super) lane: CandidateLane,
    pub(super) file: &'a str,
    pub(super) name: &'a str,
    pub(super) owner: Option<ImplOwner<'a>>,
    pub(super) location: &'a Location,
    pub(super) path: &'a PathMeta,
}

pub(super) struct CandidateIndex<'a> {
    pub(super) candidates: Vec<Candidate<'a>>,
}
~~~

CandidateIndex::from_health borrows syntax facts. Exclude a function definition
when its byte range equals an impl-method range in the same file, matching
TS/JS separation between defIndex and classMethodIndex. Sort by file, owner,
name, and byte start.

Materialize identities only for output:

~~~text
definition:       <file>::<name>
inherent method:  <file>::<target>#<name>
trait method:     <file>::<target> as <trait>#<name>
~~~

- [ ] **Step 4: Port token and lookup policy without tuning**

In tokens.rs define exactly:

~~~rust
pub(super) const TOKENIZER_VERSION: &str = "camel-snake-kebab-digit-v1";
pub(super) const TOKEN_POLICY_VERSION: &str = "prewrite-token-policy-v1";
pub(super) const WEAK_COMMON_TOKENS: [&str; 15] = [
    "add", "build", "check", "create", "delete", "get", "load", "make",
    "parse", "read", "return", "save", "set", "update", "write",
];
~~~

Port camel/snake/kebab/digit splitting, all aliases, guarded ies-to-y, and no
trailing-s stem from _lib/pre-write-token-policy.mjs.

In lookup.rs use only owner values:

~~~rust
const NEAR_NAME_MAX_LENGTH_DELTA: usize = 2;
const NEAR_NAME_SHARED_PREFIX_MIN: usize = 4;
const NEAR_NAME_MAX_DISTANCE: usize = 2;
const NEAR_NAME_MAX_RESULTS: usize = 5;
const SEMANTIC_HINT_MAX_RESULTS: usize = 5;
const SEMANTIC_HINT_MIN_SCORE: usize = 2;
~~~

Port capped Levenshtein, shared-prefix relaxation, token overlap, limits, and
sorting. Exact definitions suppress hints and emit identities. Exact impl
methods stay distance-zero REVIEW candidates. NOT_OBSERVED carries parse and
review-visible macro/cfg opaque taint, never an absence claim.

- [ ] **Step 5: Project typed cue cards**

In cues.rs define:

~~~rust
pub(super) enum CueTier {
    #[serde(rename = "SAFE_CUE")] Safe,
    #[serde(rename = "AGENT_REVIEW_CUE")] AgentReview,
    #[serde(rename = "MUTED_CUE")] Muted,
}

#[serde(rename_all = "kebab-case")]
pub(super) enum EvidenceLane {
    ExactSymbol,
    ImplMethodName,
    NearName,
    IntentToken,
}
~~~

Exact definitions emit SAFE with safeMeaning claim-only and the TS/JS
notSafeFor list. Impl methods and heuristic hints emit REVIEW. A
source-health-suppressed path emits MUTED while preserving original tier and
PathClassification. Add no path policy.

- [ ] **Step 6: Build an owned advisory and validate it**

artifact.rs owns final rows because prewrite::run cannot return references to
local intent or syntax values:

~~~rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreWriteArtifact {
    schema_version: &'static str,
    policy_version: &'static str,
    meta: PreWriteMeta,
    intent: NormalizedIntent,
    intent_warnings: Vec<IntentWarning>,
    coverage: IntentLaneCoverage,
    lookups: Vec<NameLookup>,
    cue_cards: Vec<CueCard>,
    suppressed_cues: Vec<SuppressedCue>,
}
~~~

Use rust-pre-write.v1. Record source-health parser/policy/signal-policy and
token-policy provenance; omit timestamps and repository-wide indexes. Coverage
is ran for names and unsupported/not-requested for other lanes.

validate_contract hard-stops if impl evidence is SAFE, SAFE lacks exact
definition evidence, MUTED loses original tier/path evidence, lookup names are
not in normalized intent, or non-empty unsupported input reports completed
coverage.

- [ ] **Step 7: Wire syntax-only execution and output**

prewrite.rs owns:

~~~rust
pub(crate) fn run(options: &PreWriteOptions) -> Result<PreWriteArtifact> {
    let root = canonical_existing_dir_usage(&options.root, "--root")?;
    let intent = intent::load(&options.intent)?;
    let syntax = analyze_root(RustSourceHealthOptions {
        root,
        source_commit: options.source_commit.clone(),
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
    })?;
    artifact::build(intent, &syntax)
}
~~~

Dispatch Analyze to the unchanged analyzer and PreWrite to this path. Reuse
atomic JSON output/pretty stdout. Pre-write must not call run_oracle, Cargo
metadata, calibration, or oracle targeting.

- [ ] **Step 8: Compile, inspect help, and commit production code**

~~~powershell
cargo check --locked --manifest-path experiments\Cargo.toml -p lumin-rust-analyzer
cargo run --quiet --locked --manifest-path experiments\Cargo.toml -p lumin-rust-analyzer -- pre-write --help
git diff --check -- experiments\rust-main\lumin-rust-analyzer\src
git add -- experiments\rust-main\lumin-rust-analyzer\src\cli.rs experiments\rust-main\lumin-rust-analyzer\src\main.rs experiments\rust-main\lumin-rust-analyzer\src\prewrite.rs experiments\rust-main\lumin-rust-analyzer\src\prewrite
git commit -m "Add Rust pre-write name cues"
~~~

Expected: help has no Cargo/timeout flag and no placeholder output exists.

### Task 3: Prove Real CLI Behavior

**Files:**

- Create: experiments/rust-main/lumin-rust-analyzer/tests/integration/prewrite.rs
- Create: experiments/rust-main/lumin-rust-analyzer/tests/integration/prewrite/cues.rs
- Create: experiments/rust-main/lumin-rust-analyzer/tests/integration/prewrite/coverage.rs
- Create: experiments/rust-main/lumin-rust-analyzer/tests/integration/prewrite/errors.rs
- Create: experiments/rust-main/lumin-rust-analyzer/tests/support/prewrite.rs
- Modify: experiments/rust-main/lumin-rust-analyzer/tests/integration.rs
- Modify: experiments/rust-main/lumin-rust-analyzer/tests/support/mod.rs

- [ ] **Step 1: Create a real Rust repository helper**

Write actual source with load_task, EventDispatcher::handle_delete, a custom
macro, and a non-test cfg gate. Write tests/helper.rs with
TestDispatcher::handle_delete. The helper writes intent JSON, invokes the real
compiled pre-write command, and reads output; it never constructs analyzer
evidence.

- [ ] **Step 2: Verify tier behavior**

Assert:

~~~text
load_task -> SAFE_CUE / exact-symbol / claim-only
handle_bulk_delete -> EventDispatcher#handle_delete REVIEW_CUE
handle_delete -> exact impl-method REVIEW_CUE at distance 0
no impl-method candidate appears in a SAFE_CUE
tests/helper.rs candidate -> MUTED_CUE preserving REVIEW and test classification
~~~

- [ ] **Step 3: Verify coverage and hard-stops**

Assert an unobserved name carries macro/cfg opaque taintedBy evidence and no
absence claim. Assert non-empty shape/file input is preserved with unsupported
coverage. Assert missing arrays warn. Assert exit 2 and no artifact for
malformed JSON, non-array names, empty name, empty taskId, and unknown field.

- [ ] **Step 4: Verify deterministic compact output**

Run the same command twice and compare parsed JSON equality. Assert no timestamp
and no repository-wide index. Run legacy metadata-only analysis and assert no
pre-write field appears.

- [ ] **Step 5: Run and commit focused tests**

~~~powershell
cargo test --locked --manifest-path experiments\Cargo.toml -p lumin-rust-analyzer --test integration prewrite --profile ci-test
git add -- experiments\rust-main\lumin-rust-analyzer\tests\integration.rs experiments\rust-main\lumin-rust-analyzer\tests\integration\prewrite.rs experiments\rust-main\lumin-rust-analyzer\tests\integration\prewrite experiments\rust-main\lumin-rust-analyzer\tests\support\mod.rs experiments\rust-main\lumin-rust-analyzer\tests\support\prewrite.rs
git commit -m "Prove Rust pre-write cue behavior"
~~~

### Task 4: Full Rust Verification, PR, CI, And Merge

**Files:** Verify branch-owned files only; preserve unrelated JS/TS changes.

- [ ] **Step 1: Run Rust quality gates**

~~~powershell
cargo fmt --manifest-path experiments\Cargo.toml --all --check
cargo clippy --locked --manifest-path experiments\Cargo.toml --workspace --all-targets -- -D warnings
cargo test --locked --manifest-path experiments\Cargo.toml --workspace --profile ci-test
~~~

- [ ] **Step 2: Scan for arbitrary or dynamic logic**

~~~powershell
rg -n "timeout|timed_out|elapsed.*cap|file_cap|package_cap|repository_cap" experiments\rust-main experiments\rust-sidecar
rg -n "serde_json::Value|json!\(" experiments\rust-main\lumin-rust-analyzer\src\prewrite.rs experiments\rust-main\lumin-rust-analyzer\src\prewrite
rg -n "NEAR_NAME_|SEMANTIC_HINT_|WEAK_COMMON_TOKENS|SAFE_CUE|AGENT_REVIEW_CUE|MUTED_CUE" _lib experiments\rust-main\lumin-rust-analyzer\src\prewrite
~~~

Expected: no new timeout/cap or dynamic JSON; policy values map to TS/JS or
source-health owners.

- [ ] **Step 3: Audit scope**

~~~powershell
git diff main...HEAD --check
git diff main...HEAD --stat
git diff main...HEAD --name-only
git status --short --branch
~~~

Expected: canonical, design/plan, and analyzer Rust/test files only.

- [ ] **Step 4: Open PR and require Rust CI**

PR body records owner mapping, syntax-only authority, tier behavior,
unsupported coverage, artifact size discipline, and Rust verification. Require
Detect changed surfaces and Test (Rust) success. Inspect review comments against
current code before edits and rerun Rust checks after fixes.

- [ ] **Step 5: Rebase-merge and fast-forward**

After clean CI/review, rebase-merge with expected head SHA, then:

~~~powershell
git fetch origin main
git switch main
git merge --ff-only origin/main
git status --short --branch
~~~

Expected: local main equals origin/main; unrelated JS/TS changes survive.
