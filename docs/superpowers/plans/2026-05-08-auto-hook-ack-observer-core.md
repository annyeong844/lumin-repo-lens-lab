# Auto Hook ACK Observer Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the Stop-hook ACK observer core that recognizes explicit `AUDIT_ACK` lines and marks matching event-store entries acknowledged, without activating hooks.

**Architecture:** This slice is parser-first. A small ACK parser extracts line-based sentinel acknowledgements from assistant text while ignoring code fences, inline code, indented code blocks, and blockquotes. A small observer then prefers `payload.last_assistant_message`, falls back to caller-provided transcript text, and calls the existing event-store `markAcknowledged` API.

**Tech Stack:** Node.js ESM, `_lib/hook-event-store.mjs`, direct Node test scripts, generated skill mirror.

---

## File Structure

- Create `_lib/hook-ack-observer.mjs`
  - Owns ACK text extraction and event-store acknowledgement application.
  - Exposes `parseAuditAckLines(text)` and `observeStopAcknowledgements(auditRoot, sid, payload, opts)`.
- Create `tests/test-hook-ack-observer.mjs`
  - Covers valid ACKs, invalid intents, code-fenced ACKs, inline-code ACKs, indented ACKs, blockquoted ACKs, last-assistant-message preference, transcript fallback, unsafe session handling, and event-store mutation.
- Modify `scripts/update-test-doc.mjs`
  - Adds the new test description.
- Generated:
  - `tests/README.md`
  - `skills/lumin-repo-lens-lab/_engine/lib/hook-ack-observer.mjs`

## Contract

`parseAuditAckLines(text)` returns an array:

```js
[
  { eventId: 'evt_abc123', ackSource: 'fixed', line: 4 }
]
```

Rules:

- The sentinel must occupy a line by itself except surrounding whitespace.
- Accepted form: `AUDIT_ACK <event-id> <intentional|fixed|noted>`.
- `event-id` must be a safe id.
- ACKs inside closed code fences, unclosed-to-EOF code fences, inline backticks, indented 4-space code blocks, or leading-space blockquotes are ignored.
- Invalid intents are ignored, not reported as acknowledgements.

`observeStopAcknowledgements(auditRoot, sid, payload, opts)` returns:

```js
{
  observed: 1,
  acknowledged: 1,
  ignored: 0,
  eventIds: ['evt_abc123']
}
```

Rules:

- It reads `payload.last_assistant_message` first when it is a string.
- If missing, it may use `opts.transcriptText` as an explicit transcript fallback for this core slice.
- It never reads source files.
- It calls `markAcknowledged` for parsed ACKs.
- It exits safely for unsafe session ids or missing event stores by returning zero acknowledged events.
- Hook activation remains out of scope.

## Task 1: RED Tests

**Files:**
- Create: `tests/test-hook-ack-observer.mjs`

- [ ] **Step 1: Write parser and observer tests**

Assertions:

- a valid ACK line parses;
- invalid intent does not parse;
- ACK inside fenced code does not parse;
- ACK inside an unclosed fence to EOF does not parse;
- ACK inside inline backticks does not parse;
- ACK in an indented code block does not parse;
- ACK in a leading-space blockquote does not parse;
- observer marks an event acknowledged using `last_assistant_message`;
- observer prefers `last_assistant_message` over `opts.transcriptText`;
- observer uses `opts.transcriptText` when `last_assistant_message` is absent;
- unsafe session id returns zero acknowledgement.

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-ack-observer.mjs
```

Expected: fails because `_lib/hook-ack-observer.mjs` does not exist.

## Task 2: ACK Parser Implementation

**Files:**
- Create: `_lib/hook-ack-observer.mjs`

- [ ] **Step 1: Implement `parseAuditAckLines`**

Use a single line scanner with `inFence` state. Fence starts when a line begins with three or more backticks or tildes after optional whitespace. While in a fence, ignore all lines until a matching closing fence. Ignore lines that begin with four spaces, begin with optional spaces followed by `>`, or contain backticks around the sentinel.

- [ ] **Step 2: Verify parser tests**

Run:

```bash
node tests/test-hook-ack-observer.mjs
```

Expected: parser tests pass; observer tests fail until Task 3.

## Task 3: Observer Implementation

**Files:**
- Modify: `_lib/hook-ack-observer.mjs`

- [ ] **Step 1: Implement `observeStopAcknowledgements`**

Pick source text:

```js
const text = typeof payload?.last_assistant_message === 'string'
  ? payload.last_assistant_message
  : typeof opts.transcriptText === 'string'
    ? opts.transcriptText
    : '';
```

Parse ACKs, call `markAcknowledged(auditRoot, sid, eventId, ackSource, opts)`, and count successful acknowledgements.

- [ ] **Step 2: Verify GREEN**

Run:

```bash
node tests/test-hook-ack-observer.mjs
```

Expected: all assertions pass.

## Task 4: Packaging, Docs, And Validation

**Files:**
- Modify: `scripts/update-test-doc.mjs`
- Generated: `tests/README.md`
- Generated mirror: `skills/lumin-repo-lens-lab/_engine/lib/hook-ack-observer.mjs`

- [ ] **Step 1: Register the test doc**

Add:

```js
'test-hook-ack-observer.mjs': 'auto-hook Phase 1E Stop ACK observer core',
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
node tests/test-hook-ack-observer.mjs
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
git add _lib/hook-ack-observer.mjs tests/test-hook-ack-observer.mjs scripts/update-test-doc.mjs tests/README.md skills/lumin-repo-lens-lab/_engine/lib/hook-ack-observer.mjs docs/superpowers/plans/2026-05-08-auto-hook-ack-observer-core.md
git commit -m "Add auto hook ACK observer core"
```

## Self-Review

- No hook activation is added.
- Parser ignores code contexts rather than trusting prose.
- Observer prefers `last_assistant_message` and has only an explicit transcript-text fallback.
- Observer uses event-store APIs rather than editing ledger files directly.
