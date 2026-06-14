# Pre-Write Test Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Inventory the pre-write test family into the Lumin wiki without moving test files or changing analyzer behavior.

**Architecture:** This is a documentation-only slice. `docs/lumin-wiki/workstreams/pre-write.md` becomes the human-readable map of pre-write suites, protected invariants, and known follow-up reform targets. `docs/lumin-wiki/log.md` records the inventory event, and `docs/spec/lumin-work-tracker.md` updates WT-24's current state.

**Tech Stack:** Markdown docs, existing generated `tests/README.md`, existing pre-write test suites.

---

### Task 1: Add Pre-Write Suite Inventory

**Files:**
- Modify: `docs/lumin-wiki/workstreams/pre-write.md`

- [ ] **Step 1: Replace the current "Tests To Understand First" section with a risk-based inventory**

Use this exact section body:

```markdown
## Test Inventory

| Suite | Risk Type | Protected Invariant | Edge Case Or Negative Guard |
|---|---|---|---|
| `tests/test-pre-write-advisory-artifact.mjs` | artifact shape | Pre-write advisory JSON keeps the lifecycle contract stable. | Missing fields must be visible as artifact/schema drift, not silently ignored. |
| `tests/test-pre-write-bootstrap.mjs` | bootstrap contract | First-run pre-write setup can produce grounded baseline artifacts. | Bootstrap must not make `NOT_OBSERVED` look grounded when required evidence is unavailable. |
| `tests/test-pre-write-canonical-parser.mjs` | component contract | Canonical parser inputs are read deterministically for pre-write checks. | Parser drift should fail before advisory rendering changes. |
| `tests/test-pre-write-cli.mjs` | CLI contract | CLI flags and advisory output route pre-write intent into the engine consistently. | Suppressed diagnostics remain muted evidence and do not become cue cards. |
| `tests/test-pre-write-cue-tiers.mjs` | ranking/review lane | Cue tiers separate `EXISTS`, `SAFE_FIX`, `AGENT_REVIEW_CUE`, and muted evidence. | Suppressed semantic/near candidates must not leak into promoted cue cards. |
| `tests/test-pre-write-drift.mjs` | drift guard | Pre-write output shape stays compatible with tracked expectations. | Shape drift should fail loudly rather than changing agent-facing semantics silently. |
| `tests/test-pre-write-inline-patterns.mjs` | regression edge case | Explicit refactor sources and inline-pattern artifacts can surface repeated extraction cues. | Inline repeated catch patterns are review cues, not proof that a new helper is safe. |
| `tests/test-pre-write-integration.mjs` | lifecycle integration | Pre-write integrates lookup, evidence, and rendering through the public workflow. | Integration must preserve evidence labels when a baseline is missing or partial. |
| `tests/test-pre-write-intent.mjs` | component contract | Intent parsing extracts names, files, shapes, and refactor sources predictably. | Ambiguous intent should stay advisory rather than becoming grounded absence. |
| `tests/test-pre-write-inventory-hook.mjs` | artifact availability | Pre-write can consume inventory snapshots produced by the hook flow. | Stale or missing inventory cannot justify `NOT_OBSERVED` as absence. |
| `tests/test-pre-write-lookup-dep.mjs` | lookup contract | Dependency lookup reports observed package/dependency matches. | Unobserved dependency evidence must remain scoped to scan availability. |
| `tests/test-pre-write-lookup-file.mjs` | lookup contract | File lookup identifies existing, near, and missing file targets. | Missing scan artifacts should degrade absence claims. |
| `tests/test-pre-write-lookup-name.mjs` | lookup contract | Name lookup reports exact, near, class-method, semantic, and suppressed candidates. | `searchUser` versus `fetchUser` is recorded as suppressed evidence without relaxing thresholds. |
| `tests/test-pre-write-lookup-shape.mjs` | shape contract | Shape lookup consumes exact shape-index hashes without heuristic fallback. | Unsupported or missing shape evidence must stay diagnostic. |
| `tests/test-pre-write-render.mjs` | renderer contract | Markdown rendering presents advisory evidence without overclaiming. | Muted/suppressed details should not render as user-facing proof by default. |
| `tests/test-pre-write-shape-index.mjs` | artifact integration | Pre-write shape lookup consumes `shape-index.json` by exact hash. | Shape-index absence or drift should not become a false duplicate claim. |
| `tests/test-audit-repo-pre-write.mjs` | orchestrator integration | `audit-repo.mjs --pre-write` routes pre-write through the audit lifecycle. | A pre-write-only invocation without grounded baseline must expose evidence availability. |
| `tests/test-audit-repo-post-write.mjs` | adjacent lifecycle guard | Post-write orchestration remains separate from pre-write baseline semantics. | Post-write must not mutate or reinterpret pre-write baseline evidence. |
| `tests/test-class-method-prewrite-surface.mjs` | regression edge case | Class methods surface as pre-write review cues without entering dead-export `defIndex`. | OO methods such as `handleDelete` must be visible for reuse review but not dead-export proof. |
| `tests/test-class-method-index-prototype-names.mjs` | regression edge case | Prototype-named methods are stored as ordinary class-method keys. | `constructor`, `toString`, `hasOwnProperty`, `valueOf`, and `__proto__` must not crash dictionary grouping. |
| `tests/test-inline-pattern-index.mjs` | producer/artifact shape | Repeated inline catch-block patterns are collected as review cue evidence. | Pattern facts must stay review-only until a named extraction policy exists. |
```

- [ ] **Step 2: Add a reform target section below the inventory**

Use this exact section:

```markdown
## Reform Targets

- Add missing descriptions for pre-write suites that still appear in the
  generated `tests/README.md` maintainer note.
- Split broad integration assertions only after the protected invariant is
  named in this page.
- Merge fixture shapes only when the shared fixture keeps the original failure
  mode visible.
- Prefer edge-case red tests over helper-existence red tests for new pre-write
  work.
- Keep suppressed candidates muted until a named cue policy has corpus evidence.
```

- [ ] **Step 3: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/workstreams/pre-write.md
```

Expected: the pre-write wiki page now has a `Test Inventory` table and `Reform Targets` section.

### Task 2: Record The Wiki Update

**Files:**
- Modify: `docs/lumin-wiki/log.md`

- [ ] **Step 1: Append a log entry**

Append this exact entry after the existing scaffold entry:

```markdown

## [2026-05-12] inventory | pre-write test family

Added a risk-based inventory for pre-write-related suites. The inventory names
each suite's protected invariant and the edge case or negative guard that should
survive future test reform.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/log.md
```

Expected: one new chronological inventory entry appears.

### Task 3: Update WT-24 State

**Files:**
- Modify: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Update the WT-24 row**

Replace the WT-24 `Current State` sentence with:

```markdown
`docs/lumin-wiki/` establishes a maintainer synthesis layer with workstream pages, evidence concepts, and test reform rules. `docs/lumin-wiki/workstreams/pre-write.md` now contains the first risk-based suite inventory for pre-write-related tests, including protected invariants and edge-case/negative guards. `docs/superpowers/specs/2026-05-12-lumin-wiki-test-reform-design.md` records the documentation-only scaffold and the rule that future TDD should fail on concrete edge cases rather than missing helpers.
```

Replace the WT-24 `Next Small PR` sentence with:

```markdown
Inventory the resolver test family next, then compare duplicated fixture shapes before moving any files.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/spec/lumin-work-tracker.md
```

Expected: WT-24 now says pre-write inventory exists and resolver inventory is next.

### Task 4: Verify Documentation Slice

**Files:**
- Read: `docs/lumin-wiki/workstreams/pre-write.md`
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
git add -- docs/lumin-wiki/workstreams/pre-write.md docs/lumin-wiki/log.md docs/spec/lumin-work-tracker.md docs/superpowers/plans/2026-05-12-prewrite-test-inventory.md
git commit -m "Inventory pre-write tests in Lumin wiki"
```

Expected: one commit containing only documentation changes.
