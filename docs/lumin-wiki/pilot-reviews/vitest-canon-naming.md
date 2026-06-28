# Vitest Canon Naming Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-canon-draft-naming.mjs`
> - `tests/test-canon-draft-naming-structure.mjs`
> - `tests/test-generate-canon-draft-cli-naming.mjs`
> - `tests/test-check-canon-naming.mjs`

---

## Purpose

This review decides whether the naming canon suites can move as one Lane B
Vitest mirror batch. It does not add Vitest suites. The goal is to preserve the
naming classifier, cohort collection, CLI draft, and drift contracts without
turning them into broad canon, parser, topology, or audit behavior tests.

The batch is acceptable because all four suites protect the same canon source
family, but each suite owns a different layer:

- `test-canon-draft-naming.mjs` protects pure naming classifier and basename
  normalization rules;
- `test-canon-draft-naming-structure.mjs` protects naming cohort aggregation,
  item rows, renderer shape, and diagnostics using dependency injection;
- `test-generate-canon-draft-cli-naming.mjs` protects the
  `generate-canon-draft.mjs --source naming` CLI path;
- `test-check-canon-naming.mjs` protects naming drift detection and parse-error
  downgrade behavior.

The future mirror should keep those layers visible. Shared setup may build
fixtures, run Node commands, and clean up temporary directories, but it must not
decide naming labels, cohort dominance, outlier identity, CLI source behavior,
or drift categories.

## Reviewed Evidence

| Suite                                            | Preserved Node Command                                | Proposed Focused Vitest Command                       | Surface Under Review                             |
| ------------------------------------------------ | ----------------------------------------------------- | ----------------------------------------------------- | ------------------------------------------------ |
| `tests/test-canon-draft-naming.mjs`              | `node tests/test-canon-draft-naming.mjs`              | `npm run test:vitest:canon-draft-naming`              | naming classifier and basename normalization     |
| `tests/test-canon-draft-naming-structure.mjs`    | `node tests/test-canon-draft-naming-structure.mjs`    | `npm run test:vitest:canon-draft-naming-structure`    | naming cohort aggregation and renderer contracts |
| `tests/test-generate-canon-draft-cli-naming.mjs` | `node tests/test-generate-canon-draft-cli-naming.mjs` | `npm run test:vitest:generate-canon-draft-cli-naming` | naming CLI draft behavior                        |
| `tests/test-check-canon-naming.mjs`              | `node tests/test-check-canon-naming.mjs`              | `npm run test:vitest:check-canon-naming`              | naming drift engine                              |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane B, canon/check-canon naming family.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share the naming canon source family. The PR must keep every Node
entrypoint runnable and must not absorb helper-registry, topology,
type-ownership, integration, resolver, full audit, performance, or
deadness/ranking suites.

## Protected Invariants

The future Vitest batch must preserve these contracts:

- `detectConvention` keeps the documented camelCase, PascalCase, kebab-case,
  snake_case, UPPER_SNAKE, and mixed classification boundaries;
- single-segment lowercase, uppercase, empty, and non-string names keep their
  current fallback conventions;
- `normalizeFileBasename` strips the longest recognized extensions first,
  handles `.test`, `.spec`, `.stories`, `.d.ts`, bare files, and Windows-style
  path separators;
- naming cohorts require sufficient effective evidence before producing a
  dominant convention label;
- low-info names reduce effective cohort size and always classify as
  `low-info-excluded` before any match or outlier rule;
- the `mixed` fallback convention never becomes a `mixed-dominant` label;
- file cohorts use all files in the cohort even when only some files contain
  exports;
- symbol cohorts keep the `submodule::kind` identity shape and separate
  `type-export`, `helper-export`, and `constant-export`;
- outlier identities keep file path form for file outliers and
  `<ownerFile>::<exportedName>` form for symbol outliers;
- extractor throws surface as parse-error diagnostics and do not silently shrink
  the fresh naming inventory into false drift;
- the naming renderer keeps the CohortIdentityShape meta line for `submodule`
  and `submodule::kind`, plus the file cohort section, symbol cohort section,
  and conditional outlier section;
- empty repositories still render a valid naming draft;
- existing canonical `naming.md` files produce the observational
  existing-canon header;
- `generate-canon-draft.mjs --source naming` is accepted while unknown source
  values are rejected with the supported source list;
- other supported sources remain regression guards while naming support exists;
- naming drafts keep non-overwrite versioning, `--canon-output` overrides,
  path-with-spaces shell safety, stderr summary counts, missing-root errors,
  scan-range scope text, and production scope text;
- naming drift keeps missing-canon skip semantics, cohort added/removed,
  cohort convention shifted, new-outlier introduced, and outlier-resolved
  categories;
- every naming drift record keeps `kind: "naming-drift"`;
- P3 display dash (`—`) and P5 fresh `null` dominant convention remain
  equivalent for promoted mixed-convention drafts.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- classifying `mixed` as a dominant convention must fail;
- stripping `.d.ts`, `.stories`, `.test`, or `.spec` in the wrong order must
  fail;
- allowing low-info names to override the Rule 0 exclusion must fail;
- using raw member count instead of effective count must fail;
- dropping files from file cohorts because symbol extraction has no defs must
  fail;
- hiding extractor throws as empty naming inventories must fail;
- losing `submodule::kind` symbol cohort identity must fail;
- confusing file outlier identities with symbol outlier identities must fail;
- rendering a zero-outlier draft with a stale outlier section must fail;
- rejecting `--source naming` or omitting `naming` from the supported-source
  error message must fail;
- overwriting an existing naming draft instead of writing a `.v2` draft must
  fail;
- losing path-with-spaces and `$` shell-safety coverage in the CLI naming
  fixture must fail;
- reporting a promoted mixed-convention draft as changed only because `—` and
  `null` differ must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary may use temporary directories, in-memory extractor
  stubs, submodule stubs, synthetic canon Markdown, and argument-safe process
  calls.
- Shared helper code may construct fixtures and run commands, but it must not
  decide naming labels, dominance thresholds, low-info handling, outlier
  identity, CLI source routing, or drift categories.
- The mirror must not change `_lib/canon-draft-naming.mjs`,
  `_lib/check-canon-naming.mjs`, `generate-canon-draft.mjs`, or canon output
  contracts.
- The mirror must not absorb helper-registry, topology, type-ownership,
  integration, resolver, generated/framework, deadness/ranking, performance, or
  full audit orchestration suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one implementation PR that adds:

1. `tests/canon-draft-naming.test.mjs`,
2. `tests/canon-draft-naming-structure.test.mjs`,
3. `tests/generate-canon-draft-cli-naming.test.mjs`,
4. `tests/check-canon-naming.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving all four suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, and the wiki/doc
guards.
