# PCEF Calibration Note: cal.com

> **Role:** maintainer-facing calibration note for Proof-Carrying Export Fix.
> **Status:** observation, not a public product claim.
> **Engine:** `lumin-repo-lens-lab` `0.9.0-beta.13`, source commit `488001c`.
> **Target:** unpacked cal.com-derived archive at
> `C:\Users\endof\Downloads\cal.diy-main`.
> **Date:** 2026-05-04.

This note records a large production-monorepo resolver blind-zone case. It is a
calibration anchor for P0 unresolved-spec scope, P2 entry-surface confidence,
and user-facing blind-zone reporting.

## Command

```powershell
node audit-repo.mjs --root "C:\Users\endof\Downloads\cal.diy-main" `
  --output "C:\Users\endof\Downloads\cal.diy-main\.audit-lumin-beta13-production" `
  --profile full --production
```

## Production Scan

- Scan range: 4491 TS/JS production files.
- Confidence: parse errors 0, resolved internal 20569, unresolved internal
  1641 (`unresolvedInternalRatio = 0.0739`).
- `manifest.json.blindZones = []`.
- `symbols.json.topUnresolvedSpecifiers` was dominated by one workspace prefix:
  - `@calcom/`: 1623 unresolved imports, example `@calcom/prisma/client`.
  - `@/`: 1 unresolved import, example `@/pages/api/get-managed-users`.
- `topology.json.summary.unresolvedEdges = 1180`.
- Dead-export tiers:
  - `SAFE_FIX = 27`
  - `REVIEW_FIX = 879`
  - `DEGRADED = 350`
  - `MUTED = 1131`
  - `safeFixGroups = 12`

## Observation

The current engine does not ignore the resolver gap. Many candidates are kept
out of `SAFE_FIX` by `UNRESOLVED_SPEC_MATCH_UNKNOWN` taint when the unresolved
specifier could plausibly match the candidate's scope.

The reporting gap is different: `blindZones` remained empty even though the
scan had 1641 unresolved internal imports and 1180 unresolved topology edges.
The unresolved ratio was below the existing precision-gap threshold, but the
absolute count and prefix concentration were large enough to matter in review.

This means the engine was accurate only inside the graph it successfully
constructed. It should not let users infer that the whole monorepo was
blind-zone free.

## Contract Implication

PCEF and operational reporting need two separate checks:

1. Candidate-local taint:
   unresolved specifiers that could match a finding block `SAFE_FIX` for that
   finding.
2. Run-level precision warning:
   large absolute unresolved counts or a heavily concentrated unresolved prefix
   must surface as a blind-zone or precision warning even when the global ratio
   is below the normal confidence-gap threshold.

Ratio-only reporting is insufficient for large monorepos. A 7% unresolved
ratio can still represent more than a thousand missing import edges.

## Follow-Up Requirements

- Add a run-level resolver precision warning when unresolved internal imports
  exceed an absolute-count threshold.
- Add a prefix-concentration warning when one unresolved alias/workspace prefix
  accounts for most unresolved internal imports.
- Preserve candidate-local taint semantics: positive evidence may improve
  confidence only after relevant unresolved taint is absent.
- Ensure rank/fix reasons distinguish resolver taint from parse-error taint.
  A finding blocked by `UNRESOLVED_SPEC_MATCH_UNKNOWN` should not be summarized
  as only `parse-errors-elsewhere`.

## Calibration Use

Use this case after P0/P2 changes to check:

- `@calcom/` unresolved workspace imports still appear in the manifest or
  summary as a precision warning unless they are truly resolved.
- `blindZones` or equivalent run-level reporting is non-empty for the
  unresolved prefix-concentration case.
- `SAFE_FIX` entries remain limited to candidates without relevant unresolved
  taint.
- The report still says what the graph did and did not cover; it must not imply
  "all clear" for unresolved workspace-package regions.
