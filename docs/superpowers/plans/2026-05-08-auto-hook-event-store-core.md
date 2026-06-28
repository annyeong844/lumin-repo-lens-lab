# Auto Hook Event Store Core Implementation Plan

> **For agentic workers:** Implement this plan task-by-task with TDD. Subagents are optional only when the human explicitly asks for parallel workers. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the session-scoped event ledger that future auto hooks will use for silent-new reminders and ACK suppression, without enabling hook execution.

**Architecture:** This slice owns only event persistence and delivery bookkeeping under `<auditRoot>/sessions/<sid>/event-store/`. It appends/merges events by dedupe key, preserves `until_ack` tombstones after acknowledgement cleanup, claims due deliveries atomically enough for a single-process hook runner, and sanitizes render data at append time. Hook scripts, escape diffing, Markdown rendering, and ACK parsing stay out of scope.

**Tech Stack:** Node.js ESM, `_lib/atomic-write.mjs`, sync filesystem APIs, direct script tests.

---

## File Structure

- Create `_lib/hook-event-store.mjs`
  - Owns event-store paths, state read/write, data sanitization, append/dedupe, delivery claiming, delivery marking, acknowledgement, and cleanup.
  - Exposes `eventStoreDir`, `appendEventIfNotDeduped`, `claimDueDeliveriesAndAdvanceCursor`, `markDelivered`, `markAcknowledged`, `cleanupAckedEntries`, and `readEventStoreState`.
- Create `tests/test-hook-event-store.mjs`
  - Covers append, merge, ack suppression, tombstone suppression, due delivery cursor behavior, render data sanitization, and malformed store fallback.
- Modify `scripts/hook-doctor.mjs`
  - Reports whether the default-session event store exists, without creating it.
- Modify `tests/test-hook-doctor.mjs`
  - Pins the new doctor output.
- Modify `scripts/update-test-doc.mjs`
  - Adds the new test description.

## State Contract

Store files:

```text
<auditRoot>/sessions/<sid>/event-store/ledger.json
<auditRoot>/sessions/<sid>/event-store/.event-store.lock
```

`ledger.json`:

```json
{
  "schemaVersion": "hook-event-store.v1",
  "entries": [],
  "cursor": {
    "lastClaimedAt": null
  }
}
```

Entry minimum shape:

```json
{
  "id": "evt_...",
  "active": true,
  "session_id": "sid_123",
  "kind": "silent-new",
  "severity": "warn",
  "ack_required": true,
  "delivery_policy": "until_ack",
  "diff_key": "opaque",
  "dedupe_key": "opaque",
  "data": {
    "file": "src/a.ts",
    "line": 12,
    "escape_kind": "as-any",
    "snippet": "value as any",
    "enclosing_symbol": "parse",
    "matched_line_text": "const value = raw as any"
  },
  "created_at": "...",
  "first_seen_at": "...",
  "last_seen_at": "...",
  "occurrence_count": 1,
  "delivered_count": 0,
  "delivered_at": null,
  "next_redeliver_at": null,
  "acknowledged": false,
  "acknowledged_at": null,
  "ack_source": null,
  "archived_at": null,
  "archive_reason": null
}
```

## Task 1: Event Store Tests

**Files:**
- Create: `tests/test-hook-event-store.mjs`

- [ ] **Step 1: Write failing tests**

Create tests that assert:

```js
appendEventIfNotDeduped appends a new active event
second append with same dedupe_key merges occurrence_count and data
markAcknowledged marks an event acknowledged
append after active acknowledgement suppresses same dedupe key
cleanupAckedEntries turns acknowledged until_ack entries into tombstones
append after until_ack tombstone suppresses same dedupe key forever
claimDueDeliveriesAndAdvanceCursor returns due active unacked entries only
markDelivered increments delivered_count and sets next_redeliver_at
append sanitizes snippet, matched_line_text, and enclosing_symbol
malformed ledger read degrades to empty state
unsafe sid returns empty/read-safe behavior rather than path traversal
fresh lock times out without corrupting the store
stale lock is removed and the write proceeds
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-event-store.mjs
```

Expected: fails because `_lib/hook-event-store.mjs` does not exist.

## Task 2: Event Store Implementation

**Files:**
- Create: `_lib/hook-event-store.mjs`

- [ ] **Step 1: Implement state paths and safe reads**

`eventStoreDir(auditRoot, sid)` must reject unsafe session ids.
`readEventStoreState(auditRoot, sid)` must return an empty valid state for missing or malformed stores.
Write operations must acquire `.event-store.lock` with timeout and stale-lock recovery.

- [ ] **Step 2: Implement append/dedupe**

`appendEventIfNotDeduped(auditRoot, sid, event)` must:

- create store directory only when writing;
- merge active unacknowledged entries by `dedupe_key`;
- suppress active acknowledged entries;
- suppress inactive `until_ack` tombstones;
- allow new entries otherwise;
- sanitize `event.data`.
- return `{appended:false, eventId:null, reason:'lock-timeout'}` if the lock cannot be acquired.

- [ ] **Step 3: Implement delivery and ack operations**

Implement:

- `claimDueDeliveriesAndAdvanceCursor(auditRoot, sid, { limit, now })`
- `markDelivered(auditRoot, sid, eventId, { now, redeliverAfterMs })`
- `markAcknowledged(auditRoot, sid, eventId, ackSource, { now })`
- `cleanupAckedEntries(auditRoot, sid, { now })`

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-hook-event-store.mjs
```

Expected: all assertions pass.

## Task 3: Doctor Integration

**Files:**
- Modify: `scripts/hook-doctor.mjs`
- Modify: `tests/test-hook-doctor.mjs`

- [ ] **Step 1: Add failing doctor assertion**

Doctor output should include:

```text
eventStore:
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-doctor.mjs
```

Expected: fails until doctor reports event-store status.

- [ ] **Step 3: Add status**

Doctor should print whether `.audit/sessions/default-session/event-store` exists. It must not create the directory.

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
- Generated mirror: `skills/lumin-repo-lens-lab/_engine/lib/hook-event-store.mjs`

- [ ] **Step 1: Add test docs**

Add:

```js
'test-hook-event-store.mjs': 'auto-hook Phase 1C session event store core',
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
node tests/test-hook-event-store.mjs
node tests/test-hook-doctor.mjs
node tests/test-plugin-package.mjs
npm run check
npm run lint
npm run check:test-doc
npm run check:public-plugin
```

- [ ] **Step 4: Commit**

Run:

```bash
git add _lib/hook-event-store.mjs scripts/hook-doctor.mjs tests/test-hook-event-store.mjs tests/test-hook-doctor.mjs scripts/update-test-doc.mjs tests/README.md skills/lumin-repo-lens-lab/_engine/lib/hook-event-store.mjs docs/superpowers/plans/2026-05-08-auto-hook-event-store-core.md
git commit -m "Add auto hook event store core"
```

## Self-Review

- No hook activation is added.
- No reminder renderer or ACK parser is added.
- Event data is sanitized before persistence.
- `until_ack` acknowledged identities remain suppressed through tombstones.
