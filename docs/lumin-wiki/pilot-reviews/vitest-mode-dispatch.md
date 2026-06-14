# Vitest Mode Dispatch Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-mode-dispatch.mjs`

---

## Purpose

This review decides whether the pre-write mode dispatcher suite can move as a
narrow Lane C Vitest mirror. It does not add the Vitest suite.

The suite is acceptable because it protects a pure component boundary:
`dispatchMode(userText, cwdMeta)` reads only user text and current-directory
metadata, then returns a trigger or non-trigger result. It does not run the
pre-write advisory pipeline, lookup-name policy, cue-tier promotion, Markdown
rendering, resolver expansion, dead-export ranking, generated-surface
inference, or performance cache behavior.

This batch must stay separate from `tests/test-pre-write-bootstrap.mjs`,
`tests/test-pre-write-cli.mjs`, `tests/test-pre-write-advisory-artifact.mjs`,
`tests/test-pre-write-render.mjs`, `tests/test-pre-write-cue-tiers.mjs`,
broader audit-repo orchestration, resolver behavior, deadness/ranking, and
performance/incremental cache identity.

## Reviewed Evidence

| Suite                          | Preserved Node Command              | Proposed Focused Vitest Command     | Surface Under Review          |
| ------------------------------ | ----------------------------------- | ----------------------------------- | ----------------------------- |
| `tests/test-mode-dispatch.mjs` | `node tests/test-mode-dispatch.mjs` | `npm run test:vitest:mode-dispatch` | pre-write mode dispatch table |

Current Node evidence checked for this review:

```text
node tests/test-mode-dispatch.mjs # 38 passed, 0 failed
```

Goal lane: Lane C, pre/post-write lifecycle. This review covers only the
pre-write mode-dispatch gate, not downstream pre-write execution.

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The future implementation PR may add the focused mirror because the suite is a
single pure-function contract with a canonical markdown drift check and no
pipeline side effects. The mirror must keep the Node entrypoint runnable and
must not widen mode dispatch into a hidden pre-write no-op in downstream code.

## Protected Invariants

The future Vitest mirror must preserve these mode-dispatch contracts:

- the Korean write verb list mirrors `canonical/mode-contract.md` §3.1;
- the English write verb list mirrors `canonical/mode-contract.md` §3.2;
- the Korean and English guard lists mirror `canonical/mode-contract.md` §3.4;
- guard-only requests return `mode: "none"` with an explanatory
  `nonTriggerReason`;
- write verbs in repo context return `mode: "pre-write"`;
- compound guard-plus-verb requests fire pre-write and set
  `compoundGuardPlusVerb: true`;
- missing repo context has highest non-trigger precedence, even when a write
  verb or prose rewrite is present;
- doc/prose rewrite requests such as `README 다듬어줘`,
  `CHANGELOG 업데이트해줘`, `docs/*.md 다듬어줘`, and `rewrite the README`
  return `mode: "none"` with `nonTriggerReason: "prose-rewrite"`;
- comment typo fixes return `mode: "none"` with
  `nonTriggerReason: "comment-typo-fix"`;
- generic bug-fix language that is not a comment typo still fires pre-write;
- pure inspection questions return `mode: "none"`;
- trigger results include `rationale`, `matchedVerbs`, `matchedGuards`, and
  `compoundGuardPlusVerb`, but do not carry `nonTriggerReason`;
- repeated calls with the same input are deterministic.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- drifting verb or guard vocabulary away from `canonical/mode-contract.md`
  must fail;
- treating guard-only requests as pre-write triggers must fail;
- allowing a write verb to fire without repo context must fail;
- treating prose/document editing as pre-write work must fail;
- swallowing generic bug fixes because the word `fix` is also used in comment
  typo cases must fail;
- losing `compoundGuardPlusVerb` on guard-plus-verb requests must fail;
- adding `nonTriggerReason` to trigger results must fail;
- making dispatch stateful across repeated calls must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is direct helper invocation plus read-only canonical
  markdown and source text checks.
- Shared helpers may read markdown fixtures and compare arrays.
- Shared helpers must not decide dispatch semantics, non-trigger precedence,
  repo-context meaning, prose rewrite detection, comment typo detection, or
  compound trigger behavior.
- The mirror must not change `_lib/mode-dispatch.mjs`,
  `canonical/mode-contract.md`, pre-write CLI behavior, cue-tier policy,
  renderer wording, resolver behavior, generated/framework surfaces,
  deadness/ranking, or performance cache identity.
- The mirror must not absorb broader pre-write advisory, bootstrap, CLI,
  render, cue-tier, lookup-name, audit-repo, resolver, generated, or
  deadness/ranking suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/mode-dispatch.test.mjs`,
2. `npm run test:vitest:mode-dispatch`,
3. candidate-board updates moving the suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add named `it(...)` cases for the
current Node assertions. It should run the preserved Node command, the focused
Vitest command, `npm run test:vitest`, `npm test`, and the wiki/doc guards.
