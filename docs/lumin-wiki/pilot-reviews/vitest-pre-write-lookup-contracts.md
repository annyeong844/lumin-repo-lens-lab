# Vitest Pre-Write Lookup Contracts Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-pre-write-lookup-dep.mjs`
> - `tests/test-pre-write-lookup-file.mjs`
> - `tests/test-pre-write-lookup-shape.mjs`
> - `tests/test-pre-write-shape-index.mjs`

---

## Purpose

This review decides whether the pre-write lookup contract suites can move
together as one Lane C Vitest mirror batch. It does not add the Vitest suites.

The batch is acceptable because every candidate protects structured pre-write
lookup evidence rather than cue-tier promotion, Markdown review wording,
dead-export ranking, resolver expansion, or performance cache identity:

- `test-pre-write-lookup-dep.mjs` protects dependency availability labels,
  package-root normalization, observed-import confidence, and watch-for
  eligibility;
- `test-pre-write-lookup-file.mjs` protects file existence/new/unknown labels,
  topology completeness requirements, parse-error handling, boundary
  non-evaluation, and domain-cluster watch cues;
- `test-pre-write-lookup-shape.mjs` protects exact shape-hash lookup,
  `typeLiteral` normalization, unavailable evidence labels, malformed index
  handling, and the no-heuristic-field-overlap guard;
- `test-pre-write-shape-index.mjs` protects the end-to-end path where
  `build-shape-index.mjs` feeds exact shape evidence into `pre-write.mjs`.

This batch must stay separate from `tests/test-pre-write-lookup-name.mjs`,
service-operation sibling policy, cue-tier policy, pre-write Markdown
rendering, broader pre-write CLI/advisory orchestration, resolver behavior,
deadness/ranking, and performance/incremental cache identity.

## Reviewed Evidence

| Suite                                   | Preserved Node Command                       | Proposed Focused Vitest Command              | Surface Under Review                  |
| --------------------------------------- | -------------------------------------------- | -------------------------------------------- | ------------------------------------- |
| `tests/test-pre-write-lookup-dep.mjs`   | `node tests/test-pre-write-lookup-dep.mjs`   | `npm run test:vitest:pre-write-lookup-dep`   | dependency availability lookup        |
| `tests/test-pre-write-lookup-file.mjs`  | `node tests/test-pre-write-lookup-file.mjs`  | `npm run test:vitest:pre-write-lookup-file`  | file status and domain-cluster lookup |
| `tests/test-pre-write-lookup-shape.mjs` | `node tests/test-pre-write-lookup-shape.mjs` | `npm run test:vitest:pre-write-lookup-shape` | exact shape evidence lookup           |
| `tests/test-pre-write-shape-index.mjs`  | `node tests/test-pre-write-shape-index.mjs`  | `npm run test:vitest:pre-write-shape-index`  | pre-write shape-index integration     |

Current Node evidence checked for this review:

```text
node tests/test-pre-write-lookup-dep.mjs   # 45 passed, 0 failed
node tests/test-pre-write-lookup-file.mjs  # 33 passed, 0 failed
node tests/test-pre-write-lookup-shape.mjs # 30 passed, 0 failed
node tests/test-pre-write-shape-index.mjs  # 5 passed, 0 failed
```

Goal lane: Lane C, pre/post-write lifecycle. This review covers only the
pre-write dependency/file/shape lookup subset of that lane.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share a structured lookup-result boundary and temporary fixture style. The
mirror must keep every Node entrypoint runnable and must not relax evidence
availability, exact-hash shape proof, topology completeness, parse-error, or
domain-cluster semantics.

## Protected Invariants

The future Vitest batch must preserve these lookup contracts:

- `packageRoot()` normalizes bare, scoped, and subpath package specifiers while
  rejecting relative, absolute, malformed scoped, empty, and null specifiers;
- declared dependencies with observed imports return
  `DEPENDENCY_AVAILABLE` with grounded import counts;
- declared dependencies without observed imports remain distinct from declared
  dependencies whose import graph is unavailable;
- unavailable import graph evidence must not be reported as observed count
  zero;
- dependency citations avoid action words such as "unused" and "cleanup";
- sample-only import counts never become watch-for eligible;
- topology or `defIndex` positive evidence can establish `FILE_EXISTS`;
- `NEW_FILE` requires `topology.meta.complete === true`, absence from
  topology nodes, and absence from parse-error files;
- `defIndex` absence alone never establishes `NEW_FILE`;
- parse-error files remain `FILE_STATUS_UNKNOWN`, not new files;
- file boundary status remains `NOT_EVALUATED` in this slice even when a
  blanket allow rule is present;
- missing triage evidence stays cited as unavailable boundary evidence;
- file lookup detects prefix, suffix, and domain-token sibling clusters as
  watch cues without claiming semantic reuse;
- missing shape-index evidence returns `UNAVAILABLE`;
- fields-only shapes remain unavailable even when an index exists;
- exact hash and supported `typeLiteral` lookup can return `SHAPE_MATCH`;
- exact hash misses return `NOT_OBSERVED` only when the shape index is complete;
- malformed shape indexes, invalid hashes, ghost identities, and unsupported
  type literals return `UNAVAILABLE`;
- shape lookup does not fall back to `defIndex`, `symbols.uses`, grep-like
  string matching, or field-overlap heuristics;
- the shape-index integration path renders grounded shape cues from
  `shape-index.json` and records exact matches in advisory JSON.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a relative or absolute dependency-like specifier counted as a package root
  must fail;
- an unavailable import graph reported as grounded zero usage must fail;
- a sample-only dependency count that becomes watch-for eligible must fail;
- topology incompleteness or parse errors producing `NEW_FILE` must fail;
- domain-cluster cues that pull in unrelated semantic siblings must fail;
- boundary evaluation accidentally switching from `NOT_EVALUATED` to
  `ALLOWED` must fail;
- shape lookup that matches by field names alone must fail;
- shape lookup that scans `defIndex`, `symbols.uses`, or source strings as a
  heuristic fallback must fail;
- incomplete shape indexes producing `NOT_OBSERVED` must fail;
- generated shape-index matches that lose exact hash identity, source
  `typeLiteral` normalization, or both matching identities must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is in-memory lookup inputs, temporary roots,
  `shape-index.json`, direct helper imports, and direct `pre-write.mjs`
  invocation for the shape-index integration suite.
- The mirror may share setup-only helpers for temp directories, fixture file
  writes, JSON reads, and exact fixture object construction.
- Shared helpers must not decide dependency availability, import-graph
  confidence, file status, boundary status, topology completeness,
  domain-cluster matching, shape-index completeness, shape-hash equality, or
  advisory rendering semantics.
- The mirror must not absorb `tests/test-pre-write-lookup-name.mjs`,
  `tests/test-pre-write-render.mjs`, `tests/test-pre-write-cli.mjs`,
  `tests/test-pre-write-advisory-artifact.mjs`,
  `tests/test-pre-write-inventory-hook.mjs`, cue-tier policy suites, resolver
  behavior, deadness/ranking, or performance/incremental cache identity suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/pre-write-lookup-dep.test.mjs`,
2. `tests/pre-write-lookup-file.test.mjs`,
3. `tests/pre-write-lookup-shape.test.mjs`,
4. `tests/pre-write-shape-index.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving the four suites from `REVIEWED` to `DONE`.

The implementation PR should first watch at least one focused Vitest command
fail because the script or file is missing, then add mirrors that preserve the
current Node assertion groups as named Vitest cases. It should run the
preserved Node commands, the focused Vitest commands, `npm run test:vitest`,
doc-script checks, and formatting checks.
