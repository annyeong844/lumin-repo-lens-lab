# Performance Test Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Inventory the performance, incremental, scanner, and measurement test family into the Lumin wiki without moving test files or changing analyzer behavior.

**Architecture:** This is a documentation-only slice. `docs/lumin-wiki/workstreams/performance.md` becomes the human-readable map of performance suites, measured counters, cache identity contracts, and scanner/probe boundaries. `docs/lumin-wiki/log.md` records the inventory event, and `docs/spec/lumin-work-tracker.md` updates WT-24's current and next states.

**Tech Stack:** Markdown docs, existing generated `tests/README.md`, existing performance/incremental/scanner test suites.

---

### Task 1: Add Performance Suite Inventory

**Files:**
- Modify: `docs/lumin-wiki/workstreams/performance.md`

- [ ] **Step 1: Replace the current "Tests And Evidence To Understand First" section with a risk-based inventory**

Use this exact section body:

```markdown
## Test Inventory

| Suite Or Evidence | Risk Type | Protected Invariant | Edge Case Or Negative Guard |
|---|---|---|---|
| `tests/test-audit-repo.mjs` | producer-performance artifact | Audit runs emit `producer-performance.json` with producer timings, artifact sizes, JSON read/parse counters, and memory snapshots. | Performance metadata must remain evidence, not recommendation prose. |
| `tests/test-classify-performance-metadata.mjs` | producer metadata | Classification performance metadata and text-zero shortcuts stay explicit. | Fast paths must still expose when a shortcut was used. |
| `tests/test-js-module-edge-scanner.mjs` | scanner equivalence | Tokenizer-state scanner fixtures preserve module-edge equivalence and fallback discipline. | Fake imports in comments, strings, regexes, JSX text, or templates must not become edges. |
| `tests/test-symbol-graph-incremental.mjs` | strict incremental cache | Warm symbol graph runs reuse unchanged file facts while changed/deleted files refresh correctly. | Legacy caches missing newer facts must invalidate rather than silently reuse stale symbol evidence. |
| `tests/test-shape-index-incremental.mjs` | strict incremental cache | Warm shape-index runs match cold facts and handle changed/deleted files. | Deleted or changed files must not leave stale shape facts. |
| `tests/test-function-clone-incremental.mjs` | strict incremental cache | Warm function-clone runs match cold facts and handle changed/deleted files. | Function clone caches must not preserve stale fingerprints after edits or deletes. |
| `tests/test-any-inventory-incremental.mjs` | strict incremental cache | Any-inventory cold/warm equivalence holds across changed, deleted, and scan-option changes. | Production/test scope changes must prevent stale public artifacts. |
| `tests/test-incremental-cache-store.mjs` | cache store contract | Incremental cache schema, current-hash reuse, and malformed-cache fallback are stable. | Malformed cache files must degrade safely instead of crashing or reusing invalid facts. |
| `tests/test-incremental-snapshot.mjs` | repo snapshot identity | Content hashes, unreadable file visibility, and snapshot identity stay deterministic. | Unreadable or changed files must be visible to cache identity, not hidden. |
| `tests/test-incremental.mjs` | file hash/stat fast path | File-hash cache and stat-first-cut fast path preserve correctness. | Fast stat checks must not skip content changes that matter to artifacts. |
| `tests/test-audit-repo-symbol-incremental.mjs` | orchestrator wiring | Audit orchestrator forwards strict incremental flags to `build-symbol-graph`. | CLI cache flags must not be ignored by the producer path. |
| `tests/test-function-clone-audit-forwarding.mjs` | orchestrator wiring | Audit orchestrator forwards incremental flags to the function-clone producer. | Function-clone cache behavior must be controlled by the same public audit flags. |
| `tests/test-post-write-incremental.mjs` | lifecycle cache boundary | Post-write after-snapshot incremental forwarding preserves immutable pre-write baseline semantics. | Post-write must not mutate or reinterpret the pre-write baseline cache. |
| `tests/test-build-function-clone-index.mjs` | producer artifact shape | Function clone producer emits clone cue artifacts and exact grouping. | Small exact clones with identical hashes must group without widening structure-group thresholds. |
| `tests/test-build-shape-index.mjs` | producer artifact shape | Shape-index producer emits grouping, diagnostics, and scan-scope metadata. | Unsupported or out-of-scope shapes must stay diagnostic, not fake comparable facts. |
| `tests/test-tsconfig-paths-scoped.mjs` | resolver cache counters | Scoped tsconfig/baseUrl resolution emits cache/probe counters while preserving scoped correctness. | Cache hits must not change per-importer resolution identity. |
| `tests/test-wildcard.mjs` | resolver cache overlap | Exports wildcard subpath resolution remains deterministic after wildcard alias caching. | Cached wildcard results must preserve unresolved/internal sentinels and generated virtual surfaces. |
| `docs/spec/lumin-fused-safer-graph.md` | performance architecture spec | Parser avoidance, scanner fallback, edge hashes, and fact fusion stay measurement-gated. | Scanner/fusion work should not land as broad behavior change without equivalence fixtures and counters. |
| `docs/lab/wt18-*.md` | lab evidence | cal.diy and public-package timing notes preserve measured before/after evidence. | Single-run wall time is lab evidence, not a universal performance claim. |
```

- [ ] **Step 2: Add a reform target section below the existing reform direction**

Use this exact section:

```markdown
## Reform Targets

- Keep performance tests centered on counters and cache identity, not bare
  single-run wall time.
- Separate correctness overlap from performance overlap: resolver cache tests
  also belong to resolver correctness, but performance inventory tracks the
  counter/cache invariant.
- Compare incremental suites for shared cold/warm fixture helpers before moving
  files.
- Keep scanner tests in shadow/equivalence style before expanding accepted
  syntax.
- Do not start scheduler, Rust/rayon, or full `js-facts` fusion work without a
  narrower measurement spec.
```

- [ ] **Step 3: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/workstreams/performance.md
```

Expected: the performance wiki page now has a `Test Inventory` table and `Reform Targets` section.

### Task 2: Record The Wiki Update

**Files:**
- Modify: `docs/lumin-wiki/log.md`

- [ ] **Step 1: Append a log entry**

Append this exact entry after the deadness inventory entry:

```markdown

## [2026-05-12] inventory | performance test family

Added a risk-based inventory for performance, incremental cache, scanner,
producer measurement, and resolver-cache-overlap suites. The inventory separates
measurement evidence from correctness claims before any test-file movement.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/log.md
```

Expected: one new chronological performance inventory entry appears.

### Task 3: Update WT-24 State

**Files:**
- Modify: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Update the WT-24 row**

Replace the WT-24 `Current State` sentence with:

```markdown
`docs/lumin-wiki/` establishes a maintainer synthesis layer with workstream pages, evidence concepts, and test reform rules. `docs/lumin-wiki/workstreams/pre-write.md`, `docs/lumin-wiki/workstreams/resolver.md`, `docs/lumin-wiki/workstreams/deadness.md`, and `docs/lumin-wiki/workstreams/performance.md` now contain the first risk-based suite inventories, including protected invariants and edge-case/negative guards. `docs/superpowers/specs/2026-05-12-lumin-wiki-test-reform-design.md` records the documentation-only scaffold and the rule that future TDD should fail on concrete edge cases rather than missing helpers.
```

Replace the WT-24 `Next Small PR` sentence with:

```markdown
Inventory the public-package test family, then compare duplicated fixture shapes across inventoried workstreams before moving any files.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/spec/lumin-work-tracker.md
```

Expected: WT-24 now says pre-write, resolver, deadness, and performance inventories exist and public-package inventory is next.

### Task 4: Verify Documentation Slice

**Files:**
- Read: `docs/lumin-wiki/workstreams/performance.md`
- Read: `docs/lumin-wiki/log.md`
- Read: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Check for placeholders**

Run:

```powershell
rg "TBD|TODO|PLACEHOLDER|FIXME|\\?\\?" docs/lumin-wiki docs/spec/lumin-work-tracker.md
```

Expected: exit code 1 with no matches.

- [ ] **Step 2: Run lightweight doc checks**

Run:

```powershell
git diff --check
npm run check:test-doc
npm run check:doc-script-refs
```

Expected: all commands exit 0.

- [ ] **Step 3: Commit**

Run:

```powershell
git add -- docs/lumin-wiki/workstreams/performance.md docs/lumin-wiki/log.md docs/spec/lumin-work-tracker.md docs/superpowers/plans/2026-05-12-performance-test-inventory.md
git commit -m "Inventory performance tests in Lumin wiki"
```

Expected: one commit containing only documentation changes.
