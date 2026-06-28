# Vitest Hook Runtime Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-hook-doctor.mjs`
> - `tests/test-hook-runner-scripts.mjs`
> - `tests/test-hook-path-safety.mjs`
> - `tests/test-hook-id-safety.mjs`
> - `tests/test-hook-event-store.mjs`
> - `tests/test-hook-event-drain-renderer.mjs`
> - `tests/test-hook-preimage-store.mjs`
> - `tests/test-hook-ack-observer.mjs`
> - `tests/test-hook-post-write-lite.mjs`

---

## Purpose

This review decides whether the host-hook runtime suites can move together as
one Lane G Vitest mirror batch. It does not add the Vitest suites.

The batch is acceptable because every candidate protects the local hook
runtime contract rather than analyzer classification, resolver expansion,
deadness, ranking, or package publishing semantics:

- hook discovery and doctor output name the active Phase 1 host events;
- runner scripts accept host JSON payloads, stay quiet on malformed input, and
  connect preimage capture, post-tool reminders, prompt drains, and stop ACKs;
- path and id guards prevent traversal and raw-content leakage;
- preimage and event stores preserve privacy, dedupe, delivery, cleanup, and
  lock recovery behavior;
- renderer/drain code emits scoped ACK reminders without claiming exact source
  line certainty for aggregate events;
- post-write-lite compares preimage fingerprints and emits review reminders
  without running the full analyzer pipeline.

This batch must stay separate from `tests/test-pre-write-inventory-hook.mjs`.
That suite protects pre-write advisory artifact availability and belongs with
the pre-write lifecycle surface, not the host-hook runtime batch.

## Reviewed Evidence

| Suite                                      | Preserved Node Command                          | Proposed Focused Vitest Command                 | Surface Under Review                         |
| ------------------------------------------ | ----------------------------------------------- | ----------------------------------------------- | -------------------------------------------- |
| `tests/test-hook-doctor.mjs`               | `node tests/test-hook-doctor.mjs`               | `npm run test:vitest:hook-doctor`               | `hooks/hooks.json` and `scripts/hook-doctor` |
| `tests/test-hook-runner-scripts.mjs`       | `node tests/test-hook-runner-scripts.mjs`       | `npm run test:vitest:hook-runner-scripts`       | host hook runner scripts                     |
| `tests/test-hook-path-safety.mjs`          | `node tests/test-hook-path-safety.mjs`          | `npm run test:vitest:hook-path-safety`          | hook workspace and tool-path guards          |
| `tests/test-hook-id-safety.mjs`            | `node tests/test-hook-id-safety.mjs`            | `npm run test:vitest:hook-id-safety`            | hook session/tool id guards                  |
| `tests/test-hook-event-store.mjs`          | `node tests/test-hook-event-store.mjs`          | `npm run test:vitest:hook-event-store`          | event ledger, delivery, cleanup, locks       |
| `tests/test-hook-event-drain-renderer.mjs` | `node tests/test-hook-event-drain-renderer.mjs` | `npm run test:vitest:hook-event-drain-renderer` | reminder rendering and due-event drain       |
| `tests/test-hook-preimage-store.mjs`       | `node tests/test-hook-preimage-store.mjs`       | `npm run test:vitest:hook-preimage-store`       | preimage fingerprints and cleanup            |
| `tests/test-hook-ack-observer.mjs`         | `node tests/test-hook-ack-observer.mjs`         | `npm run test:vitest:hook-ack-observer`         | ACK parsing and stop-message observation     |
| `tests/test-hook-post-write-lite.mjs`      | `node tests/test-hook-post-write-lite.mjs`      | `npm run test:vitest:hook-post-write-lite`      | lightweight post-write type-escape reminders |

Current Node evidence checked for this review:

```text
node tests/test-hook-doctor.mjs               # 2 passed, 0 failed
node tests/test-hook-runner-scripts.mjs       # 5 passed, 0 failed
node tests/test-hook-path-safety.mjs          # 4 passed, 0 failed
node tests/test-hook-id-safety.mjs            # 5 passed, 0 failed
node tests/test-hook-event-store.mjs          # 12 passed, 0 failed
node tests/test-hook-event-drain-renderer.mjs # 7 passed, 0 failed
node tests/test-hook-preimage-store.mjs       # 8 passed, 0 failed
node tests/test-hook-ack-observer.mjs         # 11 passed, 0 failed
node tests/test-hook-post-write-lite.mjs      # 6 passed, 0 failed
```

Goal lane: Lane G, public package/plugin/hooks. This review covers only the
host-hook runtime subset of that lane.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all nine mirrors together because they
share the same host-hook runtime boundary and temporary filesystem fixture
style. The mirror must keep every Node entrypoint runnable and must not replace
privacy, traversal, dedupe, or lock-recovery checks with broad happy-path
assertions.

## Protected Invariants

The future Vitest batch must preserve these host-hook contracts:

- `hooks/hooks.json` declares exactly the Phase 1 runner events:
  `PostToolBatch`, `PreToolUse`, `Stop`, and `UserPromptSubmit`;
- `scripts/hook-doctor.mjs` reports workspace root, audit root, active hook
  event count, preimage store, and event store status;
- hook runner scripts exit 0 and stay silent on malformed stdin;
- pre-tool-use captures preimages for mutating tools without printing host
  output;
- post-tool-batch emits delivered silent-new reminders from captured
  preimages;
- stop runner acknowledges valid `AUDIT_ACK` lines;
- user-prompt-submit drains already due events and advances delivery metadata;
- workspace/package/audit roots resolve from repo markers without escaping the
  workspace;
- mutating file tools are recognized while non-mutating tools are ignored;
- syntactic path validation rejects parent traversal, absolute paths, and
  Windows backslash paths without touching disk;
- session and tool ids accept only compact opaque tokens;
- deterministic fallback ids do not include raw source strings;
- event store append, dedupe, acknowledgement, tombstone, delivery, cleanup,
  malformed-ledger, unsafe-id, and stale-lock behavior stay intact;
- reminder rendering includes ACK instructions, omits whole event blocks under
  budget, and uses matching-escapes wording for aggregate events;
- missing event stores drain to no output and create no directories;
- preimage capture stores fingerprints and type-escape facts without raw source
  text;
- preimage reads degrade safely for missing, malformed, or unsafe records;
- cleanup removes only the intended preimage records;
- post-write-lite compares preimage fingerprints, emits silent-new events,
  dedupes repeated occurrences, handles same-file multiple tool calls, and
  intentionally over-warns when preimage evidence is missing.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- malformed host stdin that prints output or exits non-zero must fail;
- wrong hook-event names or missing runner events must fail;
- path traversal, absolute paths, backslash paths, or unsafe ids must fail;
- fallback ids that leak raw `old_string` or `new_string` text must fail;
- event-store duplicate handling, ACK tombstones, redelivery delay, or lock
  recovery regressions must fail;
- malformed ledgers must degrade safely rather than crash;
- reminder rendering must not turn aggregate events into exact line claims;
- budgeted reminder rendering must omit whole event blocks and report omitted
  counts;
- preimage files must not contain raw source text;
- preimage cleanup must not remove unrelated tool records;
- post-write-lite must not create event-store directories for non-mutating
  batches;
- missing preimage evidence must remain visible as incomplete baseline behavior
  rather than clean success.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public hook runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is temporary filesystem roots, `.audit/sessions/*`
  ledgers, and direct module imports from `_lib/hook-*`.
- The mirror may share setup-only fixture helpers inside test files, but those
  helpers must not decide hook delivery, ACK, path safety, id safety,
  preimage, event-store, or renderer semantics.
- The mirror must not absorb `tests/test-pre-write-inventory-hook.mjs`,
  pre/post-write advisory tests, plugin packaging, package publishing, skill
  package/surface tests, analyzer behavior, resolver behavior,
  deadness/ranking, or performance/incremental cache behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/hook-doctor.test.mjs`,
2. `tests/hook-runner-scripts.test.mjs`,
3. `tests/hook-path-safety.test.mjs`,
4. `tests/hook-id-safety.test.mjs`,
5. `tests/hook-event-store.test.mjs`,
6. `tests/hook-event-drain-renderer.test.mjs`,
7. `tests/hook-preimage-store.test.mjs`,
8. `tests/hook-ack-observer.test.mjs`,
9. `tests/hook-post-write-lite.test.mjs`,
10. focused `npm run test:vitest:*` commands for each suite,
11. candidate-board updates moving the nine suites from `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups
represented as named Vitest `it(...)` blocks. It may share local setup helpers
inside a test file, but no shared helper should decide privacy, traversal,
delivery, ACK, preimage, or event-store semantics.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest` and the doc-script checks so the reviewed
runner discovery boundary and wiki references stay current.
