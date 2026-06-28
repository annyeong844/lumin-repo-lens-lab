# Vitest Pre-Write Render Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-pre-write-render.mjs`.

---

## Purpose

This review decides whether `tests/test-pre-write-render.mjs` can move as one
narrow Lane C Vitest mirror batch. It does not add a Vitest suite.

The suite is acceptable as a single-suite batch because it protects only the
pre-write advisory renderer boundary:

- Markdown section placement and wording for existing names, new files,
  unavailable evidence, canonical drift, watch-for cues, planned type escapes,
  cue cards, and service-operation sibling review cues;
- JSON pass-through shape for advisory metadata and cue policy evidence;
- citation and confidence labels that keep grounded evidence, degraded search
  hints, and unavailable evidence distinct.

This batch must stay separate from pre-write lookup policy, cue-tier promotion,
intent parsing, canonical parser behavior, resolver behavior, deadness/ranking,
and full audit orchestration. The future mirror must test rendering behavior
from fixed advisory objects; it must not re-evaluate advisory policy.

## Reviewed Evidence

| Suite                             | Preserved Node Command                 | Proposed Focused Vitest Command        | Surface Under Review         |
| --------------------------------- | -------------------------------------- | -------------------------------------- | ---------------------------- |
| `tests/test-pre-write-render.mjs` | `node tests/test-pre-write-render.mjs` | `npm run test:vitest:pre-write-render` | pre-write advisory rendering |

Current preserved-command evidence on 2026-05-16:

```text
node tests/test-pre-write-render.mjs
98 passed, 0 failed
```

Goal lane: Lane C, pre-write lifecycle. This review covers only advisory
Markdown/JSON rendering.

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The future implementation PR may add one focused mirror for this suite because
the current Node test already uses fixed advisory fixtures and direct
`renderMarkdown` / `renderJson` calls. The mirror must keep every current
claim-bearing fixture represented as named Vitest cases and must keep the Node
entrypoint runnable.

## Protected Invariants

The future Vitest mirror must preserve these renderer contracts:

- the advisory title and lookup sections remain present for result fixtures;
- `EXISTS` rows render identity, owner file, grounded fan-in, value/type/broad
  fan-in space, and fan-in citations;
- severely-any-contaminated identities render raw measurements and a
  warn-on-reuse recommendation without turning the measurement into degraded
  proof;
- `NOT_OBSERVED` name lookups render near-name and semantic hints under search
  hints, not under reuse candidates;
- search hints remain degraded reviewer cues and never become grounded reuse
  claims;
- canonical AST-absent rows render `[확인 불가]` evidence without leaking the
  literal `CANONICAL DRIFT:` outside the canonical drift section;
- `EXISTS_MULTIPLE` renders every identity side by side with its own owner and
  fan-in evidence;
- planned type escapes render escape kind, location hint, code shape, reason,
  alternative considered, and grounded intent citation;
- empty planned type escapes render the zero-planned note with the canonical
  escape-kind/fact-model reference;
- file lookups keep `NEW_FILE`, `FILE_STATUS_UNKNOWN`, existing file, hub, and
  domain-cluster cues in the correct sections;
- dependency lookups keep available dependencies under reuse candidates while
  unavailable import-graph evidence remains explicitly unavailable;
- shape hash, shape policy, and watch-for hints render as review cues with
  citations rather than proof of equivalence;
- capability-absent notes render once while per-identity fan-in citations still
  render;
- canonical drift renders only when drift exists and cites both canonical and
  AST owners when present;
- `renderJson` preserves invocation id, intent, defaults, warnings, cue cards,
  suppressed cues, unavailable evidence, and cue policy metadata;
- intent warning notes render compact default-schema information;
- grounded fact cue cards, agent review cues, muted cues, unavailable evidence,
  and evidence availability warnings remain in distinct lanes;
- service-operation sibling review cues use explicit review wording, cite
  `pre-write-advisory.json / lookups[].serviceOperationSiblingPolicy.promoted`,
  expose policy version, shared domain tokens, operation family, locality, and
  supporting suppressed reasons;
- muted service-operation details remain hidden from default Markdown;
- exact cue-covered candidates render once in default Markdown.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- any claim-bearing Markdown line without a nearby citation must fail;
- near-name or semantic hints appearing under "Already exists" must fail;
- unavailable dependency evidence rendered as zero or clean evidence must fail;
- `FILE_STATUS_UNKNOWN` rendered as a `NEW_FILE` claim must fail;
- canonical drift wording leaking outside the canonical drift section must
  fail;
- an empty drift array rendering a canonical drift section must fail;
- planned type escapes losing order or metadata must fail;
- `renderJson` dropping optional arrays or warning metadata must fail;
- muted cues appearing in default Markdown must fail;
- service-operation sibling cues using strong action wording such as "safe",
  "exists", "reuse", "equivalent", "should call", or "blocking failure" must
  fail;
- evidence availability warnings appearing after lookup sections must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is direct imports from `_lib/pre-write-render.mjs` and
  fixed advisory objects inside the test file.
- The mirror may share local helpers for advisory fixture construction,
  claim-bearing citation checks, Markdown section extraction, and disallowed
  wording checks.
- Shared helpers must not decide lookup results, fan-in confidence,
  contamination policy, canonical owner resolution, cue-tier promotion,
  service-operation sibling policy, resolver behavior, deadness/ranking,
  unavailable evidence classification, or canonical drift detection.
- The mirror must not absorb `tests/test-pre-write-cue-tiers.mjs`,
  `tests/test-pre-write-lookup-name.mjs`,
  `tests/test-pre-write-advisory-artifact.mjs`,
  `tests/test-pre-write-cli.mjs`, `tests/test-pre-write-integration.mjs`,
  intent/canonical parser suites, resolver suites, deadness/ranking suites, or
  performance/incremental suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/pre-write-render.test.mjs`,
2. `npm run test:vitest:pre-write-render`,
3. candidate-board updates moving `tests/test-pre-write-render.mjs` from
   `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves every
current fixture group as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, `npm run test:vitest`, doc-script checks,
formatting checks, and `npm test` before completion.
