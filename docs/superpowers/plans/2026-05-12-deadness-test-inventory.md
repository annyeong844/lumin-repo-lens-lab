# Deadness Test Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Inventory the deadness and SAFE_FIX test family into the Lumin wiki without moving test files or changing analyzer behavior.

**Architecture:** This is a documentation-only slice. `docs/lumin-wiki/workstreams/deadness.md` becomes the human-readable map of deadness suites, graph lenses, action-proof boundaries, and negative guards. `docs/lumin-wiki/log.md` records the inventory event, and `docs/spec/lumin-work-tracker.md` updates WT-24's current and next states.

**Tech Stack:** Markdown docs, existing generated `tests/README.md`, existing deadness/ranking/action-safety test suites.

---

### Task 1: Add Deadness Suite Inventory

**Files:**
- Modify: `docs/lumin-wiki/workstreams/deadness.md`

- [ ] **Step 1: Replace the current "Tests To Understand First" section with a risk-based inventory**

Use this exact section body:

```markdown
## Test Inventory

| Suite | Risk Type | Protected Invariant | Edge Case Or Negative Guard |
|---|---|---|---|
| `tests/test-export-action-safety.mjs` | SAFE_FIX action proof | Export demotion/deletion actions carry proof, blockers, and syntax preservation facts. | Demote may be safe while delete remains blocked by local refs, class semantics, or side effects. |
| `tests/test-rank-fixes.mjs` | ranking contract | Fix-plan tiers merge evidence, blockers, and action safety into stable user-facing ranks. | Review evidence and soft confidence gaps must not silently promote to `SAFE_FIX`. |
| `tests/test-module-reachability.mjs` | file reachability | Runtime/type entry BFS records reachable, unreachable, and entry-unreachable SCC evidence. | Entry-unreachable SCCs are dead-file-group review evidence, not export-level SAFE_FIX proof. |
| `tests/test-namespace-reexport-deadness.mjs` | member precision regression | Namespace re-export fan-in protects used members exactly and leaves unused siblings visible. | Chained namespace reads must not blanket-protect every sibling; opaque escapes become diagnostics. |
| `tests/test-public-deep-import-risk.mjs` | public surface blocker | Public package exports/deep-import risk can block entry-unreachable confidence support. | Files excluded by `files` can reduce one blocker, but compiled artifacts are not proven absent. |
| `tests/test-public-surface.mjs` | public entry surface | Package, declaration, script-driven, and HTML entry surfaces feed public contract evidence. | Public entry detection must not create phantom files or overclaim unreachable source. |
| `tests/test-framework-resource-surfaces.mjs` | framework/resource blocker | Framework/resource lanes identify surfaces that can hide consumers outside ordinary imports. | Stories, Strapi routes, bundles, generated declarations, templates, and codemods are evidence lanes, not positive deadness proof. |
| `tests/test-generated-blind-zone-relevance.mjs` | blind-zone blocker | Generated artifact blind zones block only relevant SAFE_FIX promotion. | Unrelated generated misses must not become repo-global SAFE_FIX blockers. |
| `tests/test-generated-consumer-blind-zones.mjs` | generated consumer blocker | Missing/excluded generated consumer surfaces are reported as possible hidden consumers. | Generated consumers block absence claims without being counted as observed source consumers. |
| `tests/test-resolver-blind-zone-relevance.mjs` | resolver blocker | Resolver blind-zone relevance scopes unresolved imports to affected candidates. | Unresolved imports outside the candidate package must not block unrelated SAFE_FIX promotion. |
| `tests/test-cjs-classification.mjs` | consumer extraction | CommonJS consumers participate in symbol graph and dead-export classification. | CJS support should add real consumers without blanket-protecting opaque surfaces. |
| `tests/test-cjs-integration.mjs` | CJS opacity regression | CJS export surface, alias destructuring, and dynamic require opacity integrate with deadness. | Dynamic or broad CJS opacity must degrade claims rather than fake precise consumer evidence. |
| `tests/test-extract-cjs-consumer.mjs` | consumer extraction unit | Direct CJS require consumers are extracted for exact, side-effect-only, and broad escape forms. | Broad escapes should stay conservative and not pretend named consumers are known. |
| `tests/test-mdx-consumers.mjs` | docs-driven consumer evidence | MDX imports can contribute symbol fan-in without file-level overprotection. | Docs-driven component consumers should protect imported symbols only, not all siblings. |
| `tests/test-p6-member-precision.mjs` | member precision calibration | Namespace and dynamic import member precision protect only directly used exports when possible. | Degraded aliases remain conservative instead of fabricating exact member evidence. |
| `tests/test-p6-safe-fix-calibration.mjs` | calibration corpus | SAFE_FIX calibration uses runtime/staleness convergence on a real mini git repo. | Static confidence alone cannot promote SAFE_FIX without proof objects and calibration evidence. |
| `tests/test-definition-id-canonical.mjs` | identity contract | Canonical definition IDs align symbols, action safety, and call-graph alias fan-in. | Identity drift must fail before consumers or blockers attach to the wrong export. |
```

- [ ] **Step 2: Add a reform target section below the existing reform direction**

Use this exact section:

```markdown
## Reform Targets

- Split deadness tests by graph lens: export consumer, file reachability,
  runtime SCC, public surface, generated surface, framework surface, and action
  proof.
- Keep every review-evidence fixture paired with a "not SAFE_FIX proof"
  assertion.
- Compare CJS, MDX, namespace, and dynamic-member fixtures for shared consumer
  extraction helpers before moving files.
- Keep blocker tests scoped: public, resolver, generated, and framework
  blockers protect different absence contracts.
- Do not use unreachable-file evidence as automated export-removal proof without
  a separate ranking design.
```

- [ ] **Step 3: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/workstreams/deadness.md
```

Expected: the deadness wiki page now has a `Test Inventory` table and `Reform Targets` section.

### Task 2: Record The Wiki Update

**Files:**
- Modify: `docs/lumin-wiki/log.md`

- [ ] **Step 1: Append a log entry**

Append this exact entry after the resolver inventory entry:

```markdown

## [2026-05-12] inventory | deadness test family

Added a risk-based inventory for deadness, reachability, ranking, action-safety,
consumer extraction, and SAFE_FIX calibration suites. The inventory separates
review evidence from automated action proof before any test-file movement.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/log.md
```

Expected: one new chronological deadness inventory entry appears.

### Task 3: Update WT-24 State

**Files:**
- Modify: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Update the WT-24 row**

Replace the WT-24 `Current State` sentence with:

```markdown
`docs/lumin-wiki/` establishes a maintainer synthesis layer with workstream pages, evidence concepts, and test reform rules. `docs/lumin-wiki/workstreams/pre-write.md`, `docs/lumin-wiki/workstreams/resolver.md`, and `docs/lumin-wiki/workstreams/deadness.md` now contain the first risk-based suite inventories, including protected invariants and edge-case/negative guards. `docs/superpowers/specs/2026-05-12-lumin-wiki-test-reform-design.md` records the documentation-only scaffold and the rule that future TDD should fail on concrete edge cases rather than missing helpers.
```

Replace the WT-24 `Next Small PR` sentence with:

```markdown
Compare duplicated fixture shapes across pre-write, resolver, and deadness inventories, then inventory the performance test family before moving any files.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/spec/lumin-work-tracker.md
```

Expected: WT-24 now says pre-write, resolver, and deadness inventories exist and fixture-shape comparison is next.

### Task 4: Verify Documentation Slice

**Files:**
- Read: `docs/lumin-wiki/workstreams/deadness.md`
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
git add -- docs/lumin-wiki/workstreams/deadness.md docs/lumin-wiki/log.md docs/spec/lumin-work-tracker.md docs/superpowers/plans/2026-05-12-deadness-test-inventory.md
git commit -m "Inventory deadness tests in Lumin wiki"
```

Expected: one commit containing only documentation changes.
