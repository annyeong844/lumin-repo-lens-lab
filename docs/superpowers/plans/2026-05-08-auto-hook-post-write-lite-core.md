# Auto Hook Post-Write Lite Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the Phase 1 `post-write-lite` core that compares PreToolUse preimage escape facts with disk postimage facts and appends `silent-new` events, without activating hooks.

**Architecture:** This slice upgrades preimage records to carry source-derived type escape facts without raw source text. A new post-write-lite core groups mutating tool calls by repo file, uses the first call's preimage as baseline, diffs escape occurrence multisets, appends event-store entries, cleans all group preimages, and optionally drains reminders through the existing drainer.

**Tech Stack:** Node.js ESM, `_lib/extract-ts-escapes.mjs`, `_lib/hook-preimage-store.mjs`, `_lib/hook-event-store.mjs`, `_lib/hook-event-drain.mjs`, direct Node test scripts.

---

## File Structure

- Modify `_lib/hook-preimage-store.mjs`
  - Add `typeEscapes` and optional `parseError` to `fingerprint` for readable source files.
  - Continue storing only fingerprints/facts, never raw source text.
- Create `_lib/hook-post-write-lite.mjs`
  - Owns PostToolBatch payload processing.
  - Exposes `processPostWriteLite(payload, opts)`.
- Create `tests/test-hook-post-write-lite.mjs`
  - Covers preimage escape fact persistence, silent-new append, duplicate merge, same-file grouping, preimage cleanup, and non-mutating no-op.
- Modify `scripts/update-test-doc.mjs`
  - Adds the new test description.
- Generated:
  - `tests/README.md`
  - `skills/lumin-repo-lens-lab/_engine/lib/hook-preimage-store.mjs`
  - `skills/lumin-repo-lens-lab/_engine/lib/hook-post-write-lite.mjs`

## Contract

`capturePreimage(...)` for an existing JS/TS file records:

```json
{
  "fingerprint": {
    "sha256": "sha256:...",
    "sizeBytes": 123,
    "mtimeMs": 123,
    "typeEscapes": [
      {
        "file": "src/a.ts",
        "line": 1,
        "escapeKind": "as-any",
        "codeShape": "value as any",
        "normalizedCodeShape": "value as any",
        "insideExportedIdentity": null,
        "occurrenceKey": "sha256:..."
      }
    ],
    "parseError": null
  }
}
```

`processPostWriteLite(payload, opts)` returns:

```js
{
  processedFiles: 1,
  appendedEventIds: ['evt_...'],
  preimageIncompleteFiles: [],
  output: { hookSpecificOutput: { hookEventName: 'PostToolBatch', additionalContext: '...' } } | null
}
```

Rules:

- Mutating tools are `Edit`, `Write`, and `MultiEdit`.
- Non-mutating batches return no-op and create no event store.
- Calls are grouped by safe repo-relative file path.
- Per file, only the first call's preimage is used as baseline.
- Missing first preimage uses an empty baseline and records `preimageIncompleteFiles`.
- Later preimages are never used as fallback baseline.
- All preimages in a processed group are cleaned up.
- Events use `kind: "silent-new"`, `delivery_policy: "until_ack"`, and `occurrence_delta` equal to net added occurrences per occurrence key.
- Dedupe keys are opaque occurrence keys; code never splits them.
- Source files are read only once per processed group after the batch.
- Hook activation remains out of scope.

## Task 1: RED Tests

**Files:**
- Create: `tests/test-hook-post-write-lite.mjs`

- [ ] **Step 1: Add tests**

Assertions:

- `capturePreimage` stores `fingerprint.typeEscapes` for existing files without raw source leakage;
- `processPostWriteLite` appends one `silent-new` event when an `as any` appears after preimage capture;
- appended event carries file, line, escape kind, snippet, matched line text, and an opaque dedupe key;
- duplicate same occurrence key appends once and increments occurrence count through event-store merge;
- same-file group uses the first preimage and cleans all group preimages;
- missing first preimage over-warns from empty baseline and records the file in `preimageIncompleteFiles`;
- non-mutating batches create no event-store directory.

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-post-write-lite.mjs
```

Expected: fails because `_lib/hook-post-write-lite.mjs` does not exist and preimage records lack `typeEscapes`.

## Task 2: Preimage Escape Facts

**Files:**
- Modify: `_lib/hook-preimage-store.mjs`

- [ ] **Step 1: Add escape facts to file fingerprints**

When `capturePreimage` reads an existing file, call `extractTypeEscapes(src, repoRel)`. Store `typeEscapes` and `parseError` under `fingerprint`. Do not store raw source.

- [ ] **Step 2: Verify preimage portions**

Run:

```bash
node tests/test-hook-post-write-lite.mjs
```

Expected: preimage fact assertion passes; post-write-lite assertions still fail until Task 3.

## Task 3: Post-Write Lite Core

**Files:**
- Create: `_lib/hook-post-write-lite.mjs`

- [ ] **Step 1: Implement payload grouping**

Use `getToolTargetPath`, `safeRepoPathForToolInput`, and `safeToolUseId` to group `payload.tool_calls` by repo-relative path while preserving batch order.

- [ ] **Step 2: Implement first-preimage diff**

For each group:

- read only the first call's preimage;
- use `fingerprint.typeEscapes` as before multiset when present;
- use empty baseline and mark incomplete when missing or when type escape facts are unavailable;
- read disk postimage once;
- extract postimage type escapes;
- compute net added occurrences by `occurrenceKey`.

- [ ] **Step 3: Append and drain**

Append one event per added occurrence key, cleanup all group preimages, then call `drainDueEventReminders` with `hookEventName: "PostToolBatch"`.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-hook-post-write-lite.mjs
```

Expected: all assertions pass.

## Task 4: Packaging, Docs, And Validation

**Files:**
- Modify: `scripts/update-test-doc.mjs`
- Generated: `tests/README.md`
- Generated mirror: `skills/lumin-repo-lens-lab/_engine/lib/hook-preimage-store.mjs`
- Generated mirror: `skills/lumin-repo-lens-lab/_engine/lib/hook-post-write-lite.mjs`

- [ ] **Step 1: Register the test doc**

Add:

```js
'test-hook-post-write-lite.mjs': 'auto-hook Phase 1F post-write-lite silent-new event generation core',
```

- [ ] **Step 2: Regenerate generated surfaces**

Run:

```bash
npm run build:skill
npm run update-test-doc
```

- [ ] **Step 3: Run targeted validation**

Run:

```bash
node tests/test-hook-post-write-lite.mjs
node tests/test-hook-preimage-store.mjs
node tests/test-hook-event-store.mjs
node tests/test-hook-event-drain-renderer.mjs
node tests/test-plugin-package.mjs
npm run check
npm run lint
npm run check:test-doc
npm run check:public-plugin
```

- [ ] **Step 4: Commit**

Run:

```bash
git add _lib/hook-preimage-store.mjs _lib/hook-post-write-lite.mjs tests/test-hook-post-write-lite.mjs scripts/update-test-doc.mjs tests/README.md skills/lumin-repo-lens-lab/_engine/lib/hook-preimage-store.mjs skills/lumin-repo-lens-lab/_engine/lib/hook-post-write-lite.mjs docs/superpowers/plans/2026-05-08-auto-hook-post-write-lite-core.md
git commit -m "Add auto hook post-write lite core"
```

## Self-Review

- No hook activation is added.
- No hook script wrapper is added.
- No raw source text is persisted in preimage records.
- Later same-file preimages are not used as fallback baselines.
- Dedupe keys are opaque and never parsed by delimiter.
