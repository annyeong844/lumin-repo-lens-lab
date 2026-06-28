# Shared Temp Fixture Helper Spec Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Draft the narrow shared temporary-repo fixture helper spec before moving test files.

**Architecture:** This is a documentation-only slice. `docs/spec/shared-test-fixture-helper.md` defines a setup-only helper contract, safety boundary, proposed API, and first implementation tests. Wiki and tracker pages link to the spec so future test reform starts from a narrow helper rather than broad suite movement.

**Tech Stack:** Markdown docs, existing Lumin wiki fixture-shape comparison, existing test reform tracker.

---

### Task 1: Add The Shared Fixture Helper Spec

**Files:**
- Create: `docs/spec/shared-test-fixture-helper.md`

- [ ] **Step 1: Create the spec**

Create the spec with these required sections and concrete contents:

```markdown
# Shared Test Fixture Helper

> **Role:** maintainer-facing implementation spec for the first narrow shared
> test fixture helper.
> **Status:** SPEC.
> **Last updated:** 2026-05-12

## 1. Problem

State that many tests repeat temp repo setup, but broad test movement can hide
suite-specific failure modes.

## 2. Goals

Include isolated temporary repo setup, safe file/JSON helpers, Windows/POSIX
path safety, and analyzer interpretation staying outside the helper.

## 3. Non-Goals

Forbid broad test movement, resolver/deadness/pre-write/package/performance
semantics, command running, generated layout builders, and hand-editing
`tests/README.md`.

## 4. Proposed File

Name `tests/_helpers/temp-repo-fixture.mjs` and state that it is test-only.

## 5. Proposed API

Define `createTempRepoFixture(options)` returning `root`, `output`,
`path()`, `outputPath()`, `mkdir()`, `write()`, `writeJson()`, `read()`,
`readJson()`, and `cleanup()`.

## 6. Safety Contract

Require containment checks and path rejection for absolute, drive-letter,
parent traversal, empty, and NUL paths.

## 7. Interpretation Boundary

Allow only directory creation, text/JSON writing, text/JSON reading, path
returning, and cleanup. Forbid resolver diagnostics, dead-export
classification, pre-write intents, package allowlists, audit execution, and
rank assertions.

## 8. First Implementation Tests

Name `tests/test-temp-repo-fixture-helper.mjs` and require cases for default
package creation, nested file writes, root/output JSON readback, path rejection,
NUL rejection, and safe cleanup.

## 9. First Migration Candidate

Limit the first migration to at most one low-risk suite after helper tests
exist. Exclude resolver unsupported-family, SAFE_FIX calibration, public
install verification, scanner equivalence, and pre-write lifecycle baseline
tests.

## 10. Acceptance Criteria

Require safety tests, original negative assertions, no analyzer behavior change,
no generated package surface change, no broad test movement, and generated test
docs staying generated.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/spec/shared-test-fixture-helper.md
```

Expected: the new spec defines a setup-only temporary repo helper.

### Task 2: Link The Spec From The Wiki

**Files:**
- Modify: `docs/lumin-wiki/concepts/fixture-shapes.md`
- Modify: `docs/lumin-wiki/log.md`

- [ ] **Step 1: Add a spec link to the fixture-shapes page**

Add a sentence under `First Refactor Candidate`:

```markdown
The setup-only helper contract is specified in
[`docs/spec/shared-test-fixture-helper.md`](../../spec/shared-test-fixture-helper.md).
```

- [ ] **Step 2: Append a log entry**

Append this exact entry:

```markdown

## [2026-05-12] spec | shared temporary repo fixture helper

Added a setup-only shared fixture helper spec that limits the first test-reform
extraction to temporary repo creation, file/JSON helpers, safe path containment,
and cleanup before any broad test movement.
```

### Task 3: Link The Spec From Spec README And WT-24

**Files:**
- Modify: `docs/spec/README.md`
- Modify: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Add the spec README bullet**

Add this bullet after the wiki/test-reform design bullet:

```markdown
- shared test fixture helper specs such as
  `docs/spec/shared-test-fixture-helper.md`
```

- [ ] **Step 2: Update WT-24**

Add `docs/spec/shared-test-fixture-helper.md` to the WT-24 current state and replace the next small PR with:

```markdown
Implement the setup-only temporary repo fixture helper with safety tests, then migrate at most one low-risk suite.
```

### Task 4: Verify Documentation Slice

**Files:**
- Read: `docs/spec/shared-test-fixture-helper.md`
- Read: `docs/lumin-wiki/concepts/fixture-shapes.md`
- Read: `docs/lumin-wiki/log.md`
- Read: `docs/spec/README.md`
- Read: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Check for placeholders**

Run:

```powershell
rg "TBD|TODO|PLACEHOLDER|FIXME|\\?\\?" docs/lumin-wiki docs/spec/lumin-work-tracker.md docs/spec/shared-test-fixture-helper.md
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
git add -- docs/spec/shared-test-fixture-helper.md docs/lumin-wiki/concepts/fixture-shapes.md docs/lumin-wiki/log.md docs/spec/README.md docs/spec/lumin-work-tracker.md docs/superpowers/plans/2026-05-12-shared-temp-fixture-helper-spec.md
git commit -m "Specify shared temp repo fixture helper"
```

Expected: one commit containing only documentation changes.
