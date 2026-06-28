# WT-SFC Beta.78 SFC Evidence Audit Brief Verification

This note records the beta.78 public-install verification for the WT-SFC audit
brief surface described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#audit-brief-surface).

The source implementation landed in
[`lumin-audit` PR #567](https://github.com/annyeong844/lumin_lab/pull/567).
The beta.78 metadata, changelog, and SARIF version bump landed in
[`lumin-audit` PR #568](https://github.com/annyeong844/lumin_lab/pull/568).
The public package landed on public main at
[`2fa1fc5`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/2fa1fc5d4d08f284f547c562c953d40c4ab937d7).
Public Package CI passed for the public main push:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26685684469`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26685684469).

The source guards are
[`test-audit-repo.mjs`](../../tests/test-audit-repo.mjs),
[`audit-repo-artifact-brief.test.mjs`](../../tests/audit-repo-artifact-brief.test.mjs),
[`test-audit-manifest-export-surface.mjs`](../../tests/test-audit-manifest-export-surface.mjs),
[`audit-manifest-export-surface.test.mjs`](../../tests/audit-manifest-export-surface.test.mjs),
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs),
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs),
[`audit-repo-blind-zones.test.mjs`](../../tests/audit-repo-blind-zones.test.mjs),
[`sarif-fix-plan.test.mjs`](../../tests/sarif-fix-plan.test.mjs),
[`smoke-uncovered.test.mjs`](../../tests/smoke-uncovered.test.mjs), and
[`publish-public-plugin.test.mjs`](../../tests/publish-public-plugin.test.mjs).

## Result

PASS. The installed beta.78 package exposes SFC evidence in audit briefs as
count-only orientation. `manifest.json.sfcEvidence`,
`audit-summary.latest.md`, and `audit-review-pack.latest.md` point maintainers
back to `symbols.json` without copying raw component names, tag names, file
spans, or per-record payloads into the brief surfaces.

| #   | Checkpoint                                      | Result | Evidence                                                                                                                                                                            |
| --- | ----------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.78`            | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.78.                                                                        |
| 2   | Public package publish chain is grounded        | PASS   | Source PRs #567/#568 are merged; public main `2fa1fc5` exists and Public Package CI run `26685684469` passed on Node 20 and Node 22 smoke jobs.                                     |
| 3   | `manifest.sfcEvidence` is count-only            | PASS   | The SFC fixture reported 9 total SFC records, 2 script-import records, and 7 review-only records without raw names or spans; SFC-free dogfood reported `sfcEvidence: null`.         |
| 4   | Summary and review-pack wording stays shallow   | PASS   | `audit-summary.latest.md` and `audit-review-pack.latest.md` included only counts and pointers to `manifest.json.sfcEvidence` plus SFC arrays in `symbols.json`.                    |
| 5   | Raw SFC names and action wording do not leak     | PASS   | Fixture component names, directive names, macro names, and API-call strings had zero Markdown occurrences; SFC brief lines had zero `safe`, `remove`, `delete`, `uninstall`, or `drop` wording. |
| 6   | Review-only honesty and `sfc-scan-gap` remain   | PASS   | The fixture retained one `sfc-scan-gap` blind zone; SFC advisory arrays kept `eligibleForFanIn: false` and `eligibleForSafeFix: false`.                                               |
| 7   | Graph, fan-in, deadness, and action lanes clean | PASS   | `resolvedInternalEdges[]` contained only plain import edges; no SFC advisory lane entered fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, or export-action lanes.                 |
| 8   | SARIF tool version matches beta.78              | PASS   | Installed `emit-sarif.mjs` emitted SARIF `run.tool.driver.version` as `0.9.0-beta.78`.                                                                                              |

## Safety Notes

`manifest.json.sfcEvidence` is a navigation surface, not a proof that a
component, directive, macro, or framework convention is consumed. It mirrors
counts from `symbols.json` so maintainers know where to inspect, but it does
not promote review-only SFC evidence into fan-in, deadness, or action-tier
claims.

The default Markdown brief is intentionally shallower than `symbols.json`.
Names such as `MacroCard`, `GlobalCard`, `AsyncCard`, `UsedByAstro`,
`RegisteredSource`, `use:enhance`, `client:load`, `defineOptions`,
`defineAsyncComponent`, and `app.component` stayed out of
`audit-summary.latest.md` and `audit-review-pack.latest.md`.

The `sfc-scan-gap` blind zone still applies. SFC lanes now expose more honest
evidence, but WT-SFC is still an MVP boundary, not full framework semantics.

## Decision

Decision: `sfc-evidence-brief-public-verified`,
`sfc-evidence-stays-count-only`,
`sfc-raw-names-stay-out-of-markdown`, `sfc-scan-gap-stays`, and
`sarif-version-sync-public-verified`.

WT-SFC remains `MVP`, not `DONE`: the audit brief now helps reviewers find SFC
evidence, but it does not turn review-only SFC lanes into consumption,
deadness, or action proof.
