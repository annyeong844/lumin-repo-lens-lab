# Auto Hook Preimage Store Implementation Plan

> **For agentic workers:** Implement this plan task-by-task with TDD. Subagents are optional only when the human explicitly asks for parallel workers. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the production preimage store that future `PreToolUse` and `PostToolBatch` hooks will share, without enabling any hook behavior.

**Architecture:** This slice owns only session-scoped preimage state under `<auditRoot>/sessions/<sid>/preimages/`. It reuses Phase 1A path/id safety, writes atomically, reads defensively, and cleans up by tool-use id or age. Event ledger, escape diffing, reminders, ACK handling, and hook activation stay out of scope.

**Tech Stack:** Node.js ESM, built-in filesystem/crypto APIs, existing `_lib/atomic-write.mjs`, direct script tests.

---

## File Structure

- Create `_lib/hook-preimage-store.mjs`
  - Owns preimage path construction, capture, read, cleanup, and orphan cleanup.
  - Exposes `preimagePath`, `capturePreimage`, `readPreimage`, `cleanupPreimage`, and `cleanupOldPreimages`.
  - Stores only source fingerprints and bounded metadata, not raw source text.
- Create `tests/test-hook-preimage-store.mjs`
  - Covers existing file capture, missing file capture, malformed read fallback, cleanup, old-file cleanup, and path traversal rejection.
- Modify `scripts/hook-doctor.mjs`
  - Reports whether the session preimage directory exists for the current default session, without creating it.
- Modify `tests/test-hook-doctor.mjs`
  - Pins the new doctor output.
- Modify `scripts/update-test-doc.mjs`
  - Adds a description for the new preimage-store test.

## Data Contract

Preimage JSON shape:

```json
{
  "schemaVersion": "hook-preimage.v1",
  "capturedAt": "2026-05-08T00:00:00.000Z",
  "repoRel": "src/a.ts",
  "toolUseId": "tool_abc",
  "absent": false,
  "fingerprint": {
    "sha256": "sha256:<64hex>",
    "sizeBytes": 123,
    "mtimeMs": 1234567890
  }
}
```

Missing files use:

```json
{
  "absent": true,
  "fingerprint": null
}
```

## Task 1: Preimage Store Tests

**Files:**
- Create: `tests/test-hook-preimage-store.mjs`

- [ ] **Step 1: Write failing tests**

Create tests that assert:

```js
capturePreimage({ auditRoot, sid, tid, safe }) writes a JSON file
readPreimage(auditRoot, sid, tid) returns the captured record
existing file records sha256:size/mtime fingerprint and no raw source text
missing file records absent:true and fingerprint:null
cleanupPreimage removes one tool-use preimage
cleanupOldPreimages removes files older than maxAgeMs
invalid sid/tid or traversal repoRel throws or returns null safely
malformed JSON read returns null instead of throwing
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-preimage-store.mjs
```

Expected: fails because `_lib/hook-preimage-store.mjs` does not exist.

## Task 2: Preimage Store Implementation

**Files:**
- Create: `_lib/hook-preimage-store.mjs`

- [ ] **Step 1: Implement path construction**

`preimagePath(auditRoot, sid, tid)` must reject unsafe ids and return:

```text
<auditRoot>/sessions/<sid>/preimages/<tid>.json
```

- [ ] **Step 2: Implement capture**

`capturePreimage({ auditRoot, sid, tid, safe, now })` must:

- require `safe.ok === true`;
- require syntactically safe `safe.repoRel`;
- create the preimage directory only when writing;
- hash current file bytes when `safe.exists && safe.kind === "file"`;
- record `absent:true` when the file is missing;
- use `atomicWrite`;
- never store raw source text.

- [ ] **Step 3: Implement read and cleanup**

`readPreimage` returns parsed preimage records or `null` for missing/malformed/unsafe input.
`cleanupPreimage` removes only the matching preimage file.
`cleanupOldPreimages` deletes old `*.json` preimages under one session's preimage dir.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-hook-preimage-store.mjs
```

Expected: all assertions pass.

## Task 3: Doctor Integration

**Files:**
- Modify: `scripts/hook-doctor.mjs`
- Modify: `tests/test-hook-doctor.mjs`

- [ ] **Step 1: Write failing doctor assertion**

Add an assertion that doctor output includes:

```text
preimageStore:
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-doctor.mjs
```

Expected: fails until the doctor reports preimage store status.

- [ ] **Step 3: Implement doctor status**

Doctor should print whether `.audit/sessions/default-session/preimages` currently exists. It must not create the directory.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-hook-doctor.mjs
```

Expected: doctor test passes.

## Task 4: Packaging, Docs, And Verification

**Files:**
- Modify: `scripts/update-test-doc.mjs`
- Generated: `tests/README.md`
- Generated mirror: `skills/lumin-repo-lens-lab/_engine/lib/hook-preimage-store.mjs`

- [ ] **Step 1: Add test docs**

Add:

```js
'test-hook-preimage-store.mjs': 'auto-hook Phase 1B session preimage store',
```

- [ ] **Step 2: Regenerate skill mirror and test docs**

Run:

```bash
npm run build:skill
npm run update-test-doc
```

- [ ] **Step 3: Run targeted validation**

Run:

```bash
node tests/test-hook-preimage-store.mjs
node tests/test-hook-doctor.mjs
node tests/test-plugin-package.mjs
npm run check
npm run lint
```

- [ ] **Step 4: Commit**

Run:

```bash
git add _lib/hook-preimage-store.mjs scripts/hook-doctor.mjs tests/test-hook-preimage-store.mjs tests/test-hook-doctor.mjs scripts/update-test-doc.mjs tests/README.md skills/lumin-repo-lens-lab/_engine/lib/hook-preimage-store.mjs docs/superpowers/plans/2026-05-08-auto-hook-preimage-store.md
git commit -m "Add auto hook preimage store"
```

## Self-Review

- This plan does not enable hooks.
- This plan does not implement event-store, escape diffing, reminders, or ACK handling.
- The preimage store is fully functional for future hook scripts and has direct tests.
- Preimage files store hashes and bounded metadata only, never raw source text.
