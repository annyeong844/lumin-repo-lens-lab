# Pre-Write Workstream

Pre-write is the entry point where Lumin should help an agent before it creates
or edits code. Its strongest product promise is not "this is definitely absent";
it is "here is the grounded evidence, and here is where the evidence is missing."

## Current Themes

- Evidence availability must guard `NOT_OBSERVED` claims.
- Class methods are indexed separately from exported definitions so OO code can
  produce review cues without polluting dead-export evidence.
- Suppressed semantic and near-name candidates are recorded as diagnostics
  before thresholds are relaxed.
- `docs/spec/pre-write-service-operation-sibling-cues.md` records the
  implemented P1 `serviceOperationSiblingPolicy` boundary and the P2 readiness
  gate. P2a maps promoted policy entries into JSON review cue cards, and P2b
  renders those cards in Markdown with review-only service operation wording.
  beta.50 public install verification passed for the focused P2b matrix.
  `docs/lab/wt23-service-operation-corpus-calibration-plan-2026-05-16.md`
  now defines the P3 corpus worksheet required before mutation-family,
  signature-weighted, or threshold changes. The first maintainer-run corpus
  report,
  `docs/lab/wt23-service-operation-corpus-calibration-2026-05-16.md`, found
  that the CLI path keeps service-operation cue cards at zero because name
  intents do not preserve owner locality; owner-aware controls show useful
  Hono helper siblings, while VNplayer's repository operations are mostly
  nested inside `createRepository()` and outside `defIndex`.
- `docs/spec/pre-write-nested-service-operation-surface.md` captures the next
  WT-23 design boundary for VNplayer-style repository factories. The proposed
  local-operation surface is review-only, unavailable evidence must not become
  absence proof, and nested candidates stay out of dead-export ranking and
  `SAFE_FIX` paths. P2a consumes that surface through a separate
  `localOperationSiblingPolicy` lookup evidence object without polluting the
  existing `serviceOperationSiblingPolicy`. P2b maps promoted local-operation
  entries into review-only cue cards and renders `Review related local service
  operation` Markdown while keeping muted entries hidden by default. beta.53
  public install runtime verification passed through the installed
  `audit-repo.mjs --pre-write` entrypoint and confirmed no `SAFE_CUE`,
  `EXISTS`, or `SAFE_FIX` leakage. The beta.53 VNplayer corpus rerun recorded
  `useful-enough` for the local-operation bridge v1: four read/query intents
  rendered nine same-file local-operation review cues, the mutation intent
  rendered none, and service/local policy lanes stayed separate. The follow-up
  support-reason slice adds `local-operation-same-file-domain-overlap` to
  promoted local-operation policy entries so Markdown no longer falls back to
  `unknown`.
- `tests/test-pre-write-local-operation-index.mjs` is the first artifact-only
  WT-23 guard for that boundary. Nested read/query operations inside
  `createRepository()` appear in `preWriteLocalOperationIndex` while staying
  out of `defIndex`, `classMethodIndex`, `nearNames[]`, and `semanticHints[]`.
- Inline extraction cues exist for conservative repeated catch-block patterns.

## Test Inventory

| Suite                                               | Risk Type                | Protected Invariant                                                                                                                 | Edge Case Or Negative Guard                                                                                                                                                           |
| --------------------------------------------------- | ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tests/test-mode-dispatch.mjs`                      | mode-dispatch contract   | `dispatchMode(userText, cwdMeta)` mirrors canonical trigger vocabulary and non-trigger precedence.                                  | Guard-only, prose rewrite, comment typo, pure inspection, and no-repo-context requests must not silently enter pre-write.                                                             |
| `tests/test-pre-write-advisory-artifact.mjs`        | artifact shape           | Pre-write advisory JSON keeps the lifecycle contract stable.                                                                        | Missing fields must be visible as artifact/schema drift, not silently ignored.                                                                                                        |
| `tests/test-pre-write-bootstrap.mjs`                | bootstrap contract       | First-run pre-write setup can produce grounded baseline artifacts.                                                                  | Bootstrap must not make `NOT_OBSERVED` look grounded when required evidence is unavailable.                                                                                           |
| `tests/test-pre-write-canonical-parser.mjs`         | component contract       | Canonical parser inputs are read deterministically for pre-write checks.                                                            | Parser drift should fail before advisory rendering changes.                                                                                                                           |
| `tests/test-pre-write-cli.mjs`                      | CLI contract             | CLI flags and advisory output route pre-write intent into the engine consistently.                                                  | Suppressed diagnostics remain muted evidence and do not become cue cards.                                                                                                             |
| `tests/test-pre-write-cue-tiers.mjs`                | ranking/review lane      | Cue tiers separate `EXISTS`, `SAFE_FIX`, `AGENT_REVIEW_CUE`, and muted evidence.                                                    | Service-operation and local-operation sibling policy entries render only from `promoted[]`; muted, generated, and class-method entries stay out of user-facing cue cards.             |
| `tests/test-pre-write-drift.mjs`                    | drift guard              | Pre-write output shape stays compatible with tracked expectations.                                                                  | Shape drift should fail loudly rather than changing agent-facing semantics silently.                                                                                                  |
| `tests/test-pre-write-inline-patterns.mjs`          | regression edge case     | Explicit refactor sources and inline-pattern artifacts can surface repeated extraction cues.                                        | Inline repeated catch patterns are review cues, not proof that a new helper is safe.                                                                                                  |
| `tests/test-pre-write-integration.mjs`              | lifecycle integration    | Pre-write integrates lookup, evidence, and rendering through the public workflow.                                                   | Integration must preserve evidence labels when a baseline is missing or partial.                                                                                                      |
| `tests/test-pre-write-intent.mjs`                   | component contract       | Intent parsing extracts names, files, shapes, and refactor sources predictably.                                                     | Ambiguous intent should stay advisory rather than becoming grounded absence.                                                                                                          |
| `tests/test-pre-write-inventory-hook.mjs`           | artifact availability    | Pre-write can consume inventory snapshots produced by the hook flow.                                                                | Stale or missing inventory cannot justify `NOT_OBSERVED` as absence.                                                                                                                  |
| `tests/test-pre-write-lookup-dep.mjs`               | lookup contract          | Dependency lookup reports observed package/dependency matches.                                                                      | Unobserved dependency evidence must remain scoped to scan availability.                                                                                                               |
| `tests/test-pre-write-lookup-file.mjs`              | lookup contract          | File lookup identifies existing, near, and missing file targets.                                                                    | Missing scan artifacts should degrade absence claims.                                                                                                                                 |
| `tests/test-pre-write-lookup-name.mjs`              | lookup contract          | Name lookup reports exact, near, class-method, semantic, suppressed candidates, and the service-operation sibling policy object.    | `searchUser` versus `fetchUser` is promoted inside policy evidence without changing formal `nearNames[]` or `semanticHints[]`; cue-card rendering is handled by the separate P2 path. |
| `tests/test-pre-write-local-operation-index.mjs`    | artifact shape           | Nested local repository operations surface as review-only `preWriteLocalOperationIndex` and `localOperationSiblingPolicy` evidence. | Local operations must not enter `defIndex`, `classMethodIndex`, formal lookup lanes, the existing service-operation cue policy, or mutation/generic helper surfaces.                  |
| `tests/test-pre-write-lookup-shape.mjs`             | shape contract           | Shape lookup consumes exact shape-index hashes without heuristic fallback.                                                          | Unsupported or missing shape evidence must stay diagnostic.                                                                                                                           |
| `tests/test-pre-write-render.mjs`                   | renderer contract        | Markdown rendering presents advisory evidence without overclaiming.                                                                 | Service-operation and local-operation sibling cues render as review-only rows; muted/suppressed details should not render as user-facing proof by default.                            |
| `tests/test-pre-write-shape-index.mjs`              | artifact integration     | Pre-write shape lookup consumes `shape-index.json` by exact hash.                                                                   | Shape-index absence or drift should not become a false duplicate claim.                                                                                                               |
| `tests/test-audit-repo-pre-write.mjs`               | orchestrator integration | `audit-repo.mjs --pre-write` routes pre-write through the audit lifecycle.                                                          | A pre-write-only invocation without grounded baseline must expose evidence availability.                                                                                              |
| `tests/test-audit-repo-post-write.mjs`              | adjacent lifecycle guard | Post-write orchestration remains separate from pre-write baseline semantics.                                                        | Post-write must not mutate or reinterpret pre-write baseline evidence.                                                                                                                |
| `tests/test-class-method-prewrite-surface.mjs`      | regression edge case     | Class methods surface as pre-write review cues without entering dead-export `defIndex`.                                             | OO methods such as `handleDelete` must be visible for reuse review but not dead-export proof.                                                                                         |
| `tests/test-class-method-index-prototype-names.mjs` | regression edge case     | Prototype-named methods are stored as ordinary class-method keys.                                                                   | `constructor`, `toString`, `hasOwnProperty`, `valueOf`, and `__proto__` must not crash dictionary grouping.                                                                           |
| `tests/test-inline-pattern-index.mjs`               | producer/artifact shape  | Repeated inline catch-block patterns are collected as review cue evidence.                                                          | Pattern facts must stay review-only until a named extraction policy exists.                                                                                                           |

## Reform Direction

Future pre-write tests should prefer fixtures that explain why a cue was missed
or muted:

- `searchUser` versus `fetchUser` should produce suppressed evidence before any
  threshold change.
- Service-operation sibling promotion emits a versioned policy object first;
  JSON cue cards and Markdown rendering copy that policy only after the P2
  positive/negative fixture matrix passes. beta.50 confirms the public
  Markdown wording; corpus calibration remains separate. The 2026-05-16
  VNplayer/Hono calibration keeps WT-23 in `MVP` because the CLI intent shape
  now carries owner-file locality, but app-local nested repository functions
  require the nested-operation surface rather than defIndex-only service policy;
  `preWriteLocalOperationIndex` is the first
  artifact-only nested local operation surface, `localOperationSiblingPolicy`
  is the lookup-only P2a review evidence surface, and P2b now adds the
  review-only cue-tier/Markdown bridge for app-local repository functions.
  beta.53 public install runtime verification confirmed that bridge before any
  corpus expansion or policy relaxation, and the VNplayer corpus rerun found
  enough local-operation signal to move follow-up from bridge verification to
  support-reason cleanup and separate service-policy type-name filtering. The
  support-reason cleanup now keeps promoted local-operation cue evidence
  explicit with `local-operation-same-file-domain-overlap`.
- Service-operation sibling policy now preserves `definitionKind` on suppressed
  candidates and mutes TypeScript-only declaration names such as
  `ListLibraryDocsOptions` with `service-sibling-non-callable-definition`
  instead of promoting them as callable service operations. beta.55 public
  install verification against the VNplayer corpus confirmed the
  service-operation cue count drops from 2 to 0 while the local-operation lane
  stays stable at nine review-only cues.
- `handleBulkDelete` should see relevant class methods as review cues.
- missing baseline artifacts should mark evidence unavailable, not grounded
  absence.

Do not promote suppressed candidates into review cards until a named cue policy
has corpus evidence.

## Reform Targets

- Keep generated `tests/README.md` descriptions for pre-write suites explicit;
  the first cleanup removed the anonymous pre-write entries from the
  maintainer note.
- Split broad integration assertions only after the protected invariant is
  named in this page.
- Merge fixture shapes only when the shared fixture keeps the original failure
  mode visible.
- Prefer edge-case red tests over helper-existence red tests for new pre-write
  work.
- Keep suppressed candidates muted until the named cue policy has focused
  fixture coverage and public install evidence; keep corpus-facing claims
  behind their own review.
- Treat the read/query sibling policy as review-only: no `SAFE_CUE`, `EXISTS`,
  or reuse-equivalence wording.
- The lookup-name Vitest review page is
  `docs/lumin-wiki/pilot-reviews/vitest-pre-write-lookup-name.md`. The mirror
  now exists at `tests/pre-write-lookup-name.test.mjs`; keep
  `node tests/test-pre-write-lookup-name.mjs` runnable and preserve the
  suppressed-diagnostic edge cases named there.
- The pre-write inventory hook Vitest review page is
  `docs/lumin-wiki/pilot-reviews/vitest-pre-write-inventory-hook.md`. The
  mirror now exists at `tests/pre-write-inventory-hook.test.mjs` and stays
  limited to `any-inventory.pre.<invocationId>.json` snapshot stamping and
  advisory pointer availability; hook-failure strengthening needs its own Node
  assertion first.
- The pre-write lookup contracts Vitest review page is
  `docs/lumin-wiki/pilot-reviews/vitest-pre-write-lookup-contracts.md`. The
  mirrors now exist at `tests/pre-write-lookup-dep.test.mjs`,
  `tests/pre-write-lookup-file.test.mjs`,
  `tests/pre-write-lookup-shape.test.mjs`, and
  `tests/pre-write-shape-index.test.mjs`. They cover dependency/file/shape
  lookup and shape-index integration together while keeping
  `tests/test-pre-write-lookup-name.mjs`, cue-tier policy, renderer wording,
  deadness/ranking, resolver expansion, and performance cache identity out of
  scope.
- The pre-write input contracts Vitest review page is
  `docs/lumin-wiki/pilot-reviews/vitest-pre-write-input-contracts.md`. The
  mirrors now exist at `tests/pre-write-intent.test.mjs` and
  `tests/pre-write-canonical-parser.test.mjs`. They cover
  `tests/test-pre-write-intent.mjs` and
  `tests/test-pre-write-canonical-parser.mjs` together while keeping
  bootstrap, mode dispatch, CLI/advisory orchestration, lookup-name policy,
  cue-tier policy, renderer wording, resolver behavior, deadness/ranking, and
  performance cache identity out of scope.
- The audit-repo command lifecycle Vitest review page is
  `docs/lumin-wiki/pilot-reviews/vitest-audit-repo-command-lifecycle.md`. Its
  mirrors now exist at `tests/audit-repo-pre-write.test.mjs` and
  `tests/audit-repo-post-write.test.mjs` alongside canon/check-canon wrapper
  mirrors. They cover audit-repo manifest blocks, command-result summaries,
  evidence availability mirrors, post-write delta summary fields, and
  exit-code routing while keeping direct pre-write or post-write component
  suites, cue tiers, renderer wording, resolver behavior, deadness/ranking, and
  performance cache identity out of scope.
- The mode-dispatch Vitest review page is
  `docs/lumin-wiki/pilot-reviews/vitest-mode-dispatch.md`. The mirror now
  exists at `tests/mode-dispatch.test.mjs` and covers only
  `tests/test-mode-dispatch.mjs`: canonical mode-contract vocabulary drift,
  guard-only non-triggers, repo-context precedence, prose-rewrite and
  comment-typo non-triggers, compound guard-plus-verb firing, return-shape
  sanity, and deterministic repeat calls.
