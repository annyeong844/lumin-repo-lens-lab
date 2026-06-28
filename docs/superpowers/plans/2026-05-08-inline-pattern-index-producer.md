# Inline Pattern Index Producer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Emit `inline-patterns.json` with conservative repeated inline catch-block cues.

**Architecture:** Add a focused artifact module that extracts per-file catch-block patterns from OXC ASTs, then a root producer script that scans JS/TS files and writes the aggregate artifact. This slice does not connect the artifact to pre-write rendering or make extraction recommendations.

**Tech Stack:** Node.js ESM scripts, `oxc-parser` through existing `parseOxcOrThrow`, project CLI parsing, existing scan helpers.

---

## File Structure

- Create `_lib/inline-pattern-artifact.mjs`: extraction, normalization, grouping, deterministic assembly, read/parse diagnostics.
- Create `build-inline-pattern-index.mjs`: CLI producer for `<output>/inline-patterns.json`.
- Create `tests/test-inline-pattern-index.mjs`: fixture-based TDD coverage for catch-destroy grouping and generic/noisy exclusions.
- Modify `tests/README.md`: register the new test command.

## Task 1: RED Producer Contract Test

**Files:**
- Create: `tests/test-inline-pattern-index.mjs`

- [ ] **Step 1: Write the failing test**

Create a fixture with four repeated catch blocks:

```js
try {
  send(connection.socket, payload);
} catch {
  connection.socket.destroy();
}
```

Run `node build-inline-pattern-index.mjs --root <fixture> --output <out> --production`.

Assert:

```js
artifact.meta.schemaVersion === 'inline-patterns.v1'
artifact.meta.supports.catchBlockPatterns === true
artifact.groups.length === 1
artifact.groups[0].kind === 'catch-block'
artifact.groups[0].size === 4
artifact.groups[0].normalizedPattern === 'catch { <id>.socket.destroy(); }'
artifact.groups[0].occurrences.length === 4
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-inline-pattern-index.mjs
```

Expected: FAIL because `build-inline-pattern-index.mjs` does not exist.

## Task 2: Minimal Artifact Extraction

**Files:**
- Create: `_lib/inline-pattern-artifact.mjs`
- Create: `build-inline-pattern-index.mjs`

- [ ] **Step 1: Implement catch-block extraction**

Use `parseOxcOrThrow(src, relFile)` and walk the AST. For every `CatchClause`, inspect `body.body`. Support only catch bodies with one or two statements. For v1, normalize expression statements that are member calls ending in `.destroy()` or another member call.

Minimal public functions:

```js
export function extractInlinePatternFilePayload({ src, relFile }) {}
export function inlinePatternReadErrorPayload(relFile, message) {}
export function assembleInlinePatternArtifact({ metaBase, includeTests, exclude, files }) {}
```

- [ ] **Step 2: Implement producer script**

`build-inline-pattern-index.mjs` should:

- parse common CLI args with `parseCliArgs()`;
- scan `JS_FAMILY_LANGS` through `collectFiles`;
- read each file;
- aggregate payloads;
- write `<output>/inline-patterns.json`;
- print a one-line summary.

- [ ] **Step 3: Verify GREEN**

Run:

```bash
node tests/test-inline-pattern-index.mjs
```

Expected: PASS.

## Task 3: Noise And Determinism Tests

**Files:**
- Modify: `tests/test-inline-pattern-index.mjs`
- Modify: `_lib/inline-pattern-artifact.mjs`

- [ ] **Step 1: Add tests for excluded noisy groups**

Add fixture cases that do not produce default groups:

```js
try { x(); } catch { return; }
try { x(); } catch { console.error(err); }
```

Assert `artifact.groups.length === 0` and muted/noisy diagnostics are either absent or recorded only in `mutedGroups`.

- [ ] **Step 2: Add deterministic ordering assertion**

Assert groups and occurrences are sorted by stable keys:

```js
group.size descending
group.patternHash ascending for ties
occurrence.file, line, endLine ascending
```

- [ ] **Step 3: Verify**

Run:

```bash
node tests/test-inline-pattern-index.mjs
```

Expected: PASS.

## Task 4: Test Docs And Skill Mirror

**Files:**
- Modify: `tests/README.md`
- Generated mirror files after `npm run build:skill`

- [ ] **Step 1: Register test docs**

Add:

```text
node tests/test-inline-pattern-index.mjs
```

with a short description.

- [ ] **Step 2: Build skill mirror**

Run:

```bash
npm run build:skill
```

Expected: PASS and mirrored engine files updated.

## Task 5: Verification And Commit

- [ ] **Step 1: Focused tests**

Run:

```bash
node tests/test-inline-pattern-index.mjs
npm run check
npm run lint
npm run check:test-doc
```

Expected: all PASS.

- [ ] **Step 2: Commit**

```bash
git add build-inline-pattern-index.mjs _lib/inline-pattern-artifact.mjs tests/test-inline-pattern-index.mjs tests/README.md skills/lumin-repo-lens-lab
git commit -m "Add inline pattern index producer"
```

## Self-Review Notes

- This plan implements P1 only from `docs/spec/pre-write-inline-extraction-cues.md`.
- P2 pre-write `AGENT_REVIEW_CUE` integration remains intentionally out of scope.
- The producer reports repeated syntax evidence only; it does not claim semantic equivalence or safe extraction.
