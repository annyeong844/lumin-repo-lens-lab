# Vitest Canon Drift Contracts Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-canon-drift-parser-contract.mjs`
> - `tests/test-canonical-fact-model-drift.mjs`

---

## Purpose

This review decides whether the canon drift contract suites can move as one
Lane B Vitest mirror batch. It does not add Vitest suites. The goal is to
preserve the canonical documentation-to-renderer drift pins that keep Markdown
contracts and fact-model schema contracts synchronized, without absorbing
producer behavior, resolver behavior, deadness/ranking, performance, or full
audit orchestration.

The batch is acceptable because both suites are contract-drift guards over
canonical documents and exported helper behavior:

- `test-canon-drift-parser-contract.mjs` protects the table header contracts
  between the P3 canon renderers and `canonical/canon-drift.md` §5;
- `test-canonical-fact-model-drift.mjs` protects the `canonical/fact-model.md`
  §3.9 type-escape schema, escape-kind enum order, amendment text, and
  `PLANNED_ESCAPE_KEYS` normalization contract.

The future mirror should keep those contract pins visible. Shared setup may
read canonical Markdown, call exported renderers, extract fixture-controlled
table headers, and import constants, but it must not decide renderer
semantics, fact-model meaning, producer output, pre-write policy, resolver
meaning, or action-safety proof.

## Reviewed Evidence

| Suite                                        | Preserved Node Command                            | Proposed Focused Vitest Command                   | Surface Under Review                             |
| -------------------------------------------- | ------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------ |
| `tests/test-canon-drift-parser-contract.mjs` | `node tests/test-canon-drift-parser-contract.mjs` | `npm run test:vitest:canon-drift-parser-contract` | canon renderer table headers vs canon-drift docs |
| `tests/test-canonical-fact-model-drift.mjs`  | `node tests/test-canonical-fact-model-drift.mjs`  | `npm run test:vitest:canonical-fact-model-drift`  | fact-model type-escape schema drift              |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane B, canon/check-canon drift contract family.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add both focused mirrors together because they
share the canonical-document drift contract surface. The PR must keep every
Node entrypoint runnable and must not absorb producer integration, resolver,
generated/framework, deadness/ranking, performance, incremental cache, or full
audit suites.

## Protected Invariants

The future Vitest batch must preserve these contracts:

- fixture-controlled Markdown header extraction remains narrow and is not a
  general Markdown parser;
- `renderTypeOwnership` emits the documented 7-column type-ownership table
  header;
- `canonical/canon-drift.md` §5.a names every type-ownership column;
- `renderHelperRegistry` emits the documented 8-column helper-registry table
  header;
- `canonical/canon-drift.md` §5.b names every helper-registry column;
- `renderTopology` emits the documented topology inventory, cross-edge, and
  oversize table headers;
- `canonical/canon-drift.md` §5.c names every topology table column;
- `renderNaming` emits the documented file-cohort and symbol-cohort table
  headers;
- `canonical/canon-drift.md` §5.d names every naming table column;
- `canonical/fact-model.md` §3.9 contains the canonical `escapeKind is one of`
  block;
- the type-escape `escapeKind` list keeps the current 11 values and order;
- the §3.9 JSON example keeps `occurrenceKey` and `normalizedCodeShape`;
- the documentation explains `normalizedCodeShape` token-aware/string-literal
  behavior;
- the documentation explains `occurrenceKey` hash composition;
- the documented precedence rules remain present:
  `rest-any-args > explicit-any`, `index-sig-any > explicit-any`,
  `generic-default-any > explicit-any`, `angle-any > explicit-any`, and
  `as-unknown-as-T > as-any`;
- the P2-0 amendment date remains visible;
- `PLANNED_ESCAPE_KEYS` required and optional key sets stay synchronized with
  the validator normalization path;
- `validateIntent` normalized `plannedTypeEscapes` entries surface every
  `PLANNED_ESCAPE_ALL_KEYS` key when provided.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- changing a renderer table header without updating `canonical/canon-drift.md`
  must fail;
- changing `canonical/canon-drift.md` column text without changing the renderer
  must fail;
- deleting a canon-drift column mention must fail;
- broadening the header parser until it silently accepts unrelated Markdown
  must fail;
- reordering or renaming a type-escape `escapeKind` must fail;
- removing `occurrenceKey` or `normalizedCodeShape` from the canonical example
  must fail;
- dropping the normalization explanation or hash composition text must fail;
- omitting a precedence rule must fail;
- changing `PLANNED_ESCAPE_KEYS` without validator normalization parity must
  fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary may read canonical Markdown, call renderer functions,
  parse known table headers, import constants, and run intent validation.
- Shared helper code may reduce repeated header extraction mechanics, but it
  must not decide canon renderer semantics, fact-model schema meaning,
  pre-write policy, resolver behavior, producer behavior, or action-safety
  proof.
- The mirror must not change `canonical/canon-drift.md`,
  `canonical/fact-model.md`, canon renderer output, `pre-write-intent.mjs`, or
  producer output contracts.
- The mirror must not absorb integration, resolver, generated/framework,
  deadness/ranking, performance, incremental cache, or full audit suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Verification Snapshot

The review was grounded by running the preserved Node commands before adding
this page:

```text
node tests/test-canon-drift-parser-contract.mjs # 58 passed, 0 failed
node tests/test-canonical-fact-model-drift.mjs  # 17 passed, 0 failed
```

## Recommendation

Proceed to one implementation PR that adds:

1. `tests/canon-drift-parser-contract.test.mjs`,
2. `tests/canonical-fact-model-drift.test.mjs`,
3. focused `npm run test:vitest:*` commands for each suite,
4. candidate-board updates moving both suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, and the wiki/doc
guards.
