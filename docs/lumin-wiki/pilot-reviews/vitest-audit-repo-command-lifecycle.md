# Vitest Audit Repo Command Lifecycle Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-audit-repo-canon-draft.mjs`
> - `tests/test-audit-repo-check-canon.mjs`
> - `tests/test-audit-repo-pre-write.mjs`
> - `tests/test-audit-repo-post-write.mjs`

---

## Purpose

This review decides whether the `audit-repo.mjs` command lifecycle wrapper
suites can move together as one Vitest mirror batch. It does not add Vitest
suites.

The batch is acceptable because these suites protect orchestration surfaces,
not resolver expansion, deadness/ranking, generated surface inference, or
performance cache identity:

- `test-audit-repo-canon-draft.mjs` protects `--canon-draft` routing,
  `manifest.canonDraft`, source scoping, profile defaults, helper delegation,
  and shell-safe command invocation;
- `test-audit-repo-check-canon.mjs` protects `--check-canon` routing,
  `manifest.checkCanon`, advisory versus strict exit behavior, child exit
  normalization, and independence from `--canon-draft`;
- `test-audit-repo-pre-write.mjs` protects `--pre-write` routing, required
  intent handling, stdin intent dispatch, pre-write-only audit skipping,
  evidence availability mirroring, production scan range forwarding, and
  command-result summaries;
- `test-audit-repo-post-write.mjs` protects `--post-write` routing, advisory
  requirements, pre/post mutex behavior, manifest summary fields, scan-range
  flag forwarding, relocated delta output, stdout/stderr ordering, and strict
  confidence exit behavior.

The future mirror must keep these as `audit-repo.mjs` wrapper tests. It must
not absorb the direct canon/check-canon CLIs, direct pre-write/post-write
component tests, cue-tier policy, renderer wording, resolver behavior,
deadness/ranking, or performance/incremental cache suites.

## Reviewed Evidence

| Suite                                   | Preserved Node Command                       | Proposed Focused Vitest Command              | Surface Under Review                 |
| --------------------------------------- | -------------------------------------------- | -------------------------------------------- | ------------------------------------ |
| `tests/test-audit-repo-canon-draft.mjs` | `node tests/test-audit-repo-canon-draft.mjs` | `npm run test:vitest:audit-repo-canon-draft` | audit-repo `--canon-draft` lifecycle |
| `tests/test-audit-repo-check-canon.mjs` | `node tests/test-audit-repo-check-canon.mjs` | `npm run test:vitest:audit-repo-check-canon` | audit-repo `--check-canon` lifecycle |
| `tests/test-audit-repo-pre-write.mjs`   | `node tests/test-audit-repo-pre-write.mjs`   | `npm run test:vitest:audit-repo-pre-write`   | audit-repo `--pre-write` lifecycle   |
| `tests/test-audit-repo-post-write.mjs`  | `node tests/test-audit-repo-post-write.mjs`  | `npm run test:vitest:audit-repo-post-write`  | audit-repo `--post-write` lifecycle  |

Current Node evidence checked for this review:

```text
node tests/test-audit-repo-canon-draft.mjs # 47 passed, 0 failed
node tests/test-audit-repo-check-canon.mjs # 42 passed, 0 failed
node tests/test-audit-repo-pre-write.mjs   # 35 passed, 0 failed
node tests/test-audit-repo-post-write.mjs  # 57 passed, 0 failed
```

Goal lane: Lane H plus the audit-repo wrapper portion of Lane C. This review
covers only the command lifecycle wrappers that route through `audit-repo.mjs`.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share the same temporary-repo, spawn, manifest-read, summary-read, and
exit-code assertion shape. The mirror must keep every Node entrypoint runnable
and must not turn child command failures, missing evidence, strict-mode exits,
or mutually exclusive flags into clean success claims.

## Protected Invariants

The future Vitest batch must preserve these `audit-repo.mjs` wrapper
contracts:

- `--canon-draft` records `manifest.canonDraft.requested`, `ran`,
  `requestedSources`, `perSource`, `draftPath`, and `draftPaths` accurately;
- `--canon-draft` writes command-result summaries before a reader needs to
  spelunk JSON artifacts;
- the orchestrator guarantees `topology.json` before canon topology draft
  generation in normal `audit-repo.mjs --canon-draft` flow;
- `--sources`, `--source`, `all`, duplicate source requests, unknown sources,
  versioned draft output, and custom `--canon-output` keep their existing
  exit-code and manifest behavior;
- `audit-repo.mjs` stays a thin wrapper and delegates canon/check-canon source
  validation to the helper modules;
- `--check-canon` records `manifest.checkCanon.requested`, `ran`, `strict`,
  `requestedSources`, `perSource`, `summary`, and `driftCounts` accurately;
- `check-canon` child exit 1/2 outcomes remain structured per-source results,
  not generic spawn failures;
- advisory `--check-canon` runs with missing canon files exit 0, while strict
  no-checked-source runs exit 2;
- `--canon-draft + --check-canon` keeps independent manifest blocks and does
  not confuse generated `canonical-draft/` proposals with promoted
  `canonical/` files;
- `--pre-write` without `--intent` exits 2, records `manifest.preWrite.ran:
false`, and does not run the base quick audit chain;
- `--pre-write --intent <file>` and `--intent -` write invocation-specific and
  latest advisory pointers while preserving evidence availability metadata;
- pre-write-only runs create the grounded artifacts they need without
  pretending `triage.json`, `topology.json`, or `fix-plan.json` exist;
- default audit runs do not create pre-write advisory artifacts;
- malformed stdin intent propagates a non-zero child exit and writes a
  manifest reason instead of a partial advisory;
- production scan range flags are forwarded into both orchestrator manifest
  and pre-write advisory metadata;
- `--post-write` requires `--pre-write-advisory` and records missing advisory
  failures without partial summary fields;
- pre-write and post-write remain mutually exclusive in audit-repo mode;
- bogus advisory paths, strict post-write, strict confidence, and
  baseline-missing states keep their current exit-code matrix;
- post-write manifest summary fields mirror the delta JSON summary fields;
- `--include-tests`, `--no-include-tests`, `--production`, and `--delta-out`
  forwarding stays verbatim;
- post-write stdout and stderr remain segregated, with Markdown output ordered
  before the final wrote-line;
- shell paths containing spaces or `$` survive every wrapper path in the batch.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- treating unknown canon/check-canon sources as partial successful runs must
  fail;
- emitting unrequested canon/check-canon `perSource` keys must fail;
- running default profiles as if they included `--canon-draft` must fail;
- reintroducing source validation into `audit-repo.mjs` instead of helper
  modules must fail;
- collapsing child exit 1/2 into generic spawn failure must fail;
- making strict check-canon no-op when no source was checked must fail;
- confusing `canonical-draft/` proposals with promoted `canonical/` canon
  files must fail;
- creating pre-write advisory artifacts on failed or default runs must fail;
- running base quick-audit producers during names-only pre-write must fail;
- omitting evidence availability from the pre-write manifest mirror must fail;
- ignoring malformed stdin intent or writing a partial advisory must fail;
- allowing `--pre-write` and `--post-write` together must fail;
- treating missing or bogus post-write advisory evidence as a clean delta must
  fail;
- hiding scan-range or baseline-missing confidence limits under strict
  post-write modes must fail;
- moving post-write Markdown diagnostics to stderr or changing section order
  must fail;
- breaking path-with-spaces or `$` handling must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is temporary repos, `.audit` output directories,
  manifest JSON, command-result Markdown summaries, advisory JSON, and
  post-write delta JSON.
- Shared setup may create temporary repos, write files, run `audit-repo.mjs`,
  read JSON/Markdown artifacts, and clean up directories.
- Shared helpers must not decide canon source validity, drift labels, advisory
  evidence availability, post-write delta labels, baseline status, scan-range
  confidence, or command exit-code policy.
- The mirror must not change `audit-repo.mjs`, canon/check-canon helpers,
  pre-write lookup behavior, post-write delta behavior, renderer wording, or
  public CLI contracts.
- The mirror must not absorb direct canon/check-canon component suites,
  direct pre-write/post-write component suites, `test-mode-dispatch.mjs`,
  `test-audit-repo.mjs`, symbol incremental routing, cue tiers, resolver
  expansion, generated/framework surfaces, deadness/ranking, or performance
  cache identity.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/audit-repo-canon-draft.test.mjs`,
2. `tests/audit-repo-check-canon.test.mjs`,
3. `tests/audit-repo-pre-write.test.mjs`,
4. `tests/audit-repo-post-write.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving all four suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, `npm test`, and
the wiki/doc guards.
