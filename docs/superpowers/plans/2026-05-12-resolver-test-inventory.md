# Resolver Test Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Inventory the resolver test family into the Lumin wiki without moving test files or changing analyzer behavior.

**Architecture:** This is a documentation-only slice. `docs/lumin-wiki/workstreams/resolver.md` becomes the human-readable map of resolver suites, protected invariants, and negative guards. `docs/lumin-wiki/log.md` records the inventory event, and `docs/spec/lumin-work-tracker.md` updates WT-24's current and next states.

**Tech Stack:** Markdown docs, existing generated `tests/README.md`, existing resolver/generator/dynamic-module test suites.

---

### Task 1: Add Resolver Suite Inventory

**Files:**
- Modify: `docs/lumin-wiki/workstreams/resolver.md`

- [ ] **Step 1: Replace the current "Tests To Understand First" section with a risk-based inventory**

Use this exact section body:

```markdown
## Test Inventory

| Suite | Risk Type | Protected Invariant | Edge Case Or Negative Guard |
|---|---|---|---|
| `tests/test-resolver-diagnostics-artifacts.mjs` | artifact shape | Resolver capabilities and per-run diagnostics serialize separately and deterministically. | Static capability metadata must not be confused with repo-specific unresolved evidence. |
| `tests/test-resolver-blind-zone-relevance.mjs` | blind-zone scoping | Resolver blind zones block only candidate-relevant absence claims. | An unrelated unresolved import must not become a repo-global blocker. |
| `tests/test-resolver-paths.mjs` | resolver regression | Core path resolution handles historical resolver edge cases. | Relative or alias misses must not silently fall through to external when they are internal-looking. |
| `tests/test-tsconfig-paths-scoped.mjs` | monorepo resolver regression | Per-scope `tsconfig` paths and baseUrl aliases resolve against the importing package/app. | The same `@/*` specifier may resolve to different files in different app scopes; missing local targets stay unresolved internal. |
| `tests/test-hash-imports.mjs` | Node package imports | Node `#imports` exact, wildcard, and suffix wildcard aliases resolve only when supported. | Unsupported imports maps must not degrade to external fallback or fake edges. |
| `tests/test-node-imports-unsupported.mjs` | unsupported-family diagnostic | Unsupported Node `#imports` surfaces emit a named family and no concrete edge. | Condition-profile ambiguity and unsupported imports stay diagnostic-only. |
| `tests/test-output-source-layout-diagnostics.mjs` | unsupported-family diagnostic | Non-standard package output/source layouts emit `output-to-source-mapping` diagnostics. | Compiled output paths without supported source mapping do not become deadness evidence or fake resolved edges. |
| `tests/test-import-meta-glob-diagnostics.mjs` | dynamic-module diagnostic | Unsupported `import.meta.glob` calls are recorded as dynamic-module blind zones. | Literal globs create no concrete graph edge until scan-policy-aware expansion exists. |
| `tests/test-generated-artifact-evidence.mjs` | generated evidence policy | Generated-artifact classification requires strong package/surface evidence. | Package name, dependency, or short path token alone must not promote a miss to generated evidence. |
| `tests/test-generated-blind-zone-relevance.mjs` | blind-zone scoping | Generated artifact blind zones block only relevant SAFE_FIX promotion. | Unrelated generated misses remain confidence limitations, not global blockers. |
| `tests/test-generated-consumer-blind-zones.mjs` | artifact shape | Missing or excluded generated consumer surfaces are listed in symbols and resolver summaries. | Generated consumers can block absence claims without being treated as observed source consumers. |
| `tests/test-generated-virtual-surface.mjs` | virtual surface contract | Supported virtual generated surfaces expose conservative import/export facts. | Virtual facts must not claim runtime equivalence or body/call evidence. |
| `tests/test-workspace-no-exports.mjs` | workspace package resolver | Workspace packages without `exports` still resolve supported legacy/source-direct subpaths. | The fix is additive; truly unused siblings remain dead and missing generated typings remain unresolved. |
| `tests/test-wildcard.mjs` | package exports wildcard | Package `exports` wildcard subpaths resolve through supported source/output mappings. | Missing wildcard targets stay unresolved internal, not external. |
| `tests/test-resolved-edges.mjs` | graph artifact shape | Resolved internal file-level edges are emitted for downstream reachability. | Only concrete resolved edges enter the graph artifact. |
| `tests/test-dynamic-import.mjs` | topology resolver behavior | Literal dynamic imports contribute topology edges. | Dynamic behavior must stay distinct from unsupported dynamic-module surfaces such as `import.meta.glob`. |
```

- [ ] **Step 2: Add a reform target section below the existing reform direction**

Use this exact section:

```markdown
## Reform Targets

- Split resolver tests by output level: `resolved`, `candidate`, `unsupported`,
  `external`, and `unresolved_internal`.
- Keep every unsupported-family fixture paired with a "no fake graph edge"
  assertion.
- Compare generated, output-to-source, and dynamic-module fixtures for shared
  artifact-shape helpers before moving files.
- Keep condition-profile and workspace-scope cases separate; they protect
  different resolver identities.
- Do not widen resolver heuristics from a single fixture without adding a named
  capability or unsupported-family policy.
```

- [ ] **Step 3: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/workstreams/resolver.md
```

Expected: the resolver wiki page now has a `Test Inventory` table and `Reform Targets` section.

### Task 2: Record The Wiki Update

**Files:**
- Modify: `docs/lumin-wiki/log.md`

- [ ] **Step 1: Append a log entry**

Append this exact entry after the pre-write inventory entry:

```markdown

## [2026-05-12] inventory | resolver test family

Added a risk-based inventory for resolver-related suites. The inventory names
resolved, candidate, unsupported-family, generated, output-layout, dynamic
module, and workspace resolver invariants before any test-file movement.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/log.md
```

Expected: one new chronological resolver inventory entry appears.

### Task 3: Update WT-24 State

**Files:**
- Modify: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Update the WT-24 row**

Replace the WT-24 `Current State` sentence with:

```markdown
`docs/lumin-wiki/` establishes a maintainer synthesis layer with workstream pages, evidence concepts, and test reform rules. `docs/lumin-wiki/workstreams/pre-write.md` and `docs/lumin-wiki/workstreams/resolver.md` now contain the first risk-based suite inventories, including protected invariants and edge-case/negative guards. `docs/superpowers/specs/2026-05-12-lumin-wiki-test-reform-design.md` records the documentation-only scaffold and the rule that future TDD should fail on concrete edge cases rather than missing helpers.
```

Replace the WT-24 `Next Small PR` sentence with:

```markdown
Compare duplicated fixture shapes across pre-write and resolver inventories, then inventory the deadness test family before moving any files.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/spec/lumin-work-tracker.md
```

Expected: WT-24 now says pre-write and resolver inventories exist and fixture-shape comparison is next.

### Task 4: Verify Documentation Slice

**Files:**
- Read: `docs/lumin-wiki/workstreams/resolver.md`
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
git add -- docs/lumin-wiki/workstreams/resolver.md docs/lumin-wiki/log.md docs/spec/lumin-work-tracker.md docs/superpowers/plans/2026-05-12-resolver-test-inventory.md
git commit -m "Inventory resolver tests in Lumin wiki"
```

Expected: one commit containing only documentation changes.
