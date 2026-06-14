# Vitest Canon Topology Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-canon-draft-topology.mjs`
> - `tests/test-canon-draft-topology-structure.mjs`
> - `tests/test-generate-canon-draft-cli-topology.mjs`
> - `tests/test-check-canon-topology.mjs`

---

## Purpose

This review decides whether the topology canon suites can move as one Lane B
Vitest mirror batch. It does not add Vitest suites. The goal is to preserve the
topology classifier, topology structure aggregation and rendering, the
`generate-canon-draft.mjs --source topology` CLI path, and topology drift
contracts without absorbing integration, resolver, full audit orchestration,
performance, incremental cache, deadness, or ranking behavior.

The batch is acceptable because all four suites protect the same canon source
family, but each suite owns a different layer:

- `test-canon-draft-topology.mjs` protects pure topology classifier rules and
  canonical label constants;
- `test-canon-draft-topology-structure.mjs` protects topology inventory
  collection, degraded evidence handling, renderer sections, cross-edge display
  ordering, and workspace boundary rendering using dependency injection;
- `test-generate-canon-draft-cli-topology.mjs` protects the
  `generate-canon-draft.mjs --source topology` CLI path;
- `test-check-canon-topology.mjs` protects topology drift detection, parser
  consistency checks, display-scope behavior, and drift record shape.

The future mirror should keep those layers visible. Shared setup may construct
temporary directories, write synthetic `topology.json` and `triage.json`
objects, run Node commands, and clean up directories, but it must not decide
topology labels, cross-edge source confidence, SCC membership, top-30 display
ordering, CLI source behavior, or drift categories.

## Reviewed Evidence

| Suite                                              | Preserved Node Command                                  | Proposed Focused Vitest Command                         | Surface Under Review                        |
| -------------------------------------------------- | ------------------------------------------------------- | ------------------------------------------------------- | ------------------------------------------- |
| `tests/test-canon-draft-topology.mjs`              | `node tests/test-canon-draft-topology.mjs`              | `npm run test:vitest:canon-draft-topology`              | topology classifier rules and constants     |
| `tests/test-canon-draft-topology-structure.mjs`    | `node tests/test-canon-draft-topology-structure.mjs`    | `npm run test:vitest:canon-draft-topology-structure`    | topology aggregation and renderer contracts |
| `tests/test-generate-canon-draft-cli-topology.mjs` | `node tests/test-generate-canon-draft-cli-topology.mjs` | `npm run test:vitest:generate-canon-draft-cli-topology` | topology CLI draft behavior                 |
| `tests/test-check-canon-topology.mjs`              | `node tests/test-check-canon-topology.mjs`              | `npm run test:vitest:check-canon-topology`              | topology drift engine                       |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane B, canon/check-canon topology family.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share the topology canon source family. The PR must keep every Node
entrypoint runnable and must not absorb helper-registry, naming,
type-ownership, integration, resolver, generated/framework, full audit,
performance, incremental cache, deadness, or ranking suites.

## Protected Invariants

The future Vitest batch must preserve these contracts:

- submodule classification keeps rule precedence:
  `cyclic-submodule`, full-list `isolated-submodule`, `shared-submodule`,
  `leaf-submodule`, then `scoped-submodule`;
- cyclic submodules always win over high fan-in, isolated, and leaf patterns;
- isolated submodules require `crossEdgeSource: "full-list"` and are
  suppressed in degraded `top-30-only` mode;
- shared submodules require in-degree at or above the documented threshold;
- leaf submodules require out-degree greater than in-degree while in-degree is
  below the shared threshold;
- SCC classification remains the constant `forbidden-cycle` label for the
  current topology canon generation;
- file classification keeps the 400 LOC oversize and 1000 LOC extreme-oversize
  thresholds, including defensive non-number handling;
- `TOPOLOGY_LABELS` and `TOPOLOGY_UNCERTAIN_REASONS` remain frozen canonical
  sets;
- topology structure collection derives inventory from `triage.boundaries`,
  then `triage.topDirs`, then `topology.nodes` fallback;
- full `crossSubmoduleEdges` augments degree counts and is preferred over
  legacy `crossSubmoduleTop`;
- degraded `top-30-only` mode keeps classification confidence at `medium` and
  warns instead of pretending full-list evidence exists;
- SCC membership marks affected submodules as cyclic and keeps the SCC section
  visible;
- oversize files below 400 LOC stay filtered out;
- monorepo workspace mode renders workspace boundaries, while single-package
  mode omits that section;
- incomplete or stale topology artifacts produce diagnostics and visible header
  warnings;
- cross-edge display ordering uses count descending and stable ASCII tie-breaks
  before slicing to 30 rows;
- empty cycle output renders an explicit acyclic banner, not a silent omission;
- existing canonical `topology.md` files produce the observational
  existing-canon header;
- `generate-canon-draft.mjs --source topology` remains accepted while unknown
  sources are rejected with the supported source list;
- missing `topology.json` remains exit code 2 with a `measure-topology.mjs`
  recovery hint, distinct from argument errors;
- topology drafts keep non-overwrite versioning, existing-canon headers,
  `--canon-output` overrides, path-with-spaces shell safety, scan-range scope
  text, missing-root errors, missing-triage graceful omission of workspace
  boundaries, incomplete-topology warnings, degraded cross-edge lens warnings,
  and stderr summary counts;
- topology drift keeps missing-canon skip semantics, null-topology parse errors,
  submodule added/removed, SCC-status changed, oversize changed, cross-edge
  added/removed, clean report, and display-scope behavior;
- canon §1/§3 SCC disagreement remains a parse error and does not emit false
  SCC drift;
- structured `crossSubmoduleEdges` is preferred when present, and an empty
  structured list suppresses stale legacy top data;
- every topology drift record keeps `kind: "topology-drift"`;
- topology drift identities stay in their canonical forms: submodule path,
  `<from> → <to>` edge, or owner file path.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- allowing isolated-submodule classification in degraded `top-30-only` mode
  must fail;
- allowing shared or leaf classification to outrank `cyclic-submodule` must
  fail;
- changing the oversize threshold from 400 LOC or extreme threshold from 1000
  LOC must fail;
- mutating or shrinking the topology label and uncertain-reason canonical sets
  must fail;
- deriving inventory only from cross-edges and dropping isolated submodules
  must fail;
- using legacy `crossSubmoduleTop` counts when full `crossSubmoduleEdges` exist
  must fail;
- hiding incomplete or stale topology artifacts as clean evidence must fail;
- rendering degraded topology without `CrossEdgeSource: top-30-only` and
  `ClassificationConfidence: medium` must fail;
- omitting the acyclic banner when no SCCs exist must fail;
- rejecting `--source topology` or omitting supported sources from the
  unknown-source error must fail;
- treating missing `topology.json` as a generic exit 1 argument failure must
  fail;
- overwriting an existing topology draft instead of writing a `.v2` draft must
  fail;
- losing path-with-spaces and `$` shell-safety coverage in the CLI topology
  fixture must fail;
- treating canon §1/§3 SCC disagreement as valid drift must fail;
- including the 31st cross-edge row in the top-30 display scope must fail;
- emitting compound identities for topology drift instead of the category-owned
  identity form must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary may use temporary directories, synthetic topology
  objects, synthetic triage objects, synthetic canon Markdown, and
  argument-safe process calls.
- Shared helper code may construct fixtures, write JSON/Markdown, run commands,
  and clean up directories, but it must not decide topology labels, SCC
  membership, cross-edge confidence, top-30 ordering, CLI source routing, or
  drift categories.
- The mirror must not change `_lib/canon-draft-topology.mjs`,
  `_lib/check-canon-topology.mjs`, `generate-canon-draft.mjs`, or canon output
  contracts.
- The mirror must not absorb helper-registry, naming, type-ownership,
  integration, resolver, generated/framework, deadness/ranking, performance,
  incremental cache, or full audit orchestration suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Verification Snapshot

The review was grounded by running the preserved Node commands before adding
this page:

```text
node tests/test-canon-draft-topology.mjs             # 46 passed, 0 failed
node tests/test-canon-draft-topology-structure.mjs   # 47 passed, 0 failed
node tests/test-generate-canon-draft-cli-topology.mjs # 32 passed, 0 failed
node tests/test-check-canon-topology.mjs             # 38 passed, 0 failed
```

## Recommendation

Proceed to one implementation PR that adds:

1. `tests/canon-draft-topology.test.mjs`,
2. `tests/canon-draft-topology-structure.test.mjs`,
3. `tests/generate-canon-draft-cli-topology.test.mjs`,
4. `tests/check-canon-topology.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving all four suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors with named `it(...)`
cases for the current Node assertions. It should run every preserved Node
command, every focused Vitest command, `npm run test:vitest`, and the wiki/doc
guards.
