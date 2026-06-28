# Vitest Pre-Write Advisory Lifecycle Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidates:**
>
> - `tests/test-pre-write-advisory-artifact.mjs`
> - `tests/test-pre-write-bootstrap.mjs`
> - `tests/test-pre-write-cli.mjs`
> - `tests/test-pre-write-drift.mjs`
> - `tests/test-pre-write-integration.mjs`

---

## Purpose

This review decides whether the remaining direct pre-write advisory lifecycle
suites can move together as one Lane C Vitest mirror batch. It does not add
Vitest suites.

The batch is acceptable because every suite protects the direct pre-write
advisory lifecycle rather than lookup policy, cue-tier promotion, resolver
expansion, deadness/ranking, generated surface inference, or performance cache
identity:

- `test-pre-write-advisory-artifact.mjs` protects invocation ids, deterministic
  intent hashing, dual advisory writes, atomic writes, and capability-missing
  failures;
- `test-pre-write-bootstrap.mjs` protects pre-write prerequisite module
  exports, symbol capability support flags, conforming any-contamination owner
  facts, identity fan-in, and the downstream `FP_BUDGET=0` gate;
- `test-pre-write-cli.mjs` protects direct `pre-write.mjs` CLI behavior,
  evidence availability, cold-cache producer selection, compact/rich intent
  normalization, dependency import confidence, shape/function evidence
  cold-cache, stdout/stderr separation, timeout handling, shell-safe paths, and
  suppressed-cue recording;
- `test-pre-write-drift.mjs` protects the pure read-only canonical drift
  projection without reparsing canonical files or rerunning lookup;
- `test-pre-write-integration.mjs` protects a direct end-to-end advisory run
  across name, file, dependency, shape, canonical drift, planned type escape,
  and advisory artifact round-trip surfaces.

This batch must stay separate from pre-write lookup policy suites, pre-write
renderer wording, cue-tier ranking, service-operation sibling policy,
post-write lifecycle, `audit-repo.mjs` wrapper behavior, resolver behavior,
deadness/ranking, generated/framework surfaces, and performance/incremental
cache identity.

## Reviewed Evidence

| Suite                                        | Preserved Node Command                            | Proposed Focused Vitest Command                   | Surface Under Review                         |
| -------------------------------------------- | ------------------------------------------------- | ------------------------------------------------- | -------------------------------------------- |
| `tests/test-pre-write-advisory-artifact.mjs` | `node tests/test-pre-write-advisory-artifact.mjs` | `npm run test:vitest:pre-write-advisory-artifact` | advisory id/hash/artifact writer             |
| `tests/test-pre-write-bootstrap.mjs`         | `node tests/test-pre-write-bootstrap.mjs`         | `npm run test:vitest:pre-write-bootstrap`         | prerequisite support/capability gates        |
| `tests/test-pre-write-cli.mjs`               | `node tests/test-pre-write-cli.mjs`               | `npm run test:vitest:pre-write-cli`               | direct pre-write CLI and cold-cache behavior |
| `tests/test-pre-write-drift.mjs`             | `node tests/test-pre-write-drift.mjs`             | `npm run test:vitest:pre-write-drift`             | pure canonical drift projection              |
| `tests/test-pre-write-integration.mjs`       | `node tests/test-pre-write-integration.mjs`       | `npm run test:vitest:pre-write-integration`       | direct end-to-end advisory lifecycle         |

Current Node evidence checked for this review:

```text
node tests/test-pre-write-advisory-artifact.mjs # 22 passed, 0 failed
node tests/test-pre-write-bootstrap.mjs         # 39 passed, 0 failed
node tests/test-pre-write-cli.mjs               # 84 passed, 0 failed
node tests/test-pre-write-drift.mjs             # 25 passed, 0 failed
node tests/test-pre-write-integration.mjs       # 21 passed, 0 failed
```

Goal lane: Lane C, pre-write lifecycle. This review covers only direct
pre-write advisory lifecycle components and the direct `pre-write.mjs` CLI.

## Result

These suites are acceptable as one bounded Vitest mirror batch.

The future implementation PR may add all five focused mirrors together because
they share the same temporary-repo, intent JSON, advisory JSON, direct CLI, and
pure helper assertion boundaries. The implementation must keep every Node
entrypoint runnable and must not turn missing evidence, malformed intent,
producer failure, timeout, canonical drift, or unavailable proof into a clean
success claim.

## Protected Invariants

The future Vitest batch must preserve these direct pre-write contracts:

- invocation ids match `YYYY-MM-DDTHH-mm-ssZ-<6-char-random>`;
- consecutive invocation ids differ because of the random suffix;
- `hashIntent()` is deterministic, key-order independent at top-level and
  nested object levels, and returns lowercase sha256 hex;
- different intents produce different intent hashes;
- `writeAdvisory()` writes both latest and invocation-specific JSON artifacts
  with identical bytes;
- repeated advisory writes update `latest` while preserving prior
  invocation-specific artifacts;
- advisory atomic writes leave no temp-file leftovers;
- `capabilities: null` and `capabilities-missing` failures round-trip through
  the artifact writer;
- required pre-write modules remain importable and export the named helpers the
  pre-write phases depend on;
- `symbols.json.meta.supports` remains present with `schemaVersion >= 3`;
- `supports.anyContamination === true` is emitted only with conforming
  per-identity owner fact surfaces;
- clean identities omit any-contamination annotations when support is true,
  while contaminated helpers, types, and JSDoc any owners carry conforming
  annotations;
- legacy flat any-contamination shapes remain rejected by the conforming
  predicate;
- `supports.identityFanIn` and `fanInByIdentity` stay available with
  `ownerFile::exportedName` keys;
- `tests/test-corpus.mjs` keeps `FP_BUDGET = 0`;
- direct pre-write happy-path CLI runs exit 0, render the advisory header, and
  write latest plus invocation-specific advisory JSON;
- CLI output prints the invocation-specific `--pre-write-advisory` handoff
  rather than pointing post-write at `latest`;
- rich intent object entries normalize into lookup arrays while preserving
  self-declaration `why` metadata;
- paths containing spaces, `$`, or both continue to work end-to-end;
- missing `symbols.json` with `--no-fresh-audit` exits 0 but renders
  unavailable evidence, not grounded absence;
- dependency lookup without symbols reports import graph unavailable and never
  renders grounded zero consumers;
- fresh dependency lookup records static package consumers with grounded count
  confidence and citations to `symbols.json.dependencyImportConsumers`;
- compact intents default missing top-level arrays with schema notes instead of
  failing;
- production scan range flags propagate into cold-cache `build-symbol-graph`;
- names-only cold-cache creates only the required symbol artifact, not unrelated
  triage or topology artifacts;
- exact shape, fields-only shape, and function-signature intents cold-cache only
  the evidence producers they need and keep unavailable evidence honest where
  exactness is missing;
- partial cold-cache runs only the missing producer and does not rebuild present
  artifacts;
- `--no-fresh-audit` spawns no cold-cache producers and records missing-artifact
  failures;
- producer failure and timeout do not hang and record failure evidence when the
  CLI completes;
- stdout contains the advisory Markdown while stderr contains diagnostic
  prefixes, preserving channel separation;
- create-only weak-token hints stay hidden from default Markdown but remain
  recorded in `suppressedCues`;
- `computeDrift()` is a pure read-only projection over already-computed
  canonical claims and lookups;
- aligned and not-consulted states produce no drift;
- owner-disagrees and ast-absent states produce exactly one drift entry per
  disagreement;
- canonical aligned with one owner plus extra AST identities remains empty
  drift, because the extra identity is canon-draft staleness, not drift;
- non-name lookups are ignored by drift projection;
- the drift module does not import the canonical parser, does not import name
  lookup, and does not read the filesystem;
- the integration run renders grounded facts, agent review cues, unavailable
  evidence, already-exists candidates, new code candidates, canonical drift,
  and planned type escapes in the expected advisory sections;
- `CANONICAL_EXISTS_AST_ABSENT` appears under already-exists while the literal
  `CANONICAL DRIFT:` appears only in the canonical drift section;
- lookup ordering in the advisory JSON remains names, files, dependencies,
  then shapes;
- integration advisory drift has exactly one `GoneType` drift entry;
- integration capabilities preserve `identityFanIn: true`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- advisory id generation that drops the random suffix or changes the shape must
  fail;
- hash normalization that becomes key-order sensitive must fail;
- artifact writes that leave temp files, lose invocation-specific files, or
  make latest/specific bytes diverge must fail;
- optimistic capability support without owner facts must fail;
- legacy any-contamination shapes must not be silently accepted;
- fan-in maps must not disappear or change key shape;
- missing symbols under `--no-fresh-audit` must not become grounded zero;
- dependency availability must not claim zero consumers when import graph
  evidence is unavailable;
- compact intent handling must not crash or silently drop warnings;
- cold-cache must not run unrelated producers or rebuild already-present
  artifacts;
- fields-only shapes must not become exact structural equality evidence;
- producer failures and timeouts must not hang or become clean success;
- diagnostics must not leak into stdout advisory Markdown;
- suppressed create-only cues must not render as default review cards;
- drift projection must not reparse canonical files, rerun lookup, or read the
  filesystem;
- extra AST identities beside an aligned canonical owner must not be mislabeled
  as owner-disagrees drift;
- integration output must not move `CANONICAL DRIFT:` into the already-exists
  section;
- integration lookup ordering must not drift because downstream readers depend
  on stable advisory shape.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is temporary repos, `.audit` output directories, intent
  JSON, direct `pre-write.mjs` invocations, direct pre-write helper imports,
  generated `symbols.json`, canonical Markdown fixtures, and advisory JSON.
- Shared setup may create temporary repos, write fixture files, write intent
  JSON, run `build-symbol-graph.mjs` or `pre-write.mjs`, read JSON/Markdown
  artifacts, and clean up directories.
- Shared helpers must not decide lookup labels, cue-tier promotion, suppressed
  cue policy, cold-cache producer selection, dependency confidence, shape
  exactness, canonical drift labels, renderer wording, resolver behavior,
  deadness/ranking, action-safety, or performance/cache identity.
- The mirror must not change `pre-write.mjs`, pre-write lookup helpers,
  cue-tier adapters, renderers, symbol graph producers, canonical parsers,
  drift projection, or public CLI contracts.
- The mirror must not absorb `tests/test-pre-write-cue-tiers.mjs`,
  `tests/test-pre-write-render.mjs`, `tests/test-pre-write-lookup-name.mjs`,
  pre-write lookup contract suites, pre-write input contract suites,
  `tests/test-pre-write-inventory-hook.mjs`, `tests/test-mode-dispatch.mjs`,
  post-write lifecycle suites, `audit-repo.mjs` wrapper suites, resolver
  suites, generated/framework suites, deadness/ranking suites, or
  performance/incremental suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/pre-write-advisory-artifact.test.mjs`,
2. `tests/pre-write-bootstrap.test.mjs`,
3. `tests/pre-write-cli.test.mjs`,
4. `tests/pre-write-drift.test.mjs`,
5. `tests/pre-write-integration.test.mjs`,
6. focused `npm run test:vitest:*` commands for each suite,
7. candidate-board updates moving all five suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, `npm test`, and
the wiki/doc guards.
