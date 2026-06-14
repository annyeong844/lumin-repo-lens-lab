# Fixture Shape Comparison Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compare repeated fixture shapes across the inventoried Lumin wiki workstreams before moving test files.

**Architecture:** This is a documentation-only slice. `docs/lumin-wiki/concepts/fixture-shapes.md` names repeated fixture patterns and preserves which assertions must stay suite-local. `docs/lumin-wiki/index.md`, `docs/lumin-wiki/log.md`, and `docs/spec/lumin-work-tracker.md` are updated so the wiki records that suite inventory has moved into fixture-shape comparison.

**Tech Stack:** Markdown docs, existing Lumin wiki workstream inventories, existing generated `tests/README.md`.

---

### Task 1: Add Fixture Shape Comparison Page

**Files:**
- Create: `docs/lumin-wiki/concepts/fixture-shapes.md`

- [ ] **Step 1: Create the page**

Use this exact page body:

```markdown
# Fixture Shapes

Fixture-shape comparison is the bridge between suite inventory and test-file
movement. This page names repeated fixture patterns across the inventoried
workstreams so future refactors can merge setup code without merging unrelated
risk claims.

## Comparison Rules

- Compare fixture shapes before moving test files.
- Preserve the original failure mode when sharing setup helpers.
- Keep artifact-shape assertions near the artifact they protect.
- Keep analyzer correctness, public packaging, and lab evidence separate even
  when they use similar temporary repos.
- Do not turn a shared helper into a shared interpretation of evidence.

## Repeated Shapes

| Fixture Shape | Seen In | Shared Setup Candidate | Must Stay Separate |
|---|---|---|---|
| Temporary repo plus `.audit` output directory | [Pre-Write](../workstreams/pre-write.md), [Deadness](../workstreams/deadness.md), [Performance](../workstreams/performance.md) | A small helper that creates a repo root, output root, `package.json`, writes files, and runs one producer or audit command. | The assertion lens: pre-write evidence availability, deadness review proof, and performance counters are different claims. |
| Unsupported resolver family mini repo | [Resolver](../workstreams/resolver.md), [Deadness](../workstreams/deadness.md) | A helper for internal-looking unresolved imports, unsupported family records, blocked candidate hints, and no concrete graph edge. | Family identity must remain explicit: generated artifacts, output-to-source layouts, Node `#imports`, and dynamic modules are not interchangeable. |
| Generated or framework/resource surface package | [Resolver](../workstreams/resolver.md), [Deadness](../workstreams/deadness.md), [Public Package](../workstreams/public-package.md) | A builder for `package.json`, framework dependencies, generated-looking paths, bundled files, declarations, templates, and codemod resources. | Resolver blind-zone evidence, deadness blockers, and public package allowlist checks protect different contracts. |
| Consumer/member precision graph | [Pre-Write](../workstreams/pre-write.md), [Deadness](../workstreams/deadness.md) | A fixture with exported siblings, namespace or class-member consumers, and one unused sibling. | Pre-write class methods are review cues; deadness member precision is export-consumer evidence. They must not share ranking expectations. |
| Prototype-name dictionary edge case | [Pre-Write](../workstreams/pre-write.md), [Performance](../workstreams/performance.md) | A tiny class/function fixture that includes names such as `constructor`, `toString`, `hasOwnProperty`, `valueOf`, and `__proto__`. | The same shape can guard different grouping bugs: class-method indexing, clone grouping, symbol graph maps, or cache dictionaries. |
| Cold/warm incremental repo | [Performance](../workstreams/performance.md), [Pre-Write](../workstreams/pre-write.md) | A helper that runs cold, mutates one file, reruns warm, and compares refreshed versus reused facts. | Post-write and pre-write lifecycle caches have different baseline semantics; a helper must not hide that difference. |
| Public/internal export-surface package | [Public Package](../workstreams/public-package.md), [Deadness](../workstreams/deadness.md) | A package fixture with explicit public files, internal files, generated package output, and manifest summaries. | Public package tests protect shipped surface. Deadness tests protect absence claims. One must not justify the other. |
| Markdown renderer and manifest mirror | [Resolver](../workstreams/resolver.md), [Deadness](../workstreams/deadness.md), [Public Package](../workstreams/public-package.md) | A helper that writes raw JSON artifacts, builds manifest summaries, and asserts summary/review-pack reader guidance. | Markdown visibility is reader guidance. It is not proof that the underlying analyzer fact is correct. |

## First Refactor Candidate

The safest first extraction candidate is a temporary repo helper with these
operations:

- create an isolated root and output directory
- write `package.json`
- write source files by relative path
- read JSON artifacts by relative path
- clean up after the test

This helper should not know about resolver families, pre-write intents,
deadness tiers, package publishing, or performance interpretation. Those remain
owned by the specific suites named in the workstream inventories.

## Shapes Not Ready To Merge

- Resolver unsupported-family fixtures should not collapse until each family
  keeps a named reason, output level, and no-fake-edge assertion.
- Public install verification notes under `docs/lab/` are evidence records, not
  reusable unit-test fixtures.
- Scanner equivalence fixtures should stay close to the scanner until accepted
  syntax and fallback reasons are stable.
- SAFE_FIX calibration fixtures should stay separate from review-evidence
  fixtures until ranking behavior has a narrower design.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/concepts/fixture-shapes.md
```

Expected: the new page compares shapes without proposing test-file movement.

### Task 2: Link The New Page

**Files:**
- Modify: `docs/lumin-wiki/index.md`

- [ ] **Step 1: Add the concept link**

Add `Fixture Shapes` to the Concepts list after `Test Reform`:

```markdown
- [Fixture Shapes](concepts/fixture-shapes.md) — repeated fixture patterns
  across inventoried workstreams and what must stay suite-local.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/index.md
```

Expected: the wiki index links to the new concept page.

### Task 3: Record The Wiki Update

**Files:**
- Modify: `docs/lumin-wiki/log.md`

- [ ] **Step 1: Append a log entry**

Append this exact entry:

```markdown

## [2026-05-12] comparison | fixture shapes across inventories

Added a fixture-shape comparison page that names repeated temporary repo,
resolver unsupported-family, generated/framework surface, consumer/member,
incremental, public/internal package, and Markdown mirror shapes before any
test-file movement.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/lumin-wiki/log.md
```

Expected: one new fixture-shape comparison entry appears.

### Task 4: Update WT-24 State

**Files:**
- Modify: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Update the WT-24 row**

Add `docs/lumin-wiki/concepts/fixture-shapes.md` to the WT-24 current state and replace the next small PR with:

```markdown
Draft a narrow shared temporary-repo fixture helper spec before moving any test files.
```

- [ ] **Step 2: Save the file**

Run:

```powershell
git diff -- docs/spec/lumin-work-tracker.md
```

Expected: WT-24 now says fixture-shape comparison exists and the next step is a narrow helper spec, not test movement.

### Task 5: Verify Documentation Slice

**Files:**
- Read: `docs/lumin-wiki/concepts/fixture-shapes.md`
- Read: `docs/lumin-wiki/index.md`
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
git add -- docs/lumin-wiki/concepts/fixture-shapes.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/spec/lumin-work-tracker.md docs/superpowers/plans/2026-05-12-fixture-shape-comparison.md
git commit -m "Compare fixture shapes in Lumin wiki"
```

Expected: one commit containing only documentation changes.
