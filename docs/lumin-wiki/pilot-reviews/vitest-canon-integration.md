# Vitest Canon Integration Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-canon-draft-integration.mjs`
> - `tests/test-canon-draft-integration-helpers.mjs`
> - `tests/test-canon-draft-integration-topology.mjs`

---

## Purpose

This review decides whether the canon draft integration suites can move as one
Lane B Vitest mirror batch. It does not add Vitest suites. The goal is to
preserve the end-to-end fixture-to-Markdown checks that run real producer or
canon CLI paths, while keeping full audit orchestration, check-canon drift
integration, resolver expansion, performance, incremental cache, deadness, and
ranking behavior out of scope.

The batch is acceptable because all three suites use fixture-controlled
Markdown parsing to validate `generate-canon-draft.mjs` integration behavior,
but each suite owns a different integration surface:

- `test-canon-draft-integration.mjs` protects the type-ownership
  `build-symbol-graph.mjs` to `generate-canon-draft.mjs` path;
- `test-canon-draft-integration-helpers.mjs` protects the helper-registry
  `generate-canon-draft.mjs --source helper-registry` path through the real
  extractor and resolver;
- `test-canon-draft-integration-topology.mjs` protects the topology
  `triage-repo.mjs` plus `measure-topology.mjs` to
  `generate-canon-draft.mjs --source topology` path.

The future mirror should keep those integration layers visible. Shared setup
may construct temporary repos, write fixture files, run the existing Node CLIs,
parse fixture-controlled Markdown tables, and clean up directories, but it must
not decide classifier labels, fan-in semantics, topology SCC membership,
workspace-boundary behavior, CLI exit codes, or Markdown drift categories.

## Reviewed Evidence

| Suite                                             | Preserved Node Command                                 | Proposed Focused Vitest Command                        | Surface Under Review                         |
| ------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------ | -------------------------------------------- |
| `tests/test-canon-draft-integration.mjs`          | `node tests/test-canon-draft-integration.mjs`          | `npm run test:vitest:canon-draft-integration`          | type-ownership end-to-end draft integration  |
| `tests/test-canon-draft-integration-helpers.mjs`  | `node tests/test-canon-draft-integration-helpers.mjs`  | `npm run test:vitest:canon-draft-integration-helpers`  | helper-registry end-to-end draft integration |
| `tests/test-canon-draft-integration-topology.mjs` | `node tests/test-canon-draft-integration-topology.mjs` | `npm run test:vitest:canon-draft-integration-topology` | topology producer-to-draft integration       |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane B, canon/check-canon integration family.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all three focused mirrors together because
they share the canon draft integration surface and fixture-controlled Markdown
parsing boundary. The PR must keep every Node entrypoint runnable and must not
absorb check-canon integration, audit-repo orchestration, resolver,
generated/framework, deadness/ranking, performance, or incremental cache
suites.

## Protected Invariants

The future Vitest batch must preserve these contracts:

- fixture-controlled Markdown parsers stay narrow and are not promoted into
  general Markdown parsers;
- duplicate type rows remain one row per owner and carry the group
  classification label;
- cross-file distinct type names keep distinct identities and single-identity
  labels;
- re-export chains retain the terminal owner identity rather than the barrel
  identity;
- type-ownership Markdown round-trips back to the expected row count and names;
- helper-registry fan-in counts distinct consumer files rather than call-site
  count;
- exported-never-called helpers remain visible with
  `zero-internal-fan-in-helper`;
- callback-only helper consumption remains captured through the import-resolve
  lens;
- duplicate helpers across owners carry the shared duplicate group label;
- const-var arrow helpers surface in the helper registry;
- empty helper inputs render a valid draft with zero rows;
- helper cross-check diagnostics surface in Notes and point to the owner
  identity;
- helper row statuses remain within the canonical helper label set;
- topology inventory row count matches distinct submodule count;
- topology inventory file totals match both `topology.summary.files` and
  `Object.keys(topology.nodes).length`;
- topology SCC output renders the forbidden-cycle label, the SCC members, and
  the cyclic-submodule row status;
- topology oversize output distinguishes 1000+ LOC `extreme-oversize` from
  400+ LOC `oversize`;
- acyclic topology output renders the explicit acyclic banner;
- missing `topology.json` remains exit code 2 with a `measure-topology.mjs`
  recovery hint;
- missing `triage.json` remains graceful and omits workspace boundaries;
- topology row statuses remain within the canonical topology label set;
- path-with-spaces and `$` fixtures survive the full producer-to-draft path.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- replacing fixture-controlled Markdown parsing with broad parser assumptions
  must fail;
- collapsing duplicate type owners into one row must fail;
- reporting a barrel as the terminal type owner must fail;
- counting helper call-sites instead of distinct consumer files must fail;
- dropping callback-only helper consumption must fail;
- hiding exported-never-called helpers from the helper registry must fail;
- dropping helper cross-check diagnostics or retargeting them to the wrong
  identity must fail;
- omitting isolated topology submodules from the inventory must fail;
- losing SCC member rendering or the cyclic-submodule row status must fail;
- changing oversize thresholds or collapsing oversize and extreme-oversize must
  fail;
- omitting the acyclic banner must fail;
- treating missing `topology.json` as a generic argument failure must fail;
- rendering workspace boundaries when `triage.json` is absent must fail;
- losing path-with-spaces and `$` shell-safety coverage in the full integration
  fixture must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary may use temporary directories, synthetic repos, real
  producer CLI calls, fixture-controlled Markdown table parsers, and cleanup
  wrappers.
- Shared helper code may construct fixtures, write files, run commands, parse
  known table rows, and clean up directories, but it must not decide canon
  labels, fan-in counts, owner identities, SCC membership, workspace-boundary
  semantics, or CLI exit codes.
- The mirror must not change `generate-canon-draft.mjs`,
  `build-symbol-graph.mjs`, `measure-topology.mjs`, `triage-repo.mjs`, or canon
  output contracts.
- The mirror must not absorb check-canon integration, audit-repo orchestration,
  resolver, generated/framework, deadness/ranking, performance, incremental
  cache, or full audit suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Verification Snapshot

The review was grounded by running the preserved Node commands before adding
this page:

```text
node tests/test-canon-draft-integration.mjs          # 10 passed, 0 failed
node tests/test-canon-draft-integration-helpers.mjs  # 23 passed, 0 failed
node tests/test-canon-draft-integration-topology.mjs # 19 passed, 0 failed
```

## Recommendation

Proceed to one implementation PR that adds:

1. `tests/canon-draft-integration.test.mjs`,
2. `tests/canon-draft-integration-helpers.test.mjs`,
3. `tests/canon-draft-integration-topology.test.mjs`,
4. focused `npm run test:vitest:*` commands for each suite,
5. candidate-board updates moving all three suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, and the wiki/doc
guards.
