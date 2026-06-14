# WT-21 Resolver Capability Backbone Inventory

Maintainer inventory for the first WT-21 architecture-realignment slice. This
note records what is already implemented before adding more resolver or
framework behavior.

## Scope

- Architecture item: WT-21 P1 resolver capability backbone.
- Date: 2026-05-11.
- Purpose: avoid treating WT-21 as purely `SPEC` when resolver capability and
  diagnostics artifacts already exist.

## Implemented Backbone

- `build-resolver-diagnostics.mjs` writes both resolver artifacts:
  - `resolver-capabilities.json`
  - `resolver-diagnostics.json`
- `_lib/resolver-capabilities.mjs` owns:
  - `resolver-capabilities.v1`
  - `resolver-diagnostics.v1`
  - `resolver-2026-05-v1`
  - family status, supported/unsupported cases, reason codes, and absence-claim
    policies.
- `resolver-diagnostics.json` records:
  - unresolved imports with `outputLevel: "unresolved_with_reason"`;
  - diagnostic-only candidate targets with `createsGraphEdge: false`;
  - scoped blind zones;
  - blocked absence hints.
- `manifest.json.resolverDiagnostics` summarizes and points to the full
  capability and diagnostics artifacts.

## Verified Tests

Commands run during inventory:

```text
node tests/test-resolver-diagnostics-artifacts.mjs
node tests/test-resolver-blind-zone-relevance.mjs
```

Observed results:

- `test-resolver-diagnostics-artifacts.mjs`: 7 passed, 0 failed.
- `test-resolver-blind-zone-relevance.mjs`: 8 passed, 0 failed.

Relevant pinned behavior:

- Static capability matrix and per-run diagnostics are separate artifacts.
- Candidate targets are diagnostic-only and do not create graph edges.
- Resolver blind zones carry candidate-relevant blocking policy.
- Generated consumer blind zones carry generated-specific relevance policy.
- Scoped resolver soft taints demote `SAFE_FIX` to `REVIEW_FIX` without becoming
  repo-global blockers.

## Remaining Gaps Before DONE

- Emit explicit unsupported-family resolver events, not only unresolved records
  and static `unsupportedCases`.
- Add a fixture for unsupported Node `#imports` or ambiguous conditional exports
  that produces diagnostic-only `unsupported` output and no concrete graph edge.
- Give framework/resource support named capability-pack ownership before adding
  more ecosystem-specific ranking behavior.
- Verify the capability artifact surface through the public install path before
  claiming user-visible completion.

## Tracker Status

WT-21 should be treated as `MVP`: the resolver capability backbone exists, but
the architecture realignment is not complete.
