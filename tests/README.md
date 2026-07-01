<!--
  GENERATED FILE — do not edit by hand.
  Source: CHANGELOG.md + tests/test-*.mjs files.
  Regenerate with: npm run update-test-doc
  CI guard: npm run check:test-doc (exits non-zero if stale)
-->

# Tests

Regression guards built up across releases. Each change that could
have broken a correctness property got a corresponding assertion so
the next regression fails fast.

The authoritative assertion count is the output of `npm test`. This
README intentionally avoids hardcoding a total — four consecutive
releases (1.8.2 → 1.8.5) drifted the number in this file, so the
number was removed and the file became generated.

## Run

```bash
cd <skill-dir>
npm install        # first run only
npm test           # all suites, stops at first failing assertion
```

## Suite Map

- **Smoke:** start with `test-skill-package.mjs` and
  `test-skill-surface.mjs` when packaging or prompt surface changes.
- **Contract:** suites named `test-pre-write-*`, `test-post-write-*`,
  `test-canon-*`, `test-check-canon-*`,
  `test-generate-canon-draft-*`, and
  `test-generate-check-canon-*` guard lifecycle artifacts and CLI
  contracts.
- **Regression:** the remaining suites pin resolver, parser, ranking,
  false-positive, drift, and fixture-specific behavior. Run them when
  touching shared engine logic or before release.

Or run individual suites:

```bash
node tests/test-alias.mjs                                # export alias misclassification
node tests/test-any-inventory-incremental.mjs            # strict incremental any-inventory cold/warm equivalence + changed/deleted file behavior
node tests/test-any-inventory.mjs                        # any-inventory.json producer + meta shape (P2-0)
node tests/test-audit-manifest-export-surface.mjs        # audit-manifest export surface hides living-audit internals and mirrors review-only evidence summaries
node tests/test-audit-repo-canon-draft.mjs               # orchestrator --canon-draft + --sources CANON_DRAFT_SOURCES + thin-wrapper pin (P3-4)
node tests/test-audit-repo-check-canon.mjs               # audit-repo.mjs --check-canon orchestrator + manifest.checkCanon shape + advisory vs --strict-check-canon + child exit 1/2 are per-source outcomes (P5-4)
node tests/test-audit-repo-post-write.mjs                # audit-repo --post-write wiring + manifest summary fields (P2-2)
node tests/test-audit-repo-pre-write.mjs                 # audit-repo --pre-write lifecycle wiring and missing-baseline evidence availability
node tests/test-audit-repo-symbol-incremental.mjs        # audit-repo forwards strict incremental flags to build-symbol-graph
node tests/test-audit-repo.mjs                           # orchestrator profiles + blindZones detection
node tests/test-behavior-corpus-verifier.mjs             # saved-answer behavior verifier: offline no-jargon, caveat, and summary-order checks
node tests/test-build-block-clone-index.mjs              # build-block-clone-index.mjs producer: repeated token/block clone review-only artifact
node tests/test-build-framework-resource-surfaces.mjs    # build-framework-resource-surfaces.mjs producer and audit-repo artifact visibility
node tests/test-build-function-clone-index.mjs           # build-function-clone-index.mjs producer: exported helper/function clone cue artifact
node tests/test-build-shape-index.mjs                    # P4-2 build-shape-index.mjs producer: shape-index artifact, grouping, diagnostics, scan scope
node tests/test-calibration-corpora.mjs                  # calibration corpus registry anchors threshold policy corpus references
node tests/test-call-graph-bounded.mjs                   # PCEF P3 bounded member-call resolution for exported object member calls
node tests/test-call-graph-parse-errors.mjs              # call-graph artifact marks parse-error scans incomplete with file-level diagnostics
node tests/test-call-graph-truncation-defense.mjs        # PCEF P3 call graph full fan-in maps remain complete beyond topCallees display truncation
node tests/test-canon-draft-helper-registry.mjs          # helper aggregator + renderer via DI; PF-3/PF-4 fan-in pins (P3-2)
node tests/test-canon-draft-helpers.mjs                  # helper classifier rules (group + single-identity) + precedence pins (P3-2)
node tests/test-canon-draft-integration-helpers.mjs      # end-to-end helper-registry via real extractor + resolver (P3-2)
node tests/test-canon-draft-integration-topology.mjs     # end-to-end topology draft via measure-topology + triage-repo + canon CLI (P3-3)
node tests/test-canon-draft-integration.mjs              # end-to-end symbols→canon-draft via fixture repos (P3-1)
node tests/test-canon-draft-naming-structure.mjs         # naming aggregator + renderer via DI; cohort inventory always from collectFiles (P3-4)
node tests/test-canon-draft-naming.mjs                   # naming classifier + detectConvention + normalizeFileBasename + low-info Rule 0 (P3-4)
node tests/test-canon-draft-topology-structure.mjs       # topology aggregator + renderer via DI; §5.3.1 inventory source order + degraded-mode guard (P3-3)
node tests/test-canon-draft-topology.mjs                 # topology classifier rules (§11.1 submodule + §11.2 SCC + §11.3 oversize) (P3-3)
node tests/test-canon-draft-type-ownership.mjs           # identity aggregation + renderer scenarios (P3-1)
node tests/test-canon-draft.mjs                          # type classifier rules (group + single-identity) + markdown helpers (P3-1)
node tests/test-canon-drift-parser-contract.mjs          # round-trip: each P3 renderer header matches canon-drift.md §5 column contract (P5-0)
node tests/test-canonical-fact-model-drift.mjs           # canonical §3.9 escapeKind drift guard (P2-0)
node tests/test-check-canon-artifact.mjs                 # I/O layer — load{TypeOwnership,HelperRegistry,Topology,Naming}Canon + writeCanonDriftArtifacts no-merge policy (P5-1..P5-4)
node tests/test-check-canon-helpers.mjs                  # helper-registry drift engine + evidence-gated contamination dispatch (per-identity) + fan-in-tier-changed + extractor-throw → parse-error promotion (P5-2)
node tests/test-check-canon-integration.mjs              # end-to-end drift via 5 type + 6 helper + 7 topology fixtures; canonical bytes immutable; stale canonical-draft ignored (P5-1..P5-3)
node tests/test-check-canon-naming.mjs                   # naming drift engine + cohort + outlier sub-diffs + PF-4 identity format per category (file/symbol cohort / file/symbol outlier) (P5-4)
node tests/test-check-canon-topology.mjs                 # topology drift engine + 3 sub-diffs (submodules/oversize/cross-edges) + §1/§3 SCC agreement + top-30 sort-before-slice (P5-3)
node tests/test-check-canon-types.mjs                    # type-ownership drift engine + 1:1 owner-change upgrade + label-preserving renderer (P5-1)
node tests/test-check-canon-utils.mjs                    # PURE parser (3-tier strictness) for all 4 sources + multi-section topology + multi-section naming + makeDriftRecord + buildCanonDriftJsonObject + 4 LABEL_SETs (P5-1..P5-4)
node tests/test-checklist-facts.mjs                      # checklist-facts.mjs pre-compute layer (v1.10.2)
node tests/test-citation-verifier.mjs                    # Rule 1 grounded citation verifier: artifact path/value checks for saved model output
node tests/test-cjs-classification.mjs                   # PCEF P0 CJS consumer classification through symbol graph + dead-export pipeline
node tests/test-cjs-export-surface-artifact.mjs          # CJS export surface facts survive into symbols.json for downstream blind-zone handling
node tests/test-cjs-integration.mjs                      # CJS export surface, alias destructuring, and dynamic require opacity integration guard
node tests/test-class-method-index-prototype-names.mjs   # class method index handles prototype method names as plain dictionary keys
node tests/test-class-method-prewrite-surface.mjs        # WT-15 class method index remains separate from defIndex and feeds pre-write review cues
node tests/test-classification-gates.mjs                 # canonical §3/§9/§10.3/§10.4/§11.4/§12.3 label-set + LOW_INFO + TOPOLOGY + NAMING mirrors drift-lock (P3-1..P3-4) + canon-drift.md §3 category/family mirror (P5-0)
node tests/test-classification-label-emission-corpus.mjs # synthetic TS corpus proving canonical type classification labels emit through build-symbol-graph → canon-draft
node tests/test-classify-facts-ast.mjs                   # AST identifier ref counting (v1.10.0 P0)
node tests/test-classify-performance-metadata.mjs        # classify performance metadata + safe text-zero shortcut
node tests/test-classify-policies-export-surface.mjs     # classify-policies export surface stays limited to active policy APIs
node tests/test-cli.mjs                                  # CLI flag parsing
node tests/test-collect.mjs                              # collectFiles language filter
node tests/test-corpus.mjs                               # precision corpus + FP budget gate (v1.10.0 P2)
node tests/test-definition-id-canonical.mjs              # PCEF P3 canonical definitionId shared by symbols, action-safety, and call-graph alias fan-in
node tests/test-definition-id-export.mjs                 # definition-id export surface hides raw id builder
node tests/test-dynamic-import.mjs                       # topology dynamic imports
node tests/test-entry-surface-artifact.mjs               # PCEF P2b entry-surface artifact and audit pipeline hook
node tests/test-evidence-honesty.mjs                     # compare-repos + doc-script-refs guards
node tests/test-export-action-safety.mjs                 # PCEF P1 export-action-safety producer: demote/delete proof, blockers, and module marker patch
node tests/test-extract-cjs-consumer.mjs                 # PCEF P0 direct CJS require consumer extraction: exact, side-effect-only, and broad escape
node tests/test-extract-cjs-export-surface.mjs           # Direct CJS export surface extraction: exact exports plus opaque export forms
node tests/test-extract-ts-escapes.mjs                   # 11 escapeKind extractor + occurrenceKey stability (P2-0)
node tests/test-file-delta-export.mjs                    # post-write file-delta export surface hides path normalizer internals
node tests/test-finding-local-provenance.mjs             # per-finding taintedBy/supportedBy (v1.10.0 P1)
node tests/test-framework-policy-facts.mjs               # framework policy fact extraction for config/framework sentinel evidence
node tests/test-framework-policy-matrix.mjs              # framework policy matrix contract for config and framework sentinel muting
node tests/test-framework-resource-surfaces.mjs          # framework/resource surface classifier lanes for stories, Strapi paths, generated declarations, bundles, templates, and codemod resources
node tests/test-function-clone-audit-forwarding.mjs      # audit-repo incremental flag forwarding for function clone producer
node tests/test-function-clone-export-surface.mjs        # function-clone artifact export surface hides version internals
node tests/test-function-clone-incremental.mjs           # strict incremental build-function-clone-index cold/warm equivalence + changed/deleted file behavior
node tests/test-generate-canon-draft-cli-helpers.mjs     # CLI --source helper-registry + versioning + stale call-graph (P3-2)
node tests/test-generate-canon-draft-cli-naming.mjs      # CLI --source naming + versioning + scope + regression (P3-4)
node tests/test-generate-canon-draft-cli-topology.mjs    # CLI --source topology + exit 2 on missing topology.json (P3-3)
node tests/test-generate-canon-draft-cli.mjs             # generate-canon-draft.mjs CLI flags + versioning + scope (P3-1)
node tests/test-generate-check-canon-cli.mjs             # check-canon.mjs CLI flags + exit matrix + per-source artifact strictness asymmetry + --source all aggregation by checked-source rule (P5-1..P5-4)
node tests/test-generated-artifact-evidence.mjs          # generated artifact evidence policy: strong build/static output quorum plus supporting path-segment hints
node tests/test-generated-blind-zone-relevance.mjs       # generated artifact blind-zone relevance scoping for SAFE_FIX taint
node tests/test-generated-consumer-blind-zones.mjs       # generated consumer blind-zone inventory in symbols.json for missing or excluded generated surfaces
node tests/test-generated-virtual-surface.mjs            # generated virtual surface contract for Prisma enum imports without generator execution
node tests/test-github-actions-ci-policy.mjs             # GitHub Actions CI policy guard: draft PRs skip runner jobs while ready/manual/push still run
node tests/test-hardcoding.mjs                           # workspace labels, focus-class
node tests/test-hash-imports.mjs                         # Node `#imports` subpath — exact, wildcard, and suffix wildcard alias resolution
node tests/test-hook-ack-observer.mjs                    # auto-hook Phase 1E Stop ACK observer core
node tests/test-hook-doctor.mjs                          # auto-hook Phase 1A hook manifest and doctor smoke test
node tests/test-hook-event-drain-renderer.mjs            # auto-hook Phase 1D event drainer and reminder renderer core
node tests/test-hook-event-store.mjs                     # auto-hook Phase 1C session event store core
node tests/test-hook-id-safety.mjs                       # auto-hook Phase 1A session/tool id safety helpers
node tests/test-hook-path-safety.mjs                     # auto-hook Phase 1A path/root safety helpers
node tests/test-hook-post-write-lite.mjs                 # auto-hook Phase 1F post-write-lite silent-new event generation core
node tests/test-hook-preimage-store.mjs                  # auto-hook Phase 1B session preimage store
node tests/test-hook-runner-scripts.mjs                  # auto-hook Phase 1G hook runner scripts and manifest activation
node tests/test-import-meta-glob-diagnostics.mjs         # import.meta.glob unsupported dynamic-module diagnostics
node tests/test-incremental-cache-store.mjs              # strict incremental cache store schema, current-hash reuse, malformed-cache fallback
node tests/test-incremental-snapshot.mjs                 # strict incremental repo snapshot identity, content hashes, unreadable file visibility
node tests/test-incremental.mjs                          # file-hash cache + stat-first-cut fast path (E-6)
node tests/test-inline-pattern-index.mjs                 # build-inline-pattern-index.mjs producer: repeated inline catch-block review cue artifact
node tests/test-js-module-edge-scanner.mjs               # tokenizer-state JS module edge scanner shadow/equivalence fixtures
node tests/test-jsonc-edge-cases.mjs                     # JSONC tsconfig parser edge cases: schema URLs, comments, trailing commas, string comment markers, BOM, and unresolved extends
node tests/test-lang-matrix.mjs                          # per-extension parser dispatch
node tests/test-maintainer-scripts.mjs                   # maintainer script hardening: child process spawn errors and optional public package reads
node tests/test-mdx-consumers.mjs                        # P6-1 MDX import consumers: docs-driven component imports contribute symbol fan-in without file-level overprotection
node tests/test-mode-dispatch.mjs                        # mode dispatch contract: write triggers, non-trigger reasons, repo context, prose rewrites, comment typo fixes, and inspection guards
node tests/test-module-reachability.mjs                  # PCEF P2c module-reachability artifact: runtime/type BFS, bounded-out cap, and audit pipeline hook
node tests/test-namespace-reexport-deadness.mjs          # namespace re-export member fan-in: exact/chained used members stay live, unused members remain dead, opaque escapes are diagnosed
node tests/test-node-imports-unsupported.mjs             # Node #imports unsupported-family diagnostics: no external fallback, no graph edge, dedicated unsupported lane
node tests/test-output-source-layout-diagnostics.mjs     # package exports output-to-source layout unsupported diagnostics: no fake edge, dedicated family, candidate-scoped blind zone
node tests/test-p6-measurement.mjs                       # P6-0 measurement artifact contract: candidate counts, FP denominator, schema round-trip, dirty corpus, readiness gates
node tests/test-p6-member-precision.mjs                  # P6-3 namespace and dynamic import member precision: direct member calls protect only the called export; degraded aliases stay conservative
node tests/test-p6-safe-fix-calibration.mjs              # P6 SAFE_FIX calibration corpus: real mini git repo + runtime/staleness convergence + P6 measurement
node tests/test-plugin-package.mjs                       # Claude Code plugin-root package builder: plugin metadata, slash commands, generated skill surfaces, Codex wrapper opt-in
node tests/test-post-write-artifact.mjs                  # post-write-delta dual-write + atomic (P2-1)
node tests/test-post-write-cli.mjs                       # post-write.mjs CLI smoke + scan-range flag forwarding (P2-1)
node tests/test-post-write-delta.mjs                     # computeDelta: 6-label classification + purity (P2-1)
node tests/test-post-write-incremental.mjs               # post-write after-snapshot incremental forwarding + immutable pre-write baseline
node tests/test-post-write-integration.mjs               # release-blocking end-to-end: multi-label + baseline-missing fixtures (P2-2)
node tests/test-post-write-render.mjs                    # post-write Markdown + JSON render (P2-1)
node tests/test-pre-write-advisory-artifact.mjs          # pre-write advisory artifact shape, lifecycle metadata, and evidence availability contract
node tests/test-pre-write-bootstrap.mjs                  # pre-write first-run bootstrap keeps missing baseline evidence explicitly unavailable
node tests/test-pre-write-canonical-parser.mjs           # pre-write canonical parser keeps owner claims deterministic before lookup rendering
node tests/test-pre-write-cli.mjs                        # pre-write CLI intent parsing, baseline evidence routing, and advisory output contract
node tests/test-pre-write-cue-tiers.mjs                  # pre-write cue tier artifact contract and weak-token suppression classification
node tests/test-pre-write-drift.mjs                      # pre-write canonical/AST drift states stay structured and scoped
node tests/test-pre-write-inline-patterns.mjs            # pre-write inline extraction cues from explicit refactorSources and inline-patterns.json
node tests/test-pre-write-integration.mjs                # pre-write end-to-end lookup, evidence availability, and advisory rendering integration
node tests/test-pre-write-intent.mjs                     # pre-write intent parser extracts names, files, shapes, and refactor sources without overclaiming
node tests/test-pre-write-inventory-hook.mjs             # pre-write P2-0 snapshot hook (preWrite.anyInventoryPath)
node tests/test-pre-write-local-operation-index.mjs      # pre-write nested local operation index stays review-only and out of export lookup lanes
node tests/test-pre-write-lookup-dep.mjs                 # pre-write dependency lookup distinguishes observed package evidence from unavailable scan evidence
node tests/test-pre-write-lookup-file.mjs                # pre-write file lookup surfaces exact, near, missing, and evidence-unavailable targets
node tests/test-pre-write-lookup-name.mjs                # pre-write name lookup exact identities, suppressed diagnostics, and service-operation sibling policy evidence
node tests/test-pre-write-lookup-shape.mjs               # P4 pre-write shape lookup: exact hash/typeLiteral, schema validation, no heuristic fallback
node tests/test-pre-write-render.mjs                     # pre-write Markdown renderer keeps advisory evidence review-only and avoids stronger action wording
node tests/test-pre-write-shape-index.mjs                # P4-3 pre-write shape lookup consumes shape-index.json by exact hash
node tests/test-public-deep-import-risk.mjs              # PCEF P2 public package exports risk gate for entry-unreachable confidence support
node tests/test-public-surface.mjs                       # P6-1 package/public surface collector: root workspace package entries, declaration targets, script-driven tsup/rollup/esbuild entrypoints, HTML module entrypoints
node tests/test-publish-public-plugin.mjs                # public plugin repo publisher: generated package allowlist, changelog prepend, dry-run, and push flow
node tests/test-python-conventions.mjs                   # Python __all__, decorators, dunders
node tests/test-rank-fixes.mjs                           # 4-tier fix-plan ranking predicates + merge
node tests/test-refactor-plan-verifier.mjs               # refactor-plan output verifier: SHORT/FULL shape, tone guard, evidence anchor, pre-write handoff
node tests/test-resolved-edges.mjs                       # PCEF P2a resolvedInternalEdges file-level graph artifact
node tests/test-resolver-blind-zone-relevance.mjs        # resolver blind-zone relevance scoping for per-finding SAFE_FIX taint
node tests/test-resolver-diagnostics-artifacts.mjs       # resolver capabilities and per-run diagnostics artifact contract
node tests/test-resolver-paths.mjs                       # resolver edge cases (FP-16 etc.)
node tests/test-run-tests-grouped.mjs                    # grouped Node test runner: deterministic groups, bounded jobs, compact logs, and replay commands
node tests/test-sarif-fix-plan.mjs                       # emit-sarif fix-plan branch: tier → SARIF level
node tests/test-sfc-consumers.mjs                        # SFC consumers: script imports, script-src reachability, style assets, template refs, and global registration evidence stay in separate lanes
node tests/test-shape-hash.mjs                           # P4-1 shape-hash pure core: field normalization, stable hashes, unsupported-shape diagnostics
node tests/test-shape-index-incremental.mjs              # strict incremental build-shape-index cold/warm equivalence + changed/deleted file behavior
node tests/test-shell-safety.mjs                         # shell injection + triage refactor
node tests/test-skill-package.mjs                        # deployable skill package builder: plugin wrapper, 5 public scripts, slash commands, _engine internals, canonical/templates/references, no lab payload
node tests/test-skill-surface.mjs                        # product surface contract: shared audit engine + 3 skill surfaces + stable validation modes + internal-vs-public doc split
node tests/test-smoke-uncovered.mjs                      # scripts without dedicated suites
node tests/test-symbol-graph-incremental.mjs             # strict incremental build-symbol-graph cold/warm equivalence + changed/deleted file behavior
node tests/test-symlink-aliasing.mjs                     # symlink canonicalization
node tests/test-temp-repo-fixture-helper.mjs             # shared temporary repo fixture helper safety contract
node tests/test-threshold-policies.mjs                   # threshold policy metadata: policy ids, versions, hashes, and compact artifact summaries
node tests/test-threshold-policy-drift-guard.mjs         # threshold policy numeric drift requires an explicit snapshot review
node tests/test-topology-mermaid.mjs                     # topology.mermaid.md renderer contract: diagrams, hub files, caps, and citation guardrails
node tests/test-topology-producer-cross-edges.mjs        # measure-topology.mjs crossSubmoduleEdges full-list producer shape pin (P3-3-pre)
node tests/test-tsconfig-paths-scoped.mjs                # FP-36: scope-aware tsconfig paths in monorepos
node tests/test-type-only-reexport.mjs                   # type-only re-export runtime-lens filter
node tests/test-unused-deps-producer.mjs                 # unused-deps.json producer: review-only dependency hygiene, package script tools, and audit artifact visibility
node tests/test-update-test-doc.mjs                      # tests/README.md generator drift guard
node tests/test-vocab.mjs                                # locks _lib/vocab.mjs constant values + forwarder (v1.10.1)
node tests/test-wildcard.mjs                             # exports wildcard subpath
node tests/test-workspace-no-exports.mjs                 # FP-38: workspace packages without `exports` field
```

## Fixtures

Tests build their own fixtures under `/tmp/fx-*` on each run.
Fixtures are disposable — every suite clears its own working dirs
at start. No shared state between runs.

Each test script exits non-zero on any failure. `npm test` stops
at the first failing suite.

## What the tests cover by release

- **v1.9.11**: **FP-38: workspace packages without `exports` field.** User's empirical v1.9.10 confirmation on duyet/monorepo brought the FP rate from 73.2% down to 10.9%. The report identified that the remaining 13 of 229 Tier C findings were all workspace imports of packages that use `main` + direct subpath resolution instead of the modern `exports` map — a common pattern in Bun, older pnpm, and Turborepo monorepos.
- **v1.9.10**: **True AST Config Pass.** User found that v1.9.7-v1.9.9 produced byte-identical results to v1.9.3 on duyet/monorepo despite three releases claiming FP-36 was fixed. Investigation revealed the residual was in our hand-rolled `extends` resolution, not the JSONC parser as the user's hypothesis suggested — but the honest answer to "is full AST transition the right call?" is yes, and further than jsonc-parser. This release replaces the entire tsconfig loading path with TypeScript's own compiler API.
- **v1.9.9**: **Product UX Pass.** Reviewer's P1-c item (`audit-repo.mjs` one-shot orchestrator) closed, plus the "blindZones should be in artifacts, not just prose" recommendation that accompanied it. Two new surfaces:
- **v1.9.8**: **Evidence Honesty Patch.** Reviewer caught a doc-vs-reality drift that's especially pointed for a tool whose core claim is "evidence before claims": `SKILL.md:57` listed `compare-repos.mjs` as step 8 of the Workflow, but the file didn't exist on disk. Closed that gap plus three related "label your evidence honestly" items the reviewer grouped with it.
- **v1.9.7**: **FP-36 emergency patch.** Critical resolver bug discovered on duyet/monorepo (2026-04): 218 of 397 Tier C dead-export findings were actually consumed via per-app `tsconfig.json` `paths` aliases that the resolver did not read. 73.2% FP rate from one blind spot.
- **v1.9.6**: Review-driven completion of the v1.9.5 ranking layer. Reviewer confirmed 1.9.5's direction but caught five integration gaps between the claim and the actual pipeline. All five closed.
- **v1.9.5**: **Ranking layer.** First release of the architectural track proposed by external review: shift from "candidate detector" toward "fix proposal engine." Sixth of the reviewer's 13-point plan is the first to land.
- **v1.9.4**: Five small text fixes. No code changes.
- **v1.9.3**: Self-consistency patch. Reviewer accepted v1.9.2's correctness and claim-scoping fixes but found two CHANGELOG-internal contradictions that themselves undermined the "honesty patch" framing. Plus one cleanup that removes dead code the previous comment already flagged.
- **v1.9.2**: Small honesty patch on top of 1.9.1. Reviewer accepted the count-leak fix but caught two correctness issues and three small cleanups. All five closed.
- **v1.9.1**: Honest patch. v1.9.0 claimed "drift is mechanically impossible to ship." That was over-strong. Reviewer proved it by setting `### Tests (999 total)` in CHANGELOG and watching the false count propagate into the README through `update-test-doc.mjs`, with `check:test-doc` and `test-update-test-doc.mjs` T7 both passing.
- **v1.9.0**: Structural release. Stops the write-then-apologize cycle that ran through 1.8.2 → 1.8.5 by making `tests/README.md` a generated artifact with a CI gate against drift.
- **v1.8.5**: Review-driven precision patch, third in the review cycle. Reviewer confirmed the v1.8.4 changes landed correctly, then filed four precision issues — two release-blockers, two UX improvements. All four closed here.
- **v1.8.4**: Review-driven patch. Reviewer dogfooded v1.8.3 and confirmed the six substantive items from the prior review were closed — then filed three precision issues on the work itself. All three closed here.
- **v1.8.3**: Review-driven consolidation. An external reviewer dogfooded v1.8.2 and filed six issues, each specific and reproducible. All six closed here.
- **v1.8.2**: Performance + hardening release. Closes feedback item #5 (silent catches + perf). Three deliberate improvements plus one incidental bug found along the way.
- **v1.8.1**: Self-dogfood patch. Ran the full pipeline against the tool's own source and uncovered two things: one real dead export in our own code, and one legitimate tool bug that the existing tests didn't catch.
- **v1.8.0**: Language matrix release. Closes feedback item #4. Surfaced via dogfood on a pure-JSX fixture which pre-1.8.0 scanned as 0 files / 0 defs / 0 dead.
- **v1.7.2**: Python convention release. Surfaced by dogfooding v1.7.1 against a real Python monorepo (ouroboros) and comparing the output against a manual audit. Three separate bugs inflated the dead-export list from 35 real candidates to 166. Each is fixed in a targeted way with no effect on JS/TS behavior.
- **v1.7.1**: Patch release. Fixes a symlink-aliasing bug that caused falsely-dead symbol reports in repos using vendored symlinks or dir-symlink workspace layouts.
- **v1.7.0**: Structural release. `classify-dead-exports.mjs` (517 LOC — the second feedback hotspot: accumulating framework exceptions, text-regex occurrence counting, aliased-export special case all in one file) splits into three layers: **fact extraction**, **policy rules**, **orchestration**. Public behavior is unchanged — `dead-classify.json` keeps the same schema, all 104 test assertions pass.
- **v1.6.0**: Coverage release. Closes feedback item #2 — five scripts that previously had zero automated coverage now have smoke tests. Total assertions: 86 → 104 across 9 suites.
- **v1.5.0**: Structural release. `_lib/resolver.mjs` (737 LOC — the regression hotspot that caused the 1.3.0 → 1.3.1 fix cycle) is split into seven focused submodules. Public API is unchanged: every existing `import { … } from './_lib/resolver.mjs'` continues to work via a re-export facade. All 86 test assertions pass unchanged.
- **v1.4.0**: Infrastructure release. No behavior changes to audit logic; adds the tooling layer that prevents the class of drift we hit between 1.2.0 and 1.3.1. Version/doc drift, silent catches, unused imports, and missing CI gates were each called out in review.
- **v1.3.1**: Patch release. Fixes a regression introduced in the 1.3.0 merge where the resolver's relative-path extension probe was narrower than the parallel patch it was merged with.
- **v1.3.0**: Merge release combining the v1.2.0 fixes with improvements from a parallel patch submitted against the same 1.1.0 baseline. Net gain over either lone version: 9 additional test assertions and five strict-improvement items.
- **v1.2.0**: Correctness fixes across seven areas. All changes come with regression tests (see `tests/` folder). 68/68 assertions pass after these fixes.

## What's NOT covered

Documented honestly so future maintainers know where the guard
rails stop:

- Cross-process cache sharing between scripts (each of
  `measure-topology`, `build-call-graph`, `check-barrel-discipline`
  currently re-parses from scratch). No suite exercises shared
  cache.
- Rust source trees are owned by `lumin-rust-analyzer`; this JS test suite
  covers routing, manifest, and blind-zone behavior only, not a JS
  Rust parser fallback.
- `__getattr__`-based lazy export maps in Python `__init__.py`
  files. Known residual FP source; no fixture.
- Interactive `--focus-class` output beyond the smoke check that
  the block appears.
