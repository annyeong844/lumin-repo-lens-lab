# Vitest Canon Helper Registry Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-canon-draft-helpers.mjs`
> - `tests/test-canon-draft-helper-registry.mjs`
> - `tests/test-generate-canon-draft-cli-helpers.mjs`
> - `tests/test-check-canon-helpers.mjs`

---

## Purpose

This review decides whether the helper-registry canon suites can move as one
Lane B Vitest mirror batch. It does not add Vitest suites. The goal is to
preserve the helper-registry classifier, aggregation, CLI draft, and drift
contracts without turning them into broad canon or audit behavior tests.

The batch is acceptable because all four suites protect the same canon source
family, but each suite owns a different layer:

- `test-canon-draft-helpers.mjs` protects pure helper classification rules;
- `test-canon-draft-helper-registry.mjs` protects helper inventory fan-in,
  owner identity, render, and diagnostics using dependency injection;
- `test-generate-canon-draft-cli-helpers.mjs` protects the
  `generate-canon-draft.mjs --source helper-registry` CLI path;
- `test-check-canon-helpers.mjs` protects helper-registry drift detection.

The future mirror should keep those layers visible. Shared setup may build
fixtures, run Node commands, and clean up temporary directories, but it must not
decide helper labels, fan-in, contamination availability, CLI source behavior,
or drift categories.

## Reviewed Evidence

| Suite                                             | Preserved Node Command                                 | Proposed Focused Vitest Command                        | Surface Under Review                         |
| ------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------ | -------------------------------------------- |
| `tests/test-canon-draft-helpers.mjs`              | `node tests/test-canon-draft-helpers.mjs`              | `npm run test:vitest:canon-draft-helpers`              | helper classifier rules and precedence       |
| `tests/test-canon-draft-helper-registry.mjs`      | `node tests/test-canon-draft-helper-registry.mjs`      | `npm run test:vitest:canon-draft-helper-registry`      | helper aggregation, fan-in, render contracts |
| `tests/test-generate-canon-draft-cli-helpers.mjs` | `node tests/test-generate-canon-draft-cli-helpers.mjs` | `npm run test:vitest:generate-canon-draft-cli-helpers` | helper-registry CLI draft behavior           |
| `tests/test-check-canon-helpers.mjs`              | `node tests/test-check-canon-helpers.mjs`              | `npm run test:vitest:check-canon-helpers`              | helper-registry drift engine                 |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane B, canon/check-canon helper-registry family.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share the helper-registry canon source family. The PR must keep every Node
entrypoint runnable and must not absorb topology, naming, type-ownership,
integration, resolver, full audit, or performance suites.

## Protected Invariants

The future Vitest batch must preserve these contracts:

- helper group classification keeps the documented rule order:
  `ANY_COLLISION_HELPER`, `HELPER_DUPLICATE_STRONG`,
  `HELPER_LOCAL_COMMON`, then `HELPER_DUPLICATE_REVIEW`;
- `ANY_COLLISION_HELPER` remains universal over contaminated identities, not an
  existential one-bad-identity rule;
- heavily used low-info helper names still become
  `HELPER_DUPLICATE_STRONG`, not `HELPER_LOCAL_COMMON`;
- single helper classification preserves severe contamination, low-signal,
  central, shared, and zero-internal-fan-in precedence;
- helper inventory fan-in counts distinct consumer files, not call-site count;
- exported-never-called helpers stay present with fan-in `0`;
- same-file self-imports and type-only imports do not increase helper fan-in;
- re-export and alias hops keep terminal/source owner identity rather than
  consumer-side aliases or barrel owners;
- unknown/class-method-like helper definitions stay filtered from the helper
  registry inventory;
- unavailable contamination evidence renders as unknown/unavailable, not as a
  clean proof;
- `generate-canon-draft.mjs --source helper-registry` is accepted while unknown
  source values are rejected with the supported source list;
- `--source type-ownership` remains a regression guard while helper-registry
  support exists;
- missing `call-graph.json` still emits a helper draft with absence recorded in
  metadata rather than failing the helper-registry CLI path;
- helper drafts keep non-overwrite versioning, existing-canon observational
  headers, `--canon-output` overrides, path-with-spaces shell safety,
  scan-range scope text, mode text, and `FanInKind: consumer-file-count`;
- helper drift keeps missing-canon skip semantics, helper added/removed
  categories, and does not upgrade same-name-different-file to
  `helper-owner-changed`;
- contamination drift is evidence-gated per identity and downgrades to
  label-change plus an advisory diagnostic when enrichment is unavailable;
- fan-in tier drift is not gated by call-graph availability;
- extractor failures remain source-level parse-error diagnostics rather than
  disappearing as empty helper inventory.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- changing helper classifier rule order must fail through named cases;
- broadening contamination classification from universal to existential must
  fail;
- counting repeated calls inside one consumer as multiple fan-in edges must
  fail;
- counting type-only imports or same-file self-imports as runtime helper use
  must fail;
- attributing re-exported helpers to a barrel or import alias must fail;
- treating missing contamination enrichment as clean evidence must fail;
- rejecting `--source helper-registry` or regressing `--source type-ownership`
  must fail;
- overwriting an existing helper draft instead of writing a `.v2` draft must
  fail;
- losing path-with-spaces and `$` shell-safety coverage in the CLI helper
  fixture must fail;
- converting same-name-different-file drift into a nonexistent owner-change
  category must fail;
- emitting contamination-specific drift without the required evidence must fail;
- hiding extractor throws as empty helper registries must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary may use temporary directories, in-memory extractor
  stubs, resolver stubs, synthetic canon Markdown, and argument-safe process
  calls.
- Shared helper code may construct fixtures and run commands, but it must not
  decide helper labels, fan-in, contamination, CLI source routing, or drift
  categories.
- The mirror must not change `_lib/canon-draft-helpers.mjs`,
  `_lib/check-canon-helpers.mjs`, `generate-canon-draft.mjs`, or canon output
  contracts.
- The mirror must not absorb naming, topology, type-ownership, integration,
  resolver, generated/framework, deadness/ranking, performance, or full audit
  orchestration suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one implementation PR that adds:

1. `tests/canon-draft-helpers.test.mjs`,
2. `tests/canon-draft-helper-registry.test.mjs`,
3. `tests/generate-canon-draft-cli-helpers.test.mjs`,
4. `tests/check-canon-helpers.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving all four suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, and the wiki/doc
guards.
