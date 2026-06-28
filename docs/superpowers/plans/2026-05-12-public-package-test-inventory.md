# Public Package Test Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Inventory public-package, publish, public/internal boundary, policy, and package-verification tests into the Lumin wiki without moving test files or changing analyzer behavior.

**Architecture:** This is a documentation-only slice. `docs/lumin-wiki/workstreams/public-package.md` becomes the human-readable map of package, publish, CI, policy, and installed-package evidence. `docs/lumin-wiki/log.md` records the inventory event, and `docs/spec/lumin-work-tracker.md` updates WT-24's current and next states.

**Tech Stack:** Markdown docs, existing generated `tests/README.md`, existing packaging/publish/surface test suites.

---

### Task 1: Add Public-Package Suite Inventory

**Files:**
- Modify: `docs/lumin-wiki/workstreams/public-package.md`

- [ ] **Step 1: Replace the current "Tests And Evidence To Understand First" section with a risk-based inventory**

Use this exact section body:

```markdown
## Test Inventory

| Suite Or Evidence | Risk Type | Protected Invariant | Edge Case Or Negative Guard |
|---|---|---|---|
| `tests/test-plugin-package.mjs` | plugin package contract | Claude Code plugin package metadata, slash commands, generated skill surfaces, and Codex wrapper opt-in remain coherent. | Generated package must not include lab/private payload or stale metadata. |
| `tests/test-skill-package.mjs` | deployable skill package | Deployable skill package includes plugin wrapper, public scripts, commands, `_engine` internals, canonical/templates/references, and excludes lab payload. | Public skill surface remains small; maintainer docs do not leak unless explicitly packaged. |
| `tests/test-skill-surface.mjs` | public surface contract | Shared audit engine and skill surfaces keep stable validation modes and internal-vs-public doc split. | User-facing prompts must not expose hidden engine modes or internal-only docs. |
| `tests/test-publish-public-plugin.mjs` | publish pipeline | Public plugin publisher uses generated package allowlist, changelog prepend, dry-run, and push flow safely. | Dry-run must catch package drift before push; allowlist prevents accidental private file publication. |
| `tests/test-github-actions-ci-policy.mjs` | public CI policy | GitHub Actions CI policy skips runner jobs for draft PRs while ready/manual/push still run. | Public CI must avoid burning private runner budget for draft validation while still supporting real release checks. |
| `tests/test-maintainer-scripts.mjs` | maintainer script hardening | Maintainer scripts surface child-process spawn errors and optional public package reads safely. | Spawn failures and missing optional package files must not be silently treated as successful publish checks. |
| `tests/test-audit-manifest-export-surface.mjs` | public/internal boundary | Audit manifest exposes stable summary surfaces without living-audit internals. | Maintainer-only evidence must not become public package contract by accident. |
| `tests/test-definition-id-export.mjs` | public/internal boundary | Definition-id export surface hides raw id builder internals. | Internal identity builders should not leak into public API. |
| `tests/test-file-delta-export.mjs` | public/internal boundary | Post-write file-delta export surface hides path normalizer internals. | Internal path normalizer details should not become package API. |
| `tests/test-function-clone-export-surface.mjs` | public/internal boundary | Function-clone artifact export surface hides version internals. | Version internals should not leak into public artifact consumers. |
| `tests/test-classify-policies-export-surface.mjs` | public/internal boundary | Classify-policies export surface stays limited to active policy APIs. | Deprecated or internal policy helpers must not become public API. |
| `tests/test-threshold-policies.mjs` | policy metadata | Threshold policy ids, versions, hashes, and compact artifact summaries remain explicit. | Magic-number thresholds must not affect public output without named policy metadata. |
| `tests/test-threshold-policy-drift-guard.mjs` | drift guard | Numeric threshold changes require explicit snapshot review. | Threshold tuning must not silently change ranking/rendering behavior. |
| `tests/test-update-test-doc.mjs` | generated docs drift guard | `tests/README.md` is generated from actual suites and changelog and omits assertion-count authority. | Hand-edited or stale test docs must fail check mode. |
| `tests/test-behavior-corpus-verifier.mjs` | behavior corpus | Saved-answer verifier protects no-jargon, caveat, and summary-order behavior. | Prompt/user-facing behavior checks stay separate from engine correctness tests. |
| `docs/lab/*public*verification*.md` | installed package evidence | Public/package install verification records real installed version behavior. | Dev-tree parity is not assumed; installed package evidence is required before `DONE`. |
```

- [ ] **Step 2: Add a reform target section below the existing reform direction**

Use this exact section:

```markdown
## Reform Targets

- Separate package build tests from installed-package verification notes.
- Keep generated package allowlist checks close to publish tests.
- Keep public/internal boundary tests focused on export surface, not behavior.
- Treat public CI policy as budget and routing evidence, not analyzer
  correctness.
- Do not mark user-visible tracker items `DONE` from dev-tree tests alone when
  package install verification is required.
```

- [ ] **Step 3: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/workstreams/public-package.md
```

Expected: the public-package wiki page now has a `Test Inventory` table and `Reform Targets` section.

### Task 2: Record The Wiki Update

**Files:**
- Modify: `docs/lumin-wiki/log.md`

- [ ] **Step 1: Append a log entry**

Append this exact entry:

```markdown

## [2026-05-12] inventory | public-package test family

Added a risk-based inventory for plugin package, skill package, publish, public
CI, export-surface boundary, threshold policy, generated docs, behavior corpus,
and installed-package verification evidence before any test-file movement.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/log.md
```

Expected: one new chronological public-package inventory entry appears.

### Task 3: Update WT-24 State

**Files:**
- Modify: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Update the WT-24 row**

Replace the WT-24 `Current State` sentence with:

```markdown
`docs/lumin-wiki/` establishes a maintainer synthesis layer with workstream pages, evidence concepts, and test reform rules. `docs/lumin-wiki/workstreams/pre-write.md`, `docs/lumin-wiki/workstreams/resolver.md`, `docs/lumin-wiki/workstreams/deadness.md`, `docs/lumin-wiki/workstreams/performance.md`, and `docs/lumin-wiki/workstreams/public-package.md` now contain the first risk-based suite inventories, including protected invariants and edge-case/negative guards. `docs/superpowers/specs/2026-05-12-lumin-wiki-test-reform-design.md` records the documentation-only scaffold and the rule that future TDD should fail on concrete edge cases rather than missing helpers.
```

Replace the WT-24 `Next Small PR` sentence with:

```markdown
Compare duplicated fixture shapes across inventoried workstreams before moving any files.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/spec/lumin-work-tracker.md
```

Expected: WT-24 now says pre-write, resolver, deadness, performance, and public-package inventories exist and fixture-shape comparison is next.

### Task 4: Verify Documentation Slice

**Files:**
- Read: `docs/lumin-wiki/workstreams/public-package.md`
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
git add -- docs/lumin-wiki/workstreams/public-package.md docs/lumin-wiki/log.md docs/spec/lumin-work-tracker.md docs/superpowers/plans/2026-05-12-public-package-test-inventory.md
git commit -m "Inventory public-package tests in Lumin wiki"
```

Expected: one commit containing only documentation changes.
