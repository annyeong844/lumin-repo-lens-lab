# Rust Module Reachability Producer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `module-reachability.json` artifact construction into `lumin-audit-core` while keeping JS as a thin compatibility wrapper.

**Architecture:** JS keeps artifact reads, artifact write, CLI compatibility, and console output. Rust receives already-produced `symbols.json` and `entry-surface.json` facts, then owns known-file collection, bounded BFS, unreachable SCC detection, deterministic sorting, and summary projection. No JS/TS resolver, parser, or entry-surface semantics move in this slice.

**Tech Stack:** Rust `lumin-audit-core`, `serde`, `serde_json`, `anyhow`; existing JS wrapper bridge through `_lib/audit-core.mjs`; focused Node smoke tests only.

---

## Files

- Create: `experiments/rust-main/lumin-audit-core/src/module_reachability.rs`
- Create: `experiments/rust-main/lumin-audit-core/src/cli/module_reachability.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/lib.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/mod.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/usage.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs`
- Modify: `experiments/rust-main/lumin-audit-core/tests/orchestration_plan.rs`
- Modify: `_lib/module-reachability.mjs`
- Modify: `build-module-reachability.mjs`
- Modify: `_lib/audit-core.mjs`
- Modify: `scripts/build-skill.mjs`
- Modify: `canonical/audit-core.md`
- Generated sync: `skills/lumin-repo-lens-lab/**` via `npm run build:skill`

Do not migrate `entry-surface.json`, `symbols.json`, or resolver logic.

---

### Task 1: Add Rust Module Reachability Artifact Builder

**Files:**
- Create: `experiments/rust-main/lumin-audit-core/src/module_reachability.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/lib.rs`

- [ ] **Step 1: Add request/artifact types**

Create `module_reachability.rs` with constants:

```rust
pub const MODULE_REACHABILITY_SCHEMA_VERSION: &str = "module-reachability.v1";
pub const MODULE_REACHABILITY_REQUEST_SCHEMA_VERSION: &str =
    "lumin-module-reachability-producer-request.v1";
pub const DEFAULT_MAX_FILES_VISITED: usize = 200_000;
pub const DEFAULT_MAX_EDGES_VISITED: usize = 400_000;
```

Request shape:

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleReachabilityRequest {
    pub schema_version: String,
    pub root: String,
    pub symbols: SymbolsInput,
    pub entry_surface: EntrySurfaceInput,
    #[serde(default = "default_max_files_visited")]
    pub max_files_visited: usize,
    #[serde(default = "default_max_edges_visited")]
    pub max_edges_visited: usize,
}
```

Use nested defaults for `defIndex`, `reExportsByFile`, `resolvedInternalEdges`,
`entryFiles`, `globalCompleteness`, and `completenessBySubmodule` to match the
checked JS helper.

- [ ] **Step 2: Implement known-file collection**

Known files must include:

```text
symbols.defIndex keys
symbols.reExportsByFile keys
symbols.resolvedInternalEdges[].from
symbols.resolvedInternalEdges[].to
entrySurface.entryFiles[]
```

Normalize graph paths with `replace('\\', "/")`. Ignore empty edge endpoints.
Do not reject absolute-looking artifact paths.

- [ ] **Step 3: Implement adjacency and bounded BFS**

Build two adjacency maps:

```text
runtime graph: resolvedInternalEdges excluding typeOnly === true
type graph: all resolvedInternalEdges
```

Deduplicate and sort adjacency targets before BFS. Preserve entry seed insertion
order after normalization and deduplication. Match JS counter semantics:

```text
seed insertion: check maxFilesVisited before adding the next seed
adjacency loop: increment edgesVisited, then check edgesVisited > maxEdgesVisited
new target: check maxFilesVisited before adding the target
edge limit wins over file limit inside the adjacency loop
```

- [ ] **Step 4: Implement unreachable and bounded-out projection**

If neither BFS is bounded, known files outside `reachableFiles` become
`unreachableFiles`. If any BFS is bounded, known files outside `reachableFiles`
become `boundedOutFiles`, and SCC evidence must be empty.

- [ ] **Step 5: Implement runtime SCC projection**

Use deterministic Kosaraju or Tarjan implementation. Emit only runtime SCCs
where:

```text
component size > 1
all files are in unreachableFiles
traversal was not bounded
```

Sort component members lexicographically. Sort emitted components by descending
size, then first file after member sorting. Single-file self-cycles are not
emitted.

- [ ] **Step 6: Add Rust unit tests**

Cover:

```text
runtime vs type-only edge divergence
entry file absent from symbols remains reachable seed
reExportsByFile contributes known files only, not adjacency
bounded file limit sends unvisited files to boundedOutFiles
bounded edge limit uses JS off-by-one semantics
duplicate edges are deduped before edge counting
backslash paths normalize to slash paths
absolute-looking paths are preserved
empty edge endpoints are ignored
unreachable runtime SCC ordering
self-loop is not emitted as SCC
```

- [ ] **Step 7: Export the module**

Add to `experiments/rust-main/lumin-audit-core/src/lib.rs`:

```rust
pub mod module_reachability;
```

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test module_reachability
```

Expected: module reachability tests pass.

---

### Task 2: Add CLI Command

**Files:**
- Create: `experiments/rust-main/lumin-audit-core/src/cli/module_reachability.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/mod.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/cli/usage.rs`

- [ ] **Step 1: Add CLI runner**

Create `cli/module_reachability.rs` following the
`framework_resource_surfaces.rs` and `unused_deps.rs` pattern:

```rust
pub(super) fn run_module_reachability_artifact(args: Vec<String>) -> Result<()> {
    // parse --input and optional --result-output
    // read JSON
    // deserialize ModuleReachabilityRequest
    // build artifact
    // write stdout or result file
}
```

Unknown arguments hard-stop. Missing `--input` hard-stops with:

```text
module-reachability-artifact: missing --input <path|->
```

- [ ] **Step 2: Wire CLI dispatch and usage**

Add subcommand:

```text
lumin-audit-core module-reachability-artifact --input <path|-> [--result-output <path>]
```

- [ ] **Step 3: Add CLI result-file test**

Add a test in `module_reachability.rs` or an integration test that invokes the
CLI with `--result-output` and verifies the output file contains
`schemaVersion: "module-reachability.v1"` under `meta`.

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test module_reachability
```

Expected: CLI and library tests pass.

---

### Task 3: Thin JS Helper And Producer Wrapper

**Files:**
- Modify: `_lib/module-reachability.mjs`
- Modify: `build-module-reachability.mjs`
- Modify: `_lib/audit-core.mjs`
- Modify: `scripts/build-skill.mjs`

- [ ] **Step 1: Convert `_lib/module-reachability.mjs` to wrapper**

Remove JS graph traversal, SCC detection, bounded traversal decisions, and
summary count math. Keep only:

```js
import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const MODULE_REACHABILITY_REQUEST_SCHEMA_VERSION =
  'lumin-module-reachability-producer-request.v1';

export function buildModuleReachabilityArtifact(request) {
  return runAuditCoreJsonResultFile(
    ['module-reachability-artifact', '--input', '-'],
    'module-reachability-artifact',
    {
      input: JSON.stringify({
        schemaVersion: MODULE_REACHABILITY_REQUEST_SCHEMA_VERSION,
        root: request.root ?? process.cwd(),
        symbols: request.symbolsData ?? request.symbols ?? {},
        entrySurface: request.entrySurface ?? {},
        maxFilesVisited: request.maxFilesVisited,
        maxEdgesVisited: request.maxEdgesVisited,
      }),
    },
  );
}
```

Do not keep fallback graph logic.

- [ ] **Step 2: Keep `build-module-reachability.mjs` as file wrapper**

Keep existing artifact reads, positive integer parsing, artifact write, and
console summary. Adjust request field names only if needed for the thin helper.

- [ ] **Step 3: Add audit-core contract probe**

Add `module-reachability-artifact` missing-input probe to `_lib/audit-core.mjs`
and `scripts/build-skill.mjs`:

```js
['module-reachability-artifact'],
'module-reachability-artifact: missing --input <path|->'
```

- [ ] **Step 4: Run focused Node test**

Run:

```powershell
node tests/test-module-reachability.mjs
```

Expected: existing module reachability product checks pass.

---

### Task 4: Update Ownership And Package

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs`
- Modify: `experiments/rust-main/lumin-audit-core/tests/orchestration_plan.rs`
- Modify: `canonical/audit-core.md`
- Generated sync: `skills/lumin-repo-lens-lab/**`

- [ ] **Step 1: Mark producer owner as Rust**

In `orchestration_plan.rs`, change `build-module-reachability.mjs` from
`ProducerOwner::JsMjs` to `ProducerOwner::Rust`. Preserve script, phase,
preconditions, and skip reason.

- [ ] **Step 2: Update plan tests**

Update Rust-owned step counts and add an assertion that
`build-module-reachability.mjs` has `producerOwner: "rust"`.

- [ ] **Step 3: Update canonical owner map**

Add `module_reachability.rs` to `canonical/audit-core.md` as owner of
`module-reachability.json` artifact construction from already-produced
`symbols.json` and `entry-surface.json`. Its “must not own” list must include
JS/TS module resolution, source parsing, entry-surface discovery, and safe-delete
claims.

- [ ] **Step 4: Rebuild skill package**

Run:

```powershell
npm run build:skill
```

Expected: generated skill package includes the updated binary, JS wrappers, and
fallback Rust source.

- [ ] **Step 5: Check packaged fallback**

Run:

```powershell
cargo check --manifest-path skills/lumin-repo-lens-lab/_engine/rust/Cargo.toml --locked -p lumin-audit-core
```

Expected: package fallback workspace compiles.

Remove any generated `skills/lumin-repo-lens-lab/_engine/rust/target` directory
before committing.

---

### Task 5: Verification And Commit

**Files:**
- All touched files

- [ ] **Step 1: Format and lint**

Run:

```powershell
cargo fmt --all
cargo clippy --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --all-targets -- -D warnings
```

Expected: clippy exits 0.

- [ ] **Step 2: Run focused behavior checks**

Run:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test module_reachability
node tests/test-module-reachability.mjs
npm run test:audit-runtime-gate
git diff --check
```

Expected: all commands exit 0.

Do not run the legacy Node umbrella/Vitest suite for this slice.

- [ ] **Step 3: Review staged diff**

Check:

```powershell
git status -sb
git diff --stat
git diff --cached --stat
```

Confirm no generated `target/` directory is staged.

- [ ] **Step 4: Commit**

Use:

```powershell
git add <touched files>
git commit -m "Migrate module reachability to audit-core"
```

Expected: one implementation commit after the spec commits.
