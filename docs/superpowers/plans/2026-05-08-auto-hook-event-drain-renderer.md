# Auto Hook Event Drain Renderer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the auto-hook event drainer and reminder renderer core that turns due ledger entries into hook `additionalContext`, without activating any hook.

**Architecture:** This slice reads the existing session event store, renders due events from ledger snapshots only, and marks rendered events delivered before returning hook output to the caller. It does not add hook scripts, ACK parsing, post-write diffing, or any `hooks/hooks.json` activation.

**Tech Stack:** Node.js ESM, sync filesystem guards, `_lib/hook-event-store.mjs`, direct Node test scripts.

---

## File Structure

- Create `_lib/hook-event-renderer.mjs`
  - Owns ledger snapshot to reminder text rendering.
  - Exposes `renderEventReminderContext(events, opts)`.
  - Enforces deterministic order, complete-event budget handling, ACK instruction wording, and safe text caps.
- Create `_lib/hook-event-drain.mjs`
  - Owns read-only empty-store guard, due-event claiming, rendering, and delivered marking.
  - Exposes `drainDueEventReminders(auditRoot, sid, opts)`.
  - Returns hook output JSON but does not print or install hooks.
- Create `tests/test-hook-event-drain-renderer.mjs`
  - Covers renderer wording, budget behavior, no-store no-write behavior, due delivery, delivered marking, and redelivery throttling.
- Modify `scripts/update-test-doc.mjs`
  - Adds the new test description.
- Generated:
  - `tests/README.md`
  - `skills/lumin-repo-lens-lab/_engine/lib/hook-event-renderer.mjs`
  - `skills/lumin-repo-lens-lab/_engine/lib/hook-event-drain.mjs`

## Contract

`renderEventReminderContext(events, opts)` returns:

```js
{
  text: '...',
  eventIds: ['evt_...'],
  omittedCount: 0
}
```

Rules:

- Empty input returns empty text and no ids.
- Text starts with `[audit · observed in this/previous tool batch]`.
- Every rendered event includes its event id.
- ACK guidance uses `AUDIT_ACK <event id> <intentional|fixed|noted>`.
- `occurrence_count > 1` renders as a matching-escapes summary, not an exact line assertion.
- Output is capped at `maxChars` by omitting whole event blocks. It must not cut an event id in half.

`drainDueEventReminders(auditRoot, sid, opts)` returns:

```js
{
  emitted: true,
  output: {
    hookSpecificOutput: {
      hookEventName: 'UserPromptSubmit',
      additionalContext: '...'
    }
  },
  eventIds: ['evt_...'],
  omittedCount: 0
}
```

Rules:

- If no `ledger.json` exists, it returns `emitted:false` and does not create event-store directories.
- It calls `claimDueDeliveriesAndAdvanceCursor(...)` only after confirming a ledger file exists.
- It renders ledger snapshots only; it never re-reads source files.
- It calls `markDelivered(...)` for rendered event ids before returning output.
- It does not mark omitted events delivered.
- Hook activation remains out of scope.

## Task 1: RED Tests

**Files:**
- Create: `tests/test-hook-event-drain-renderer.mjs`

- [ ] **Step 1: Add renderer and drainer tests**

Add tests that import:

```js
import { drainDueEventReminders } from '../_lib/hook-event-drain.mjs';
import { renderEventReminderContext } from '../_lib/hook-event-renderer.mjs';
```

Assertions:

- empty renderer input returns no text;
- a `silent-new` event renders file, line, snippet, event id, and ACK instruction;
- aggregate events use "matching escapes near" wording;
- budget handling omits whole events and reports `omittedCount`;
- draining a missing store emits nothing and creates no event-store directory;
- draining a due event returns hook output JSON and marks the event delivered;
- a second drain before `next_redeliver_at` emits nothing.

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-event-drain-renderer.mjs
```

Expected: fail with missing `_lib/hook-event-drain.mjs` or `_lib/hook-event-renderer.mjs`.

## Task 2: Renderer Implementation

**Files:**
- Create: `_lib/hook-event-renderer.mjs`

- [ ] **Step 1: Implement `renderEventReminderContext`**

Implementation requirements:

- sort events by `created_at`, then `id`;
- build complete event blocks;
- include one ACK instruction block after rendered events;
- cap output by `maxChars`, default `2048`;
- return rendered event ids only for blocks included in `text`.

- [ ] **Step 2: Verify renderer tests**

Run:

```bash
node tests/test-hook-event-drain-renderer.mjs
```

Expected: renderer assertions pass; drainer assertions still fail until Task 3.

## Task 3: Drainer Implementation

**Files:**
- Create: `_lib/hook-event-drain.mjs`

- [ ] **Step 1: Implement read-only empty-store guard**

Use `eventStoreDir(auditRoot, sid)` to locate `ledger.json`, but do not create the directory. If the path is unsafe or the ledger is missing, return:

```js
{ emitted: false, output: null, eventIds: [], omittedCount: 0 }
```

- [ ] **Step 2: Implement due-event drain**

Call `claimDueDeliveriesAndAdvanceCursor`, render snapshots, then call `markDelivered` for every rendered event id before returning hook output.

- [ ] **Step 3: Verify GREEN**

Run:

```bash
node tests/test-hook-event-drain-renderer.mjs
```

Expected: all assertions pass.

## Task 4: Packaging, Docs, And Validation

**Files:**
- Modify: `scripts/update-test-doc.mjs`
- Generated: `tests/README.md`
- Generated mirror: `skills/lumin-repo-lens-lab/_engine/lib/hook-event-renderer.mjs`
- Generated mirror: `skills/lumin-repo-lens-lab/_engine/lib/hook-event-drain.mjs`

- [ ] **Step 1: Register the test doc**

Add:

```js
'test-hook-event-drain-renderer.mjs': 'auto-hook Phase 1D event drainer and reminder renderer core',
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
node tests/test-hook-event-drain-renderer.mjs
node tests/test-hook-event-store.mjs
node tests/test-plugin-package.mjs
npm run check
npm run lint
npm run check:test-doc
npm run check:public-plugin
```

- [ ] **Step 4: Commit**

Run:

```bash
git add _lib/hook-event-renderer.mjs _lib/hook-event-drain.mjs tests/test-hook-event-drain-renderer.mjs scripts/update-test-doc.mjs tests/README.md skills/lumin-repo-lens-lab/_engine/lib/hook-event-renderer.mjs skills/lumin-repo-lens-lab/_engine/lib/hook-event-drain.mjs docs/superpowers/plans/2026-05-08-auto-hook-event-drain-renderer.md
git commit -m "Add auto hook event drain renderer"
```

## Self-Review

- No hook activation is added.
- No ACK observer is added.
- No post-write diffing is added.
- Renderer uses ledger snapshots only.
- Drainer does not create an event store for empty sessions.
- Delivered marking happens before hook output is returned.
