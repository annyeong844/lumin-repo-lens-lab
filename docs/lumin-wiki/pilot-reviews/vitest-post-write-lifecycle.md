# Vitest Post-Write Lifecycle Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-post-write-artifact.mjs`
> - `tests/test-post-write-cli.mjs`
> - `tests/test-post-write-delta.mjs`
> - `tests/test-post-write-incremental.mjs`
> - `tests/test-post-write-integration.mjs`
> - `tests/test-post-write-render.mjs`

---

## Purpose

This review decides whether the post-write lifecycle suites can move together
as one Lane C Vitest mirror batch. It does not add the Vitest suites.

The batch is acceptable because every candidate protects the post-write delta
contract rather than resolver expansion, deadness ranking, package publishing,
or performance cache identity:

- `test-post-write-artifact.mjs` protects delta invocation ids and atomic
  post-write delta artifact writes;
- `test-post-write-cli.mjs` protects the direct `post-write.mjs` CLI,
  advisory loading, baseline discovery, scan-range flags, and shell-safe paths;
- `test-post-write-delta.mjs` protects deterministic delta classification,
  baseline completeness, planned matching, duplicate occurrence handling, and
  acknowledgement requirements;
- `test-post-write-incremental.mjs` protects after-snapshot incremental
  routing without mutating the pre-write baseline artifact;
- `test-post-write-integration.mjs` protects the real pre-write to post-write
  lifecycle through `audit-repo.mjs`;
- `test-post-write-render.mjs` protects Markdown/JSON rendering of the delta
  without turning incomplete evidence into a clean claim.

This batch must stay separate from pre-write advisory shape tests,
`tests/test-pre-write-inventory-hook.mjs`, cue-tier policy tests, and broader
audit-repo orchestration suites.

## Reviewed Evidence

| Suite                                   | Preserved Node Command                       | Proposed Focused Vitest Command              | Surface Under Review                         |
| --------------------------------------- | -------------------------------------------- | -------------------------------------------- | -------------------------------------------- |
| `tests/test-post-write-artifact.mjs`    | `node tests/test-post-write-artifact.mjs`    | `npm run test:vitest:post-write-artifact`    | delta id and artifact writer                 |
| `tests/test-post-write-cli.mjs`         | `node tests/test-post-write-cli.mjs`         | `npm run test:vitest:post-write-cli`         | direct post-write CLI                        |
| `tests/test-post-write-delta.mjs`       | `node tests/test-post-write-delta.mjs`       | `npm run test:vitest:post-write-delta`       | pure delta classifier                        |
| `tests/test-post-write-incremental.mjs` | `node tests/test-post-write-incremental.mjs` | `npm run test:vitest:post-write-incremental` | after-snapshot incremental routing           |
| `tests/test-post-write-integration.mjs` | `node tests/test-post-write-integration.mjs` | `npm run test:vitest:post-write-integration` | pre-write/post-write audit-repo lifecycle    |
| `tests/test-post-write-render.mjs`      | `node tests/test-post-write-render.mjs`      | `npm run test:vitest:post-write-render`      | Markdown and JSON post-write delta rendering |

Current Node evidence checked for this review:

```text
node tests/test-post-write-artifact.mjs    # 18 passed, 0 failed
node tests/test-post-write-cli.mjs         # 32 passed, 0 failed
node tests/test-post-write-delta.mjs       # 88 passed, 0 failed
node tests/test-post-write-incremental.mjs # 3 passed, 0 failed
node tests/test-post-write-integration.mjs # 39 passed, 0 failed
node tests/test-post-write-render.mjs      # 41 passed, 0 failed
```

Goal lane: Lane C, pre/post-write lifecycle. This review covers only the
post-write lifecycle subset of that lane.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all six focused mirrors together because
they share the same post-write delta lifecycle boundary and temporary fixture
style. The mirror must keep every Node entrypoint runnable and must not turn
baseline-missing, scan-range mismatch, parse-error, capability mismatch, or
planned-match ambiguity into a clean success claim.

## Protected Invariants

The future Vitest batch must preserve these post-write contracts:

- delta invocation ids follow the same stable shape and are distinct from the
  pre-write invocation id;
- `post-write-delta.latest.json` and invocation-specific delta artifacts are
  byte-identical after a write;
- prior invocation-specific delta artifacts are preserved across reruns while
  `latest` points at the newest delta;
- invalid invocation ids are rejected before writing artifact paths;
- delta classification is deterministic when `deltaInvocationId` is fixed and
  the pure classifier does not read files, write files, or call time/random
  APIs;
- baseline-available, baseline-missing, baseline-unusable, parse-error, and
  scan-range-mismatch states remain distinct;
- duplicate occurrence keys are compared as multisets and can produce both
  pre-existing and silent-new entries;
- required acknowledgements include only `silent-new` entries;
- planned entries match by exported identity, unknown single-candidate match,
  file-prefix hint, and deterministic absent-from-before selection without
  matching unrelated prefixes;
- ambiguous planned matches carry diagnostics instead of hiding ambiguity;
- capability parity mismatches suppress per-occurrence sections and surface the
  mismatch reason;
- post-write after-snapshot uses incremental any-inventory by default while
  forwarding `--no-incremental`;
- post-write never mutates the pre-write baseline artifact;
- direct CLI runs load advisory baselines by absolute path, advisory directory,
  or scan-range output path where supported;
- direct CLI and audit-repo lifecycle runs preserve stdout/stderr channel
  boundaries and do not prefix Markdown output with diagnostics;
- integration output records manifest postWrite summary fields and writes a
  structurally round-trippable JSON delta;
- renderer section order, caveated summaries, acknowledgement-required
  summaries, capability mismatch states, and clean summaries remain distinct;
- observed-unbaselined and planned-not-observed entries never become required
  acknowledgements.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- artifact writers that leave `.tmp.*` files or overwrite old specific deltas
  must fail;
- missing or non-existent advisory paths must fail with a clear diagnostic;
- spaces and `$` in paths must keep working without shell splitting;
- missing after inventory must remain `capabilityParity.status: "missing"`;
- baseline-missing runs must produce `observed-unbaselined`, not `silent-new`;
- parse-error files in the before inventory must downgrade only affected files;
- scan-range mismatches must suppress removed/silent-new proof where the
  baseline cannot be compared;
- duplicate occurrence keys must not collapse into a single clean existing
  entry;
- ambiguous planned matches must not be silently treated as exact proof;
- Markdown rendering must not show "No silent new any" when evidence is
  incomplete or acknowledgements are required;
- capability mismatch rendering must not emit misleading per-occurrence
  sections;
- audit-repo integration must keep planned and ambiguous remainder entries on
  distinct files where the fixture expects them.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is temporary repos, `.audit` output directories,
  advisory JSON, before/after `any-inventory.json`, and direct imports from the
  post-write delta/render/artifact helpers.
- The mirror may share setup-only temp-root, file-write, JSON-read, and CLI
  invocation helpers inside test files.
- Shared helpers must not decide delta labels, baseline status, capability
  parity, acknowledgement requirement, planned-match semantics, or Markdown
  claim wording.
- The mirror must not absorb pre-write advisory artifact tests,
  `tests/test-pre-write-inventory-hook.mjs`, cue-tier policy tests,
  `tests/test-audit-repo-post-write.mjs`, broader audit-repo lifecycle tests,
  analyzer behavior, resolver behavior, deadness/ranking, or
  performance/incremental cache identity suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/post-write-artifact.test.mjs`,
2. `tests/post-write-cli.test.mjs`,
3. `tests/post-write-delta.test.mjs`,
4. `tests/post-write-incremental.test.mjs`,
5. `tests/post-write-integration.test.mjs`,
6. `tests/post-write-render.test.mjs`,
7. focused `npm run test:vitest:*` commands for each suite,
8. candidate-board updates moving the six suites from `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups
represented as named Vitest `it(...)` blocks. It may share local setup helpers
inside a test file, but no shared helper should decide artifact identity,
baseline proof, delta labels, capability parity, acknowledgement policy,
planned-match ambiguity, or renderer wording.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest`, doc-script checks, and formatting checks
so the reviewed runner discovery boundary and wiki references stay current.
