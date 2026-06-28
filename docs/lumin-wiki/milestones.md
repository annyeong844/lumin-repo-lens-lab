# Lumin Wiki Milestones

This page records where the wiki/test-reform work is, what has landed, and what
must happen before the next kind of work starts. It is intentionally small:
`log.md` records chronology, while this page records milestone state.

## Status Vocabulary

- `DONE`: the milestone landed and its immediate checks passed.
- `ACTIVE`: the current work area.
- `NEXT`: the next small candidate after the active milestone.
- `PARKED`: recorded, but intentionally not active.

## Milestone Board

| ID    | Milestone                        |   Status | Landed Evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | Exit Gate                                                                                                                                                                                   |
| ----- | -------------------------------- | -------: | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| WM-01 | Wiki scaffold                    |   `DONE` | `overview.md`, `index.md`, `log.md`, core concept pages, and workstream pages exist.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | New wiki pages must be linked from `index.md` and recorded in `log.md`.                                                                                                                     |
| WM-02 | Risk-based suite inventories     |   `DONE` | Pre-write, resolver, deadness, performance, and public-package workstream pages name protected invariants and edge cases.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | Future test movement must preserve the named invariant and failure mode.                                                                                                                    |
| WM-03 | Fixture-shape comparison         |   `DONE` | `concepts/fixture-shapes.md` names repeated temporary repo, resolver, generated/framework, member, incremental, package, and Markdown mirror shapes.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | A shared helper may only merge setup, not analyzer meaning.                                                                                                                                 |
| WM-04 | Structure review charter         |   `DONE` | `concepts/review-charter.md` defines the shape, function, helper, boundary, anti-pattern, barrel, and test review lens.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | Reviews should report symptom, cause, and first fix before broad cleanup.                                                                                                                   |
| WM-05 | Setup-only temp repo helper      |   `DONE` | `tests/_helpers/temp-repo-fixture.mjs`, `tests/test-temp-repo-fixture-helper.mjs`, and `docs/spec/shared-test-fixture-helper.md`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Helper stays setup-only: no resolver, deadness, pre-write, package, performance, ranking, or command-runner semantics.                                                                      |
| WM-06 | First low-risk helper migrations |   `DONE` | `tests/test-behavior-corpus-verifier.mjs` and `tests/test-update-test-doc.mjs` use the helper while keeping their original assertions local.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | Stop or proceed only one low-risk suite at a time; do not bulk-move tests.                                                                                                                  |
| WM-07 | Test runner migration spec       |   `DONE` | `docs/spec/test-runner-migration.md` covers Node coexistence, Vitest pilot scope, CI impact, rollback, and Bun parking.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | Vitest or Bun implementation must follow the spec rather than direct suite churn.                                                                                                           |
| WM-08 | Vitest pilot                     |   `DONE` | `tests/temp-repo-fixture-helper.test.mjs`, `tests/behavior-corpus-verifier.test.mjs`, `tests/update-test-doc.test.mjs`, `tests/citation-verifier.test.mjs`, `tests/refactor-plan-verifier.test.mjs`, `tests/maintainer-scripts.test.mjs`, `tests/threshold-policy-drift-guard.test.mjs`, `tests/pre-write-lookup-name.test.mjs`, `tests/node-imports-unsupported.test.mjs`, `tests/import-meta-glob-diagnostics.test.mjs`, `tests/output-source-layout-diagnostics.test.mjs`, `tests/generated-artifact-evidence.test.mjs`, `tests/generated-blind-zone-relevance.test.mjs`, `tests/jsonc-edge-cases.test.mjs`, `tests/cli.test.mjs`, `tests/vocab.test.mjs`, `tests/collect.test.mjs`, `tests/shape-hash.test.mjs`, `vitest.config.mjs`, `npm run test:vitest`, `npm run test:vitest:citation-verifier`, `npm run test:vitest:cli`, `npm run test:vitest:collect`, `npm run test:vitest:generated-artifact-evidence`, `npm run test:vitest:generated-blind-zone-relevance`, `npm run test:vitest:import-meta-glob-diagnostics`, `npm run test:vitest:jsonc-edge-cases`, `npm run test:vitest:maintainer-scripts`, `npm run test:vitest:node-imports-unsupported`, `npm run test:vitest:output-source-layout-diagnostics`, `npm run test:vitest:pre-write-lookup-name`, `npm run test:vitest:refactor-plan-verifier`, `npm run test:vitest:shape-hash`, `npm run test:vitest:threshold-policy-drift-guard`, `npm run test:vitest:vocab`, dev-only `vitest`, `pilot-reviews/vitest-temp-repo-fixture.md`, `pilot-reviews/vitest-behavior-corpus-verifier.md`, `pilot-reviews/vitest-update-test-doc.md`, `pilot-reviews/vitest-citation-verifier.md`, `pilot-reviews/vitest-refactor-plan-verifier.md`, `pilot-reviews/vitest-maintainer-scripts.md`, `pilot-reviews/vitest-threshold-policy-drift-guard.md`, `pilot-reviews/vitest-pre-write-lookup-name.md`, `pilot-reviews/vitest-node-imports-unsupported.md`, `pilot-reviews/vitest-import-meta-glob-diagnostics.md`, `pilot-reviews/vitest-output-source-layout-diagnostics.md`, `pilot-reviews/vitest-generated-artifact-evidence.md`, `pilot-reviews/vitest-generated-blind-zone-relevance.md`, `pilot-reviews/vitest-jsonc-edge-cases.md`, `pilot-reviews/vitest-cli.md`, `pilot-reviews/vitest-vocab.md`, `pilot-reviews/vitest-collect.md`, `pilot-reviews/vitest-shape-hash.md`, and `test-migration-candidate-board.md` exercise the current pilot lane while Node entrypoints remain runnable; `vitest.config.mjs` keeps discovery scoped to reviewed pilot files and disables file-level parallelism for producer-backed suites. | Future runner migrations must preserve Node entrypoints, name protected invariants before changing suite shape, and keep Vitest discovery scoped to reviewed pilot files.                   |
| WM-09 | Wiki v1 consolidation            | `ACTIVE` | `overview.md`, `milestones.md`, `index.md`, `log.md`, workstream pages, concept pages, fixture-shape inventory, candidate board, pilot reviews, WT-23 P2 readiness references, and `vitest-mirror-goal.md` are linked into a maintained index.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | Every new wiki page must be linked from `index.md`; every material status change must update `log.md`; stale gating language must be removed when public verification or policy specs land. |
| WM-10 | Bun evaluation                   | `PARKED` | No implementation yet.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | Evaluate only as an optional future runtime/test-runner experiment; do not make public package verification Bun-only.                                                                       |

Current WT-24 evidence also includes
`workstreams/dependency-hygiene.md`,
`pilot-reviews/vitest-unused-deps-producer.md`,
`tests/unused-deps-producer.test.mjs`,
`tests/audit-repo-artifact-brief.test.mjs`, and beta.57/beta.58 public install
evidence for `unused-deps.json`, which keep dependency hygiene review-only:
package declarations may be classified as observed, muted, or `review-unused`,
`manifest.json.unusedDependencies` mirrors shallow counts, reasons, and capped
`topReviewUnused[]` package-name examples, and the default summary/review-pack
Markdown surfaces expose only counts and artifact paths. The dependency hygiene
lane does not create package edits, fix-plan/SARIF entries, package-name
Markdown examples, strong package-edit wording, or `SAFE_FIX` claims.

Current WT-SFC evidence is summarized in
[`wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md`](../lab/wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md)
and governed by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#acceptance-for-current-mvp).
The latest public package verification is
[`WT-SFC beta.81 Nuxt #components alias verification`](../lab/wt-sfc-beta81-nuxt-components-alias-verification-2026-05-31.md),
which follows the beta.80
[`Nuxt app-dir verification`](../lab/wt-sfc-beta80-nuxt-app-dir-verification-2026-05-31.md)
and the beta.79
[`SvelteKit local action regression`](../lab/wt-sfc-beta79-sveltekit-local-action-regression-2026-05-31.md),
which follows the beta.78
[`SFC evidence audit brief verification`](../lab/wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md).
WT-SFC is `MVP`, not `DONE`: current lanes expose bounded SFC evidence and
count-only audit-brief summaries, while full template semantics, custom
framework resolvers, compiler/runtime magic, strong framework absence claims,
and review-only evidence promotion remain out of scope.
The next stronger WT-SFC claim must cite the
[`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md)
and a completed corpus report covering at least one Vue app, one Svelte app,
and one Astro app.
The first mixed smoke corpus is recorded in
[`WT-SFC Vite corpus calibration`](../lab/wt-sfc-vite-corpus-calibration-2026-05-31.md);
it supports keeping the current MVP safe but does not satisfy the full
Vue/Svelte/Astro corpus gate.
The first Astro-only partial pass is recorded in
[`WT-SFC IMA2 Astro corpus calibration`](../lab/wt-sfc-ima2-astro-corpus-calibration-2026-05-31.md);
it exercises Astro script/template evidence but does not close the Astro leg
because no `client:*` framework-convention evidence appeared. The Astro
`client:*` requirement is now covered by
[`WT-SFC Astro client corpus calibration`](../lab/wt-sfc-astro-client-corpus-calibration-2026-05-31.md),
which records 3 muted `sfc-framework-astro-client-directive` records with no
graph, deadness, fix-plan, or export-action leakage.
The Svelte corpus leg is now reviewed in
[`WT-SFC SvelteKit corpus calibration`](../lab/wt-sfc-sveltekit-corpus-calibration-2026-05-31.md):
the beta.78 public package recorded 17 muted
`sfc-framework-svelte-action-directive` records and kept graph/deadness/action
surfaces clean, while exposing a local-action wrapper gap for future policy
work. That gap is now closed by
[`WT-SFC beta.79 SvelteKit local action regression`](../lab/wt-sfc-beta79-sveltekit-local-action-regression-2026-05-31.md):
the beta.79 public package recovered `enhanceWrapper` and `focusAndScroll`,
lowered `missedUsefulEvidenceCount` from 2 to 0, and kept
`falsePositiveCount` and `actionLeakCount` at 0. The first Vue-focused partial
pass is recorded in
[`WT-SFC Vue Options corpus calibration`](../lab/wt-sfc-vue-options-corpus-calibration-2026-05-31.md):
the beta.78 public package recorded muted Vue Options API and template-ref
evidence with no graph/deadness/action leakage, while a supplemental
`nuxt-main` run identified Nuxt `#components` and app-dir component
conventions as custom-resolver/framework gaps. The full Vue corpus leg
is now covered by
[`WT-SFC Storybook Vue corpus calibration`](../lab/wt-sfc-storybook-vue-corpus-calibration-2026-05-31.md):
the beta.78 public package recorded 8 muted template refs, 2 muted Vue Options
API records, and 1 muted global `app.component(...)` registration with no
graph/fan-in/deadness/action leakage. The Vue corpus leg is covered for the
current MVP, but runtime registries, Nuxt custom resolvers, and stronger
absence/action claims remain out of scope. The Nuxt follow-up boundary is now
recorded in
[`WT-SFC Nuxt app-dir and custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md):
generated manifests, root `components/` convention evidence,
`app/components/**`, `#components`, literal component-dir config, custom
resolver functions, and Nuxt layers are separated into explicit-supportable,
muted-observed, or unavailable buckets before implementation. The first
follow-up implementation now records `app/components/**` as muted
`sfc-framework-nuxt-app-dir-convention` evidence only when a Nuxt 4 dependency
or explicit `srcDir: "app"` signal is present, with no graph, fan-in,
deadness, or action-lane effect; a Nuxt 3 dependency alone does not enable the
app-dir root. The next follow-up is publicly verified in
[`WT-SFC beta.81 Nuxt #components alias verification`](../lab/wt-sfc-beta81-nuxt-components-alias-verification-2026-05-31.md)
and records static SFC script imports from
`#components` only when backed by `.nuxt/components.d.ts`, with unresolved
review-only diagnostics for missing mappings and no dependency, graph, fan-in,
deadness, or action-lane effect
([SFC support policy](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
[tests/test-sfc-consumers.mjs](../../tests/test-sfc-consumers.mjs),
[tests/sfc-consumers.test.mjs](../../tests/sfc-consumers.test.mjs)).
Literal component-dir config now emits muted
`sfc-framework-nuxt-components-dir-config` directory evidence from literal
`nuxt.config.*` values without scanning those directories into component
targets; `~/...` and `@/...` config paths resolve through explicit `srcDir` or
the Nuxt 4 default `app/` source directory. Custom resolvers and layers remain
gaps. The
Vue, Svelte, and Astro corpus legs are now all reviewed for current-MVP safety;
WT-SFC remains `MVP`, not `DONE`.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-cue-tiers.md`, which keeps
`tests/test-pre-write-cue-tiers.mjs` parked as a direct mirror and records the
split-track boundary for exact safe cues, class-method review cues, suppressed
diagnostics, service-operation sibling cues, local-operation sibling cues,
unavailable evidence, policy exclusions, file cues, token policy, and
inline-pattern cues. Future Vitest work must review one cue adapter lane at a
time before adding a focused mirror.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-exact-safe-cues.md`, which marks the
exact/signature cue-tier split track as implemented through
`tests/pre-write-exact-safe-cues.test.mjs` and
`npm run test:vitest:pre-write-exact-safe-cues`. The mirror covers only T1-T3
exact and function-signature cue adaptation: exact identities and normalized
signatures create `SAFE_CUE` records, exact-symbol cues stay claim-only and not
semantic-equivalence proof, and mixed safe/review candidates render at
`AGENT_REVIEW_CUE` without dropping either cue record.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-class-method-cues.md`, which marks the
class-method cue-tier split track as implemented through
`tests/pre-write-class-method-cues.test.mjs` and
`npm run test:vitest:pre-write-class-method-cues`. The mirror covers only
T3c-T3d class-method cue adaptation: `classMethodIndex` near-name evidence
creates review-only `AGENT_REVIEW_CUE` cards, cites `classMethodIndex`,
preserves `ClassName#methodName` identities, and never becomes `defIndex`,
`SAFE_CUE`, `EXISTS`, `SAFE_FIX`, or top-level export proof.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-cue-suppressed-diagnostics.md`, which marks the
first cue-tier split track as implemented through
`tests/pre-write-cue-muted.test.mjs` and
`npm run test:vitest:pre-write-cue-muted`. The mirror covers only T4-T4b
suppressed diagnostics: muted semantic and near-name candidates stay in
`suppressedCues[]`, preserve reason/lane/score/distance/locality/token-policy
metadata, and do not create `cueCards[]`, `SAFE_CUE`, `EXISTS`, `SAFE_FIX`, or
`AGENT_REVIEW_CUE` entries.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-service-operation-cues.md`, which marks the
second cue-tier split track as implemented through
`tests/pre-write-service-op-cues.test.mjs` and
`npm run test:vitest:pre-write-service-op-cues`. The mirror covers only T4c-T4g
service-operation cue adaptation: promoted siblings create review-only
`AGENT_REVIEW_CUE` cards with copied policy evidence, original suppressed
diagnostics stay muted, and muted, class-method, generated, or policy-excluded
service-operation candidates stay out of `cueCards[]`.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-local-operation-cues.md`, which marks the third
cue-tier split track as implemented through
`tests/pre-write-local-op-cues.test.mjs` and
`npm run test:vitest:pre-write-local-op-cues`. The mirror covers only T4h-T4j
local-operation cue adaptation: promoted nested local operations create
review-only `AGENT_REVIEW_CUE` cards with copied container, surface, operation,
domain-token, support-reason, and same-file locality evidence, while
mutation-family mismatches stay muted and out of `cueCards[]`.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-unavailable-policy-cues.md`, which marks the
unavailable/policy-excluded cue-tier split track as implemented through
`tests/pre-write-evidence-gaps.test.mjs` and
`npm run test:vitest:pre-write-evidence-gaps`. The mirror covers only T5-T6b
unavailable and policy cue adaptation: missing artifacts remain
`unavailableEvidence[]`, preserve reason and artifact identity, and
policy-excluded exact evidence stays suppressed with original `SAFE_CUE` context
instead of creating `cueCards[]`, `EXISTS`, `SAFE_FIX`, or observed-absence
claims.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-file-token-inline-cues.md`, which marks the
file/token/inline cue-tier split track as implemented through
`tests/pre-write-file-inline-cues.test.mjs` and
`npm run test:vitest:pre-write-file-inline`. The mirror covers only T7-T10
file, token, and inline-pattern cue adaptation: exact file hits create
`SAFE_CUE` evidence, `tokenizePreWrite()` preserves class/process/status and
analysis stems, inline-pattern matches remain review-only extraction cues, and
missing inline-pattern artifacts stay `unavailableEvidence[]` instead of
becoming suppressed cues or observed absence.

Current WT-24 evidence also includes
`pilot-reviews/vitest-export-surface-guards.md`,
`tests/definition-id-export.test.mjs`, `tests/file-delta-export.test.mjs`,
`tests/function-clone-export-surface.test.mjs`,
`tests/classify-policies-export-surface.test.mjs`,
`pilot-reviews/vitest-parser-ast-guards.md`,
`tests/classify-facts-ast.test.mjs`, `tests/lang-matrix.test.mjs`,
`pilot-reviews/vitest-hardcoding.md`, `tests/hardcoding.test.mjs`,
`pilot-reviews/vitest-audit-manifest-export-surface.md`, and
`tests/audit-manifest-export-surface.test.mjs`, and
`pilot-reviews/vitest-definition-id-canonical.md`, and
`tests/definition-id-canonical.test.mjs`, and
`pilot-reviews/vitest-shell-safety.md`, and
`tests/shell-safety.test.mjs`, which keep the Lane A export-surface,
parser/AST, hardcoding, audit-manifest, definition-id canonical, and
shell-safety batches focused on module boundary, AST reference-counting,
language-dispatch, workspace-label, focus-class, public-export,
manifest-summary, cross-producer identity, shell-metacharacter path safety, and
staleness contracts.

Current WT-24 evidence also includes
`pilot-reviews/vitest-evidence-honesty.md` and
`tests/evidence-honesty.test.mjs`, which keep the Lane A evidence-honesty mirror
focused on compare artifact deltas and live doc `.mjs` reference guards.

Current WT-24 evidence also includes
`pilot-reviews/vitest-audit-repo-manifest-performance.md`, which marks the
`tests/test-audit-repo.mjs` manifest/performance split track as implemented
through `tests/audit-repo-manifest-performance.test.mjs` and
`npm run test:vitest:audit-repo-manifest-performance`. The mirror covers only
O0-O3/O1-O1f4: output location notes, `manifest.json.performance`,
`producer-performance.json`, artifact sizes, artifact read/parse counters,
orchestrator memory snapshots, heavy producer phase counters, source-use
resolver timings, and quick-profile producer boundaries.

Current WT-24 evidence also includes
`pilot-reviews/vitest-canon-helper-registry.md`,
`tests/canon-draft-helpers.test.mjs`,
`tests/canon-draft-helper-registry.test.mjs`,
`tests/generate-canon-draft-cli-helpers.test.mjs`, and
`tests/check-canon-helpers.test.mjs`, which keep the Lane B helper-registry
canon batch focused on helper classifier precedence, helper fan-in aggregation,
helper-registry CLI draft behavior, and helper drift evidence gates.

Current WT-24 evidence also includes `pilot-reviews/vitest-canon-naming.md`,
`tests/canon-draft-naming.test.mjs`,
`tests/canon-draft-naming-structure.test.mjs`,
`tests/generate-canon-draft-cli-naming.test.mjs`, and
`tests/check-canon-naming.test.mjs`, which keep the Lane B naming canon batch
focused on naming classifier conventions, cohort aggregation,
`generate-canon-draft.mjs --source naming`, and naming drift evidence gates.

Current WT-24 evidence also includes
`pilot-reviews/vitest-canon-type-ownership.md`,
`tests/canon-draft.test.mjs`, `tests/canon-draft-type-ownership.test.mjs`,
`tests/generate-canon-draft-cli.test.mjs`, and
`tests/check-canon-types.test.mjs`, which keep the Lane B type-ownership canon
batch focused on type classifier rules, type identity aggregation, the
`generate-canon-draft.mjs --source type-ownership` CLI path, and type drift
evidence gates.

Current WT-24 evidence also includes
`pilot-reviews/vitest-canon-topology.md`,
`tests/canon-draft-topology.test.mjs`,
`tests/canon-draft-topology-structure.test.mjs`,
`tests/generate-canon-draft-cli-topology.test.mjs`, and
`tests/check-canon-topology.test.mjs`, which keep the Lane B topology canon
batch focused on topology classifier rules, structure aggregation,
`generate-canon-draft.mjs --source topology`, and topology drift evidence
gates.

Current WT-24 evidence also includes
`pilot-reviews/vitest-canon-integration.md`,
`tests/canon-draft-integration.test.mjs`,
`tests/canon-draft-integration-helpers.test.mjs`, and
`tests/canon-draft-integration-topology.test.mjs`, which keep the Lane B canon
draft integration batch focused on type-ownership, helper-registry, and
topology fixture-to-Markdown integration through the real canon draft CLIs
while excluding check-canon integration, audit-repo orchestration, resolver
expansion, generated/framework, deadness/ranking, performance, and incremental
cache behavior.

Current WT-24 evidence also includes
`pilot-reviews/vitest-canon-drift-contracts.md`,
`tests/canon-drift-parser-contract.test.mjs`, and
`tests/canonical-fact-model-drift.test.mjs`, which keep the Lane B canon drift
contract batch focused on canon renderer table header contracts and fact-model
type-escape schema drift guards while excluding producer integration, resolver
behavior, generated/framework surfaces, deadness/ranking, performance,
incremental cache, and full audit orchestration.

Current WT-24 evidence also includes
`pilot-reviews/vitest-check-canon-core.md`, which marks the Lane B check-canon
core batch as reviewed, and `tests/check-canon-utils.test.mjs`,
`tests/check-canon-artifact.test.mjs`,
`tests/generate-check-canon-cli.test.mjs`, and
`tests/check-canon-integration.test.mjs`, which keep the batch limited to
check-canon parser strictness, canon loader/writer I/O, CLI exit/output policy,
and end-to-end drift fixtures while excluding audit-repo orchestration,
resolver expansion, generated/framework surfaces, deadness/ranking,
performance, and incremental cache behavior.

Current WT-24 evidence also includes
`pilot-reviews/vitest-public-package-publish.md`, which marks the Lane G public
package publish batch as reviewed, and `tests/plugin-package.test.mjs`,
`tests/publish-public-plugin.test.mjs`, and
`tests/github-actions-ci-policy.test.mjs`, which keep the batch focused on
plugin package build output, local public publish git fixtures, and GitHub
Actions CI policy while keeping public skill-surface text suites, hook runtime
suites, analyzer behavior, resolver behavior, generated/framework surfaces,
deadness/ranking, and performance/incremental cache behavior out of scope.

Current WT-24 evidence also includes
`pilot-reviews/vitest-skill-surface.md` and
`tests/skill-surface.test.mjs`, which keep the Lane G public skill-surface text
batch focused on root package metadata, README install and evidence wording,
split SKILL surfaces, command-routing docs, template docs, and public/private
doc staging while keeping `test-skill-package.mjs`, package publishing, hook
runtime suites, analyzer behavior, resolver behavior, deadness/ranking, and
performance/incremental cache behavior out of scope.

Current WT-24 evidence also includes
`pilot-reviews/vitest-skill-package.md`, which marks
`tests/test-skill-package.mjs` as reviewed, and
`tests/skill-package.test.mjs`, which keeps the Lane G generated skill-package
mirror focused on `scripts/build-skill.mjs` generated wrapper scripts, shared
engine relocation, packaged skill surfaces, references/templates/canonical
spine, package metadata, smoke test, Codex wrapper metadata, and dependency
setup behavior while keeping `test-skill-surface.mjs`, plugin packaging,
package publishing, hook runtime suites, analyzer behavior, resolver behavior,
deadness/ranking, and performance/incremental cache behavior out of scope.

Current WT-24 evidence also includes
`pilot-reviews/vitest-hook-runtime.md`,
`tests/hook-doctor.test.mjs`, `tests/hook-runner-scripts.test.mjs`,
`tests/hook-path-safety.test.mjs`, `tests/hook-id-safety.test.mjs`,
`tests/hook-event-store.test.mjs`, `tests/hook-event-drain-renderer.test.mjs`,
`tests/hook-preimage-store.test.mjs`, `tests/hook-ack-observer.test.mjs`, and
`tests/hook-post-write-lite.test.mjs`, which keep the Lane G host hook runtime
batch focused on hook doctor/manifest evidence, host runner stdin/output
behavior, path and id safety, event-store delivery and lock recovery, preimage
privacy, ACK observation, reminder drain/rendering, and post-write-lite
silent-new reminders while keeping
`test-pre-write-inventory-hook.mjs`, pre/post-write advisory tests, package
publishing, skill package/surface tests, analyzer behavior, resolver behavior,
deadness/ranking, and performance/incremental cache behavior out of scope.

Current WT-24 evidence also includes
`pilot-reviews/vitest-post-write-lifecycle.md`,
`tests/post-write-artifact.test.mjs`, `tests/post-write-cli.test.mjs`,
`tests/post-write-delta.test.mjs`, `tests/post-write-incremental.test.mjs`,
`tests/post-write-integration.test.mjs`, and
`tests/post-write-render.test.mjs`, which keep the Lane C post-write lifecycle
batch focused on post-write delta artifact identity, direct post-write CLI
behavior, pure delta classification, after-snapshot incremental routing,
end-to-end pre-write/post-write lifecycle behavior, and Markdown/JSON delta
rendering while keeping pre-write advisory shape tests,
`test-pre-write-inventory-hook.mjs`, cue-tier policy tests, broader audit-repo
lifecycle tests, analyzer behavior, resolver behavior, deadness/ranking, and
performance/incremental cache behavior out of scope.

Current WT-24 evidence also includes
`pilot-reviews/vitest-class-method-prewrite.md`, which marks
`tests/class-method-index-prototype-names.test.mjs` and
`tests/class-method-prewrite-surface.test.mjs` as the Lane C class-method
pre-write surface mirror batch focused on prototype-named class method
dictionary safety, class-method index metadata, `defIndex` non-promotion, and
pre-write near-name review cue visibility.

Current WT-24 evidence also includes `pilot-reviews/vitest-cjs-surface.md`,
`tests/extract-cjs-consumer.test.mjs`,
`tests/extract-cjs-export-surface.test.mjs`,
`tests/cjs-export-surface-artifact.test.mjs`,
`tests/cjs-classification.test.mjs`, and `tests/cjs-integration.test.mjs`,
which keep the Lane D CJS surface mirror batch focused on exact CJS consumer
evidence, CJS export surface facts, broad/opaque opacity, artifact metadata,
and integrated fan-in classification.

Current WT-24 evidence also includes
`pilot-reviews/vitest-framework-resource-surfaces.md`,
`tests/framework-resource-surfaces.test.mjs`,
`tests/build-framework-resource-surfaces.test.mjs`,
`tests/framework-policy-facts.test.mjs`, and
`tests/framework-policy-matrix.test.mjs`, which keep the Lane D
framework/resource surface mirror batch focused on framework/resource surface
artifacts, capability-pack summaries, Hono route facts, package-scoped
framework policy, workspace pattern merging, and framework sentinel/review-hint
counters while keeping broader resolver expansion, deadness/ranking,
action-safety promotion, performance, pre-write cue policy, and full audit
orchestration out of scope.

Current WT-24 evidence also includes
`pilot-reviews/vitest-public-workspace-surfaces.md`, which marks
`tests/test-public-surface.mjs`,
`tests/test-public-deep-import-risk.mjs`,
`tests/test-workspace-no-exports.mjs`, and `tests/test-mdx-consumers.mjs` as a
reviewed Lane D public/workspace surface batch focused on package public
surface collection, public deep-import risk, legacy workspace subpath fallback,
output-to-source aliasing, and MDX consumer fan-in evidence.

Current WT-24 implementation evidence also includes
`tests/public-surface.test.mjs`, `tests/public-deep-import-risk.test.mjs`,
`tests/workspace-no-exports.test.mjs`, and `tests/mdx-consumers.test.mjs`,
which mirror the reviewed Lane D public/workspace surface batch while keeping
the original Node commands runnable.

Current WT-24 evidence also includes
`pilot-reviews/vitest-artifact-output-presentation.md`, which marks
`tests/test-topology-mermaid.mjs` and `tests/test-sarif-fix-plan.mjs` as a
reviewed Lane H artifact-output presentation batch focused on topology
Markdown/Mermaid companion rendering and SARIF fix-plan tier output while
keeping graph computation, resolver behavior, dead-export classification,
ranking policy selection, full audit orchestration, public package install
behavior, and performance measurement out of scope.

Current WT-24 implementation evidence also includes
`tests/topology-mermaid.test.mjs` and `tests/sarif-fix-plan.test.mjs`, which
mirror the reviewed Lane H artifact-output presentation batch while keeping the
original Node commands runnable.

Current WT-24 evidence also includes
`pilot-reviews/vitest-call-graph-evidence.md`, which marks
`tests/test-call-graph-bounded.mjs`,
`tests/test-call-graph-parse-errors.mjs`, and
`tests/test-call-graph-truncation-defense.mjs` as a reviewed Lane H call-graph
evidence batch focused on bounded imported member-call fan-in, parse-error
completeness warnings, and full fan-in preservation outside `topCallees` while
keeping ranking, deadness, action-safety, resolver expansion, full audit
orchestration, performance, and incremental cache behavior out of scope.

Current WT-24 implementation evidence also includes
`tests/call-graph-bounded.test.mjs`,
`tests/call-graph-parse-errors.test.mjs`, and
`tests/call-graph-truncation-defense.test.mjs`, which mirror the reviewed Lane
H call-graph evidence batch while keeping the original Node commands runnable.

Current WT-24 evidence also includes
`pilot-reviews/vitest-threshold-metadata.md`, which marks
`tests/test-threshold-policies.mjs` and
`tests/test-calibration-corpora.mjs` as a reviewed metadata-only threshold
batch focused on policy ids, versions, classes, numeric threshold values,
hashes, calibration corpus references, corpus ids, metrics, compact summaries,
and unknown-corpus errors while keeping threshold drift snapshots, ranking,
deadness, resolver confidence behavior, cue-tier behavior, calibration quality,
and performance out of scope.

Current WT-24 implementation evidence also includes
`tests/threshold-policies.test.mjs` and `tests/calibration-corpora.test.mjs`,
which mirror the reviewed metadata-only threshold batch while keeping the
original Node commands runnable.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-inventory-hook.md`, which marks
`tests/test-pre-write-inventory-hook.mjs` as a reviewed Lane C pre-write
artifact-availability suite focused on invocation-specific
`any-inventory.pre.<invocationId>.json` snapshots and advisory
`preWrite.anyInventoryPath` stamping.

Current WT-24 implementation evidence also includes
`tests/pre-write-inventory-hook.test.mjs`, which mirrors the reviewed pre-write
inventory hook suite while keeping the original Node command runnable.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-lookup-contracts.md`, which marks
`tests/test-pre-write-lookup-dep.mjs`,
`tests/test-pre-write-lookup-file.mjs`,
`tests/test-pre-write-lookup-shape.mjs`, and
`tests/test-pre-write-shape-index.mjs` as a reviewed Lane C pre-write lookup
contract batch focused on dependency availability labels, file status labels,
exact shape-hash evidence, and shape-index integration while keeping
lookup-name service-operation cues, cue-tier policy, renderer wording,
deadness/ranking, resolver expansion, and performance cache identity out of
scope.

Current WT-24 implementation evidence also includes
`tests/pre-write-lookup-dep.test.mjs`,
`tests/pre-write-lookup-file.test.mjs`,
`tests/pre-write-lookup-shape.test.mjs`, and
`tests/pre-write-shape-index.test.mjs`, which mirror the reviewed pre-write
lookup contract batch while keeping the original Node commands runnable.

Current WT-24 evidence also includes
`pilot-reviews/vitest-pre-write-input-contracts.md`, which marks
`tests/test-pre-write-intent.mjs` and
`tests/test-pre-write-canonical-parser.mjs` as a reviewed Lane C pre-write
input contract batch focused on intent normalization, planned type-escape
validation, refactor source safety, canonical owner-claim parsing, free-form
prose rejection, and group-level canonical row exclusion.

Current WT-24 implementation evidence also includes
`tests/pre-write-intent.test.mjs` and
`tests/pre-write-canonical-parser.test.mjs`, which mirror the reviewed
pre-write input contract batch while keeping the original Node commands
runnable.

Current WT-24 evidence also includes
`pilot-reviews/vitest-audit-repo-command-lifecycle.md`, which marks
`tests/test-audit-repo-canon-draft.mjs`,
`tests/test-audit-repo-check-canon.mjs`,
`tests/test-audit-repo-pre-write.mjs`, and
`tests/test-audit-repo-post-write.mjs` as a reviewed audit-repo command
lifecycle wrapper batch focused on manifest command blocks, command-result
summaries, source scoping, advisory versus strict exit-code matrices, evidence
availability mirrors, post-write delta summary fields, stdout/stderr ordering,
and shell-safe paths.

Current WT-24 implementation evidence also includes
`tests/audit-repo-canon-draft.test.mjs`,
`tests/audit-repo-check-canon.test.mjs`,
`tests/audit-repo-pre-write.test.mjs`, and
`tests/audit-repo-post-write.test.mjs`, which mirror the reviewed audit-repo
command lifecycle wrapper batch while keeping the original Node commands
runnable.

Current WT-24 evidence also includes
`pilot-reviews/vitest-mode-dispatch.md`, which marks
`tests/test-mode-dispatch.mjs` as a reviewed Lane C mode-dispatch batch focused
on canonical trigger vocabulary, guard-only non-triggers, repo-context
precedence, prose-rewrite and comment-typo non-triggers, compound
guard-plus-verb firing, return-shape sanity, and deterministic repeat calls.

Current WT-24 implementation evidence also includes
`tests/mode-dispatch.test.mjs`, which mirrors the reviewed mode-dispatch batch
while keeping the original Node command runnable.

## Current Rule

Do not start a test-runner migration until the wiki can answer these questions:

- Which invariant does the migrated suite protect?
- Which edge case should fail if the migration loses meaning?
- Does the helper or runner change only execution mechanics, or does it alter
  analyzer evidence?
- Can the old Node path and the pilot runner path coexist during review?

## Recommended Next Step

The wiki is now the maintained index for WT-23 follow-up policy work and test
reform. The Vitest mirror effort is closed as a broad migration lane in
`vitest-mirror-closure-audit.md`: 165 Node `tests/test-*.mjs` suites and 176
focused Vitest mirrors are recorded, with only two Node-authoritative parked
umbrellas left (`tests/test-audit-repo.mjs` and
`tests/test-pre-write-cue-tiers.mjs`). Future work should not chase more direct
mirrors from that remainder. It should either add a fresh split-track review
for a new product behavior or use the parked-suite dogfooding guide before
touching the umbrella suites.
