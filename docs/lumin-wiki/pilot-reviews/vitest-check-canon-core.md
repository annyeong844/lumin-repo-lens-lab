# Vitest Check Canon Core Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-check-canon-utils.mjs`
> - `tests/test-check-canon-artifact.mjs`
> - `tests/test-generate-check-canon-cli.mjs`
> - `tests/test-check-canon-integration.mjs`

---

## Purpose

This review decides whether the check-canon core suites can move as one Lane B
Vitest mirror batch. It does not add Vitest suites. The goal is to preserve the
parser, artifact writer, CLI, and end-to-end drift evidence contracts that
protect `check-canon` without absorbing audit-repo orchestration, resolver
behavior, generated/framework surfaces, deadness/ranking, performance, or
incremental cache behavior.

The batch is acceptable because all four suites protect the same check-canon
contract surface:

- `test-check-canon-utils.mjs` protects pure parser strictness, drift record
  shaping, category-family mapping, and source-specific label sets;
- `test-check-canon-artifact.mjs` protects canonical file loading and
  `canon-drift.json` / Markdown writer behavior;
- `test-generate-check-canon-cli.mjs` protects `check-canon` CLI source
  dispatch, exit-code policy, optional enrichment diagnostics, and
  all-source aggregation;
- `test-check-canon-integration.mjs` protects fixture-to-artifact drift
  detection for type-ownership, helper-registry, and topology sources through
  the real CLI path.

The future mirror should keep those contracts explicit. Shared setup may create
temporary roots, copy fixtures, write JSON/Markdown files, run the CLI, and
parse generated artifacts, but it must not decide parser semantics, exit-code
policy, drift family mapping, source aggregation, stale draft behavior,
resolver meaning, or audit orchestration behavior.

## Reviewed Evidence

| Suite                                     | Preserved Node Command                         | Proposed Focused Vitest Command                | Surface Under Review                         |
| ----------------------------------------- | ---------------------------------------------- | ---------------------------------------------- | -------------------------------------------- |
| `tests/test-check-canon-utils.mjs`        | `node tests/test-check-canon-utils.mjs`        | `npm run test:vitest:check-canon-utils`        | parser strictness and drift JSON shape       |
| `tests/test-check-canon-artifact.mjs`     | `node tests/test-check-canon-artifact.mjs`     | `npm run test:vitest:check-canon-artifact`     | canon loader and drift artifact writer I/O   |
| `tests/test-generate-check-canon-cli.mjs` | `node tests/test-generate-check-canon-cli.mjs` | `npm run test:vitest:generate-check-canon-cli` | check-canon CLI exit and output matrix       |
| `tests/test-check-canon-integration.mjs`  | `node tests/test-check-canon-integration.mjs`  | `npm run test:vitest:check-canon-integration`  | end-to-end check-canon drift fixture outputs |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane B, canon/check-canon core family.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share the check-canon parser/artifact/CLI contract surface. The PR must
keep every Node entrypoint runnable and must not absorb audit-repo
orchestration, resolver expansion, generated/framework surfaces,
deadness/ranking, performance, incremental cache, or broader analyzer behavior
suites.

## Protected Invariants

The future Vitest batch must preserve these contracts:

- type parser strictness keeps `skipped-unrecognized-schema`, top-level
  `parse-error`, per-row `canon-parse-error`, and clean parse outcomes
  distinct;
- optional `Fan-in space` input remains accepted but ignored for type drift
  semantics;
- prefix memo tables do not poison real type canon rows;
- memo-only type tables remain unrecognized rather than silently parsed as
  clean canon;
- `makeDriftRecord(...)` derives family values from category without changing
  canonical identity shape;
- `CATEGORY_TO_FAMILY` and per-source label sets remain pinned;
- helper parser required columns, known statuses, `anyUnknownSignal`, and pipe
  handling inside Signature cells remain strict;
- topology parser keeps inventory, cycles, cross-edge, and oversize sections
  distinct;
- topology §1/§3 SCC disagreement remains a parse error;
- topology cross-edge and oversize tables require their documented headers;
- naming parser keeps file cohorts, symbol cohorts, outliers, placeholder
  normalization, and low-info-excluded filtering distinct;
- missing canon files are reported with source-specific skipped status and
  diagnostics;
- real canon file loads report clean status and line counts;
- `writeCanonDriftArtifacts(...)` overwrites stale prior JSON instead of
  append-merging foreign source entries;
- absent report Markdown skips the `.md` artifact without affecting JSON;
- CLI exit codes remain `0` for clean, `1` for drift, and `2` for invalid or
  missing required input;
- helper-registry enrichment remains non-strict where documented, while
  type-ownership strict input failures still exit `2`;
- stale `canonical-draft/*` files do not affect check-canon results;
- canonical files remain byte-identical before and after check-canon runs;
- `--source all` writes every per-source key and preserves the checked-source
  rule table.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- parser schemas that are unrecognized, malformed, or row-corrupt must not
  collapse into one generic failure;
- helper Signature cells containing escaped pipes or code-span pipes must not
  split into false columns;
- topology required-section or malformed-header failures must not silently
  produce clean output;
- naming §2 absence and present-but-empty states must remain distinct;
- placeholder `—` must normalize to null where the current parser expects it;
- low-info-excluded outlier rows must validate without entering the outlier
  drift set;
- missing source artifacts must name the missing file in diagnostics and use
  the correct exit policy;
- corrupt JSON must report `[check-canon]` diagnostics without exposing a raw
  Node stack trace;
- stale `canonical-draft` fixture data must not be read as canonical truth;
- canonical Markdown files must never be rewritten by check-canon;
- all-source aggregation must keep all four per-source keys even when sources
  are skipped.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary may create temporary repos, copy fixture trees, write
  JSON/Markdown artifacts, invoke `generate-check-canon.mjs`, and inspect the
  resulting artifacts.
- Shared helper code may reduce repeated temp-root setup, CLI invocation, and
  JSON/Markdown reads, but it must not decide parser semantics, drift category
  meaning, exit-code policy, source aggregation, stale draft behavior, resolver
  behavior, or audit-repo orchestration behavior.
- The mirror must not absorb `tests/test-audit-repo-check-canon.mjs`,
  `tests/test-audit-repo-canon-draft.mjs`, audit-repo lifecycle tests,
  resolver suites, generated/framework suites, deadness/ranking suites,
  performance/incremental suites, or pre/post-write suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Verification Snapshot

The review was grounded by running the preserved Node commands before adding
this page:

```text
node tests/test-check-canon-utils.mjs          # 66 passed, 0 failed
node tests/test-check-canon-artifact.mjs       # 21 passed, 0 failed
node tests/test-generate-check-canon-cli.mjs   # 52 passed, 0 failed
node tests/test-check-canon-integration.mjs    # 66 passed, 0 failed
```

## Recommendation

Proceed to one implementation PR that adds:

1. `tests/check-canon-utils.test.mjs`,
2. `tests/check-canon-artifact.test.mjs`,
3. `tests/generate-check-canon-cli.test.mjs`,
4. `tests/check-canon-integration.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving all four suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, and the wiki/doc
guards.
