# Vitest Canon Type Ownership Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-canon-draft.mjs`
> - `tests/test-canon-draft-type-ownership.mjs`
> - `tests/test-generate-canon-draft-cli.mjs`
> - `tests/test-check-canon-types.mjs`

---

## Purpose

This review decides whether the type-ownership canon suites can move as one
Lane B Vitest mirror batch. It does not add Vitest suites. The goal is to
preserve the type classifier, type-ownership aggregation and rendering, the
`generate-canon-draft.mjs --source type-ownership` CLI path, and type drift
contracts without absorbing broad canon, topology, integration, resolver,
orchestrator, deadness, or performance behavior.

The batch is acceptable because all four suites protect the same canon source
family, but each suite owns a different layer:

- `test-canon-draft.mjs` protects pure type classifier rules and Markdown cell
  helpers;
- `test-canon-draft-type-ownership.mjs` protects type identity aggregation,
  fan-in space rendering, re-export ownership, shape evidence, generated-shape
  summarization, and degraded evidence wording;
- `test-generate-canon-draft-cli.mjs` protects the
  `generate-canon-draft.mjs --source type-ownership` CLI path;
- `test-check-canon-types.mjs` protects type-ownership drift detection and
  renderer contracts.

The future mirror should keep those layers visible. Shared setup may construct
temporary directories, write synthetic `symbols.json`, write synthetic
`shape-index.json`, run Node commands, and clean up directories, but it must
not decide type labels, contamination scope, owner identity, shape-pairing
confidence, CLI source behavior, or drift categories.

## Reviewed Evidence

| Suite                                       | Preserved Node Command                           | Proposed Focused Vitest Command                  | Surface Under Review                             |
| ------------------------------------------- | ------------------------------------------------ | ------------------------------------------------ | ------------------------------------------------ |
| `tests/test-canon-draft.mjs`                | `node tests/test-canon-draft.mjs`                | `npm run test:vitest:canon-draft`                | type classifier rules and Markdown cell helpers  |
| `tests/test-canon-draft-type-ownership.mjs` | `node tests/test-canon-draft-type-ownership.mjs` | `npm run test:vitest:canon-draft-type-ownership` | type identity aggregation and renderer contracts |
| `tests/test-generate-canon-draft-cli.mjs`   | `node tests/test-generate-canon-draft-cli.mjs`   | `npm run test:vitest:generate-canon-draft-cli`   | type-ownership CLI draft behavior                |
| `tests/test-check-canon-types.mjs`          | `node tests/test-check-canon-types.mjs`          | `npm run test:vitest:check-canon-types`          | type-ownership drift engine                      |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane B, canon/check-canon type-ownership family.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share the type-ownership canon source family. The PR must keep every Node
entrypoint runnable and must not absorb helper-registry, naming, topology,
integration, resolver, generated/framework, full audit, performance, or
deadness/ranking suites.

## Protected Invariants

The future Vitest batch must preserve these contracts:

- `LOW_INFO_NAMES` stays frozen and retains the current low-info type name set;
- type group classification keeps the documented rule order:
  `ANY_COLLISION`, `DUPLICATE_STRONG`, `LOCAL_COMMON_NAME`, then
  `DUPLICATE_REVIEW`;
- `ANY_COLLISION` remains universal over contaminated identities and does not
  trigger for `has-any`, `unknown-surface`, or mixed contaminated/non-contaminated
  groups;
- high fan-in still wins over low-info duplicate names such as `Result`;
- single-identity classification preserves severe contamination, low-signal
  type aliases, fan-in tiers, and zero-internal-fan-in precedence;
- Markdown helper behavior keeps pipe/backslash escaping, newline collapse, and
  CommonMark backtick wrapping;
- type identities use `ownerFile::exportedName`, not aliases, barrels, or
  internal display names;
- duplicate type groups are keyed by name while each identity remains distinct;
- fan-in totals and value/type/broad fan-in space are rendered together without
  changing the total fan-in semantics;
- re-export chains retain the terminal owner identity and record barrels in
  `reExportedThrough`;
- only exported TypeScript type declarations enter the type-ownership draft;
- severely contaminated type owners keep the severe label even with high fan-in;
- empty inputs still render a valid type-ownership draft;
- shape evidence enriches duplicate groups without replacing fan-in-based
  labels;
- incomplete shape indexes emit degraded and partial evidence instead of
  treating missing shape facts as clean proof;
- generated-only duplicate shape groups are summarized without expanding noisy
  generated details;
- malformed generated shape evidence fails closed and does not hide shape notes;
- `generate-canon-draft.mjs --source type-ownership` remains accepted while
  unknown sources are rejected with the supported source list;
- type-ownership drafts keep non-overwrite versioning, existing-canon
  observational headers, `--canon-output` overrides, path-with-spaces shell
  safety, scan-range scope text, missing-root errors, missing-symbols graceful
  fallback, and optional shape-index evidence;
- type drift keeps missing-canon skip semantics, identity added/removed,
  label-changed, owner-changed, and shape-assisted ambiguous rename behavior;
- 1:1 owner-change records keep the top-level identity in
  `ownerFile::exportedName` form and place old/new identities under
  `canon.identity` and `fresh.identity`;
- ambiguous same-name moves stay low-confidence add/remove records unless
  unique shape evidence grounds a subset;
- malformed shape indexes fail closed and never force an owner-changed upgrade;
- promoted drafts with a `Fan-in space` column are accepted by the drift parser;
- every type drift record keeps `kind: "type-drift"`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- broadening `ANY_COLLISION` from universal to existential must fail;
- allowing `has-any` or `unknown-surface` to trigger `ANY_COLLISION` must fail;
- changing classifier rule order so low-info names outrank high fan-in must
  fail;
- treating severe contamination as clean high-fan-in ownership must fail;
- collapsing duplicate identities by exported name alone must fail;
- deriving identity from a barrel, alias, or hypothetical internal `typeName`
  field must fail;
- counting value/type/broad fan-in space as a replacement for total fan-in must
  fail;
- treating missing or incomplete shape facts as clean evidence must fail;
- hiding malformed generated-file metadata behind generated-shape summarization
  must fail;
- rejecting `--source type-ownership` or omitting supported sources from the
  unknown-source error must fail;
- overwriting an existing type-ownership draft instead of writing a `.v2` draft
  must fail;
- losing path-with-spaces and `$` shell-safety coverage in the CLI fixture must
  fail;
- converting ambiguous 2:1 owner moves into high-confidence owner-changed drift
  without unique shape evidence must fail;
- emitting compound or arrow-style top-level identities for owner-changed drift
  must fail;
- silently ignoring malformed shape-index data while promoting owner-change
  confidence must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary may use temporary directories, synthetic `symbols.json`,
  synthetic `shape-index.json`, synthetic canon Markdown, and argument-safe
  process calls.
- Shared helper code may construct fixtures, write JSON/Markdown, run commands,
  and clean up directories, but it must not decide type labels, fan-in tiers,
  contamination, shape-evidence status, CLI source routing, or drift
  categories.
- The mirror must not change `_lib/canon-draft-types.mjs`,
  `_lib/check-canon-types.mjs`, `generate-canon-draft.mjs`, or canon output
  contracts.
- The mirror must not absorb helper-registry, naming, topology, integration,
  resolver, generated/framework, deadness/ranking, performance, or full audit
  orchestration suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Verification Snapshot

The review was grounded by running the preserved Node commands before adding
this page:

```text
node tests/test-canon-draft.mjs                 # 39 passed, 0 failed
node tests/test-canon-draft-type-ownership.mjs  # 37 passed, 0 failed
node tests/test-generate-canon-draft-cli.mjs    # 20 passed, 0 failed
node tests/test-check-canon-types.mjs           # 36 passed, 0 failed
```

## Recommendation

Proceed to one implementation PR that adds:

1. `tests/canon-draft.test.mjs`,
2. `tests/canon-draft-type-ownership.test.mjs`,
3. `tests/generate-canon-draft-cli.test.mjs`,
4. `tests/check-canon-types.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving all four suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, and the wiki/doc
guards.
