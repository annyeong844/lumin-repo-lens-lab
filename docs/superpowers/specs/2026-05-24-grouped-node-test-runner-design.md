# Grouped Node Test Runner Design

Date: 2026-05-24
Status: design for review

## Context

The repository now has two large verification lanes:

- The legacy Node lane, driven by [`scripts/run-tests.mjs`](../../../scripts/run-tests.mjs),
  runs every `tests/test-*.mjs` suite serially and stops on the first failure.
- The reviewed Vitest lane, driven by `npm run test:vitest`, runs the mirrored
  `tests/*.test.mjs` suites.

The latest full verification showed the real cost:

- `npm run test:vitest`: 174 files passed, 1 skipped; 2037 tests passed, 6
  skipped; about 912 seconds.
- `npm test`: 164 suites passed; about 812 seconds.

Running the full Node lane and the full Vitest lane at the same time timed out
and left child processes behind. That was unbounded parallelism. It does not
mean parallelism is impossible; it means "run everything at once" is a bad
runner.

The useful shape is grouped parallelism: run independent groups in parallel,
keep suites serial inside each group, and replay only the failing group when a
group fails.

## Goals

- Keep `npm test` unchanged and authoritative.
- Add an opt-in grouped Node runner path for faster local verification.
- Preserve the existing fresh-process-per-suite isolation from
  [`scripts/run-tests.mjs`](../../../scripts/run-tests.mjs).
- Run suites serially inside each group so related fixture families keep the
  same ordering and failure behavior.
- Run groups with a bounded worker count.
- Make failures easy to debug by printing the failing group and a replay
  command.
- Keep output readable by buffering group logs and printing full logs only for
  failed groups by default.

## Non-Goals

- Do not change `npm test` in this slice.
- Do not make the grouped runner a CI gate until it has dogfood data.
- Do not parallelize every individual suite.
- Do not run Node and Vitest full lanes together by default.
- Do not hide failures behind retry logic.
- Do not rewrite the test harness or migrate more suites to Vitest as part of
  this runner work.

## Decision

Add a separate grouped Node test command in a future implementation slice.

The default `npm test` path remains:

```text
node scripts/run-tests.mjs
```

The new opt-in path should expose commands shaped like:

```text
npm run test:node:groups
npm run test:node:groups -- --jobs 3
npm run test:node:groups -- --group pre-write
npm run test:node:groups -- --group pre-write --serial
npm run test:node:groups -- --list-groups
```

The exact package script name can change during implementation, but it must not
replace `npm test`.

## Runner Model

The grouped runner should:

1. Discover the same `tests/test-*.mjs` suites that the legacy runner discovers.
2. Assign each suite to exactly one deterministic group.
3. Run suites inside a group in sorted order.
4. Run groups concurrently with a bounded `--jobs` value.
5. Spawn each suite as a fresh Node subprocess.
6. Stop scheduling new work once any group fails.
7. Print a summary of all completed groups, their durations, and their suite
   counts.
8. Print full buffered logs for failed groups.
9. Exit non-zero on the first failed group.

This keeps the existing process isolation while cutting wall time when groups
do not contend on the same temp fixtures or expensive shared paths.

## Group Taxonomy

The first implementation should use a checked-in deterministic group table, not
ad hoc substring matching scattered through the runner.

Initial group families should be based on the current suite risk areas:

- `audit-repo`
- `pre-write`
- `post-write`
- `resolver`
- `symbol-graph`
- `module-reachability`
- `function-clone`
- `shape-index`
- `canon`
- `public-surface`
- `wiki-docs`
- `misc`

Unknown suites go to `misc`. That is boring and correct. If `misc` becomes too
large, split it with a follow-up inventory PR instead of teaching the runner
clever guesses.

## Failure Replay

When a group fails, the runner should print both the group replay and the exact
suite replay.

Example:

```text
[run-tests:groups] FAIL group=pre-write suite=test-pre-write-render.mjs exit=1
[run-tests:groups] replay group: npm run test:node:groups -- --group pre-write --serial
[run-tests:groups] replay suite: node tests/test-pre-write-render.mjs
```

This is the whole point of the design. The fast path finds the failing family;
the replay path gives the old serial debugging experience.

## Output Contract

Passing groups should produce compact output:

```text
[run-tests:groups] PASS resolver 18 suites 74.2s
[run-tests:groups] PASS pre-write 22 suites 118.7s
```

Failed groups should print their buffered stdout/stderr after the summary.

Interleaved raw logs from multiple groups are not acceptable. They turn a test
failure into noise soup.

## Safety Rules

- The runner must use a bounded default worker count. Start with 3 or
  `cpuCount - 1`, whichever is lower, with a floor of 1.
- The runner must support `--jobs 1`; this should behave like grouped serial
  execution and be useful for debugging.
- Group assignment must be deterministic across operating systems.
- Suite ordering inside each group must be deterministic.
- The runner must report child process spawn errors explicitly, matching the
  legacy runner's safety fix.
- A group failure must not be retried automatically. Replay is explicit.
- The grouped path must never mutate fixtures outside the existing per-suite
  behavior.
- If a suite is known to be unsafe for grouped execution, it must be assigned
  to a `serial-only` group or forced behind `--jobs 1` until proven safe.

## Test Plan

The implementation slice should add focused tests for the runner helpers before
using the grouped runner for broad verification:

- suite discovery mirrors the legacy runner
- every discovered suite receives exactly one group
- unknown suites fall into `misc`
- `--list-groups` prints deterministic group names and suite counts
- `--group <name>` runs only that group
- `--jobs 1` preserves serial group execution
- spawn errors are reported with the suite name
- failed group output includes replay commands
- passing group output stays compact

The first implementation should then run a small smoke group, not the whole
suite, before asking reviewers to trust the runner.

## Rollout

1. Land this design.
2. Implement the grouped runner as an opt-in command.
3. Run focused helper tests and one small real group smoke.
4. Dogfood with a few local full runs and record timing in the wiki.
5. Only after dogfood data, decide whether to recommend it as a maintainer
   shortcut.
6. Keep `npm test` as the authoritative compatibility lane until the project
   explicitly changes that contract.

## Acceptance Criteria

- `npm test` remains unchanged.
- The grouped runner command is opt-in.
- Grouped execution preserves fresh Node subprocess isolation per suite.
- A failed group prints a group replay command and an individual suite replay
  command.
- Passing group logs do not interleave.
- The first implementation includes unit tests for grouping, CLI selection,
  spawn error handling, and failure replay output.
- Documentation states that the grouped runner is a maintainer shortcut, not a
  replacement for the authoritative Node lane.

## Open Questions

- Which suites need a `serial-only` group after real dogfood runs?
- Should group timing history live in the wiki log or in a lab note?
- Should Vitest get a similar grouped command later, or is Vitest's own runner
  enough once the mirror lane is mature?
