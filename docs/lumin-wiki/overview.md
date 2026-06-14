# Lumin Wiki Overview

Lumin Repo Lens now has enough specs, lab notes, tests, and verification runs
that conversation memory is no longer a safe index. This wiki is the maintained
synthesis layer between raw evidence and future implementation work.

The wiki is not RAG and not a replacement source of truth. It is a curated map:
the maintainer updates pages as work lands, links each claim to the underlying
spec or test, and records material changes in `log.md`.

## Operating Model

- Read `index.md` first.
- Check `milestones.md` to see which wiki/test-reform phase is active.
- Follow the relevant workstream page.
- Check linked specs, lab notes, and tests before editing code.
- Update the wiki when a workstream's status, risk model, or test strategy
  changes.

## Current State

The wiki is now at **v1 maintainer index** status:

- entrypoint, overview, milestones, and chronology exist;
- pre-write, resolver, deadness, performance, and public-package workstreams
  have risk-based inventories;
- core evidence concepts and the structure review charter are recorded;
- repeated fixture shapes are mapped before broad helper extraction;
- the first setup-only fixture helper and reviewed Vitest pilot lane are
  tracked without replacing existing Node test entrypoints, including the first
  resolver unsupported-family Vitest mirror.

This is enough to orient future work and prevent "we already did that" drift.
It is not yet a complete architecture manual or a complete test migration plan.
When a workstream changes, update the workstream page and `log.md` in the same
PR that changes the underlying spec, test, or lab evidence.

## Current Focus

The immediate focus is dogfooding and test reform:

- stop relying on chat history to remember why an edge case matters
- classify tests by the risk they protect
- merge duplicate fixture shapes only after preserving the original invariant
- write future red tests against real regression behavior

The first wiki slice has moved from scaffold to maintained index. Test movement
and harness extraction should still happen only in smaller PRs with a current
review page for the target suite.

The current milestone board lives in [`milestones.md`](milestones.md). Test
runner changes such as Vitest or Bun should wait until the board records the
spec gate for that migration.
