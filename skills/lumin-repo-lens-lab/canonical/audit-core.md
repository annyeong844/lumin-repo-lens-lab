# canonical/audit-core.md

> **Role:** canonical owner map for Rust audit orchestration and manifest evidence migration.
> **Owner:** this file.

## Scope

`lumin-audit-core` owns typed audit artifact registry, manifest evidence
summary contracts, manifest metadata projection, initial manifest root-shell
projection, combined initial manifest root plus evidence-read assembly,
manifest evidence refresh patch projection, final manifest summary
patch projection, `manifest.json.rustAnalysis` run/evidence merge projection,
current Rust-analysis artifact usability projection for produced-artifact lists,
typed full `manifest.json` evidence projection including
`manifest.json.blindZones` from already-produced output artifacts and optional
`manifest.json.rustAnalysis` run/evidence merge through the Rust-owned
rust-analysis projection,
typed `commandsRun` / `skipped` runtime-log shape, the audit orchestration plan
contract, base audit child-process executor core, typed runtime executor request
projection that builds the base plan inside audit-core before execution, typed
orchestration event ledger, producer-performance artifact construction from completed execution
observations, audit-run artifact-size measurement from Rust-owned
produced-artifact registry enumeration plus the typed Rust-analysis block,
artifact-read metric summary projection from JS-supplied read observations
plus Rust-observed manifest-evidence artifact reads, base producer phase
timing sidecar reads, lifecycle manifest patch projection,
lifecycle summary projection,
orchestration result summary projection, produced-artifact manifest patch projection,
lifecycle strict exit-policy
projection, lifecycle request hard-stop guard projection, typed pre-write engine
routing, manifest evidence refresh patch projection, human companion manifest
block projection for already-rendered Markdown companions, final closeout
patch application plus `manifest.json` file emission for an already-assembled
manifest object, combined lifecycle patch plus manifest evidence refresh
application for already-assembled manifest objects, initial manifest assembly
from JS-supplied run metadata plus Rust-owned evidence reads, final
audit-run closeout that writes `producer-performance.json` and `manifest.json`
from typed JS-supplied observations and Rust-rendered companion paths,
Rust-owned git source-commit fallback for Rust analyzer lifecycle requests,
review-only `unused-deps.json` dependency hygiene artifact construction,
`barrels.json` artifact projection from JS-produced barrel discipline facts,
`topology.json` artifact assembly from JS-produced per-file topology entries,
`framework-resource-surfaces.json` artifact construction from JS-discovered
file/package/content inputs,
`call-graph.json` artifact construction from JS-produced call graph facts,
`function-clones.json` artifact construction from JS-produced function facts,
`symbols.json` artifact construction and staged graph finalization from
JS-produced raw symbol facts plus Rust-owned source-use assembly facts,
`block-clones.json` artifact construction from JS-tokenized normalized token
streams,
`entry-surface.json` artifact projection from JS-discovered entry-surface facts,
`export-action-safety.json` artifact projection from JS-produced edit-safety
findings,
`module-reachability.json` artifact construction from already-produced
`symbols.json` and `entry-surface.json` graph facts, identity-level
any-contamination owner-map projection from JS-produced type-escape facts, and
`resolver-capabilities.json` / `resolver-diagnostics.json` artifact
construction from already-produced `symbols.json` unresolved resolver facts, and
`checklist-facts.json` artifact construction from JS-collected AST facts plus
already-produced audit artifacts, and
`staleness.json` temporal evidence construction from already-produced
`symbols.json.deadProdList` and git history, and
`lumin-repo-lens-lab.sarif` projection from already-produced audit artifacts, and
the migrated Rust pre-write / canon-draft / check-canon / post-write
lifecycle child-process wrappers that are not source-language analysis, and
the Rust-owned `execute-audit-lifecycle` sequencing wrapper that applies the
checked lifecycle request guard, pre/post/canon/check execution order, and
strict post-write exit policy before manifest refresh.

`triage-repo.mjs` owns lint-tool adapters and lowers supported configuration
surfaces into normalized `triage.json.boundaries[]` plus
`triage.json.lintEnforcement`. Checklist C5 consumes only that normalized
evidence. If a declared lint command is unsupported or a config cannot be
parsed, `lintEnforcement.status` is `degraded`; without an independently
grounded boundary rule, Rust must project C5 as `unknown`, never as grounded
absence. `rustfmt` is formatting evidence, not lint-enforcement evidence.

It does not own JS/TS producer behavior, Rust source-health syntax analysis,
Cargo semantic oracle behavior outside the explicitly migrated canon-draft
lifecycle child-process wrapper, check-canon lifecycle child-process wrapper,
post-write lifecycle child-process wrapper, Rust pre-write lifecycle wrapper,
live artifact-read observation outside the migrated manifest-evidence
summary/refresh artifact reads and Rust-owned companion closeout reads,
lifecycle phase timing reads, JS/TS blind-zone producer semantics, lifecycle
request construction from CLI flags/files, or final `manifest.json` assembly
before the final write.

## Remaining JS-Owned Manifest Boundaries

These fields are deliberately not Rust-owned yet. They need lane-specific parity
or orchestration ownership before migration.

| Manifest area                                                                                                        | Current owner                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Why it stays JS-owned for now                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Next migration gate                                                                                                                                                                 |
| -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Blind-zone source artifacts and JS reference helper                                                                  | JS-owned producer artifacts, with `_engine/lib/blind-zones.mjs` retained only as the legacy parity/reference surface                                                                                                                                                                                                                                                                                                                                                                              | Final `manifest.json.blindZones` projection is Rust-owned through `manifest_evidence.rs` composing `blind_zones.rs`. The already-produced source artifacts still carry TS/JS producer semantics from `triage.json`, `symbols.json`, `dead-classify.json`, `entry-surface.json`, resolver diagnostics, and current-run Rust analysis availability; Rust consumes those artifacts without taking over their producers. `_engine/lib/blind-zones.mjs` is not the product projection owner.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Retire `_engine/lib/blind-zones.mjs` after the remaining legacy reference tests and summary formatting no longer need it as the JS reference surface.                                      |
| Producer runtime observation values (`commandsRun`, `skipped`)                                                       | Base audit profile: `orchestration_executor.rs` through the `execute-base-runtime` wrapper; compatibility path: `execute-base-plan`; lifecycle request guard, pre-write routing, Rust/JS pre-write lifecycle, canon-draft, check-canon, post-write, and strict exit sequencing: `cli/lifecycle.rs` through `execute-audit-lifecycle`; lifecycle manifest patch: `lifecycle.rs` through the `manifest-lifecycle-update` wrapper; remaining lifecycle request construction: `audit-repo.mjs` | Rust now owns base plan construction from runtime flags, base-step skip decisions, statuses, stderr snippets, wall-clock measurements, orchestrator memory snapshots, request-level pre/post-write hard-stops, pre-write intent file/stdin reads after those hard-stops clear, typed pre-write engine routing from requested engine plus intent language, Rust pre-write analyzer child execution plus advisory projection, canon-draft child execution aggregation, check-canon child execution aggregation, post-write child execution plus delta-summary projection, strict post-write exit policy application, and final placement of already-built raw lifecycle blocks plus `manifest.json.lifecycle` summary projection. JS builds only thin typed request objects from CLI flags/files, passes the pre-write intent flag/path without reading its body, invokes Rust CLIs, and owns JS/TS producer semantics.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Migrate remaining lifecycle request construction only when CLI parsing/input ownership moves into audit-core; keep JS/TS producer internals outside audit-core.                     |
| Producer performance measurement inputs                                                                              | Base audit profile: `orchestration_events.rs` from typed runtime observations; manifest-evidence artifact reads: `cli/manifest.rs`; artifact-read summary math: `artifact_read_metrics.rs`; lifecycle helpers and remaining JSON artifact read observation: `audit-repo.mjs`                                                                                                                                                                                                               | Rust owns base child status/wall/stderr/memory observations, base producer phase sidecar reads, manifest-evidence summary/refresh artifact read observations for the already-produced files it reads, artifact-read metric summary projection from JS-supplied and Rust-supplied observations, audit-run context projection for `scanRange`/`cache`/generated-artifact mode, companion input artifact reads inside `finalize-audit-run-with-companions`, produced-artifact enumeration from the output directory plus typed `rustAnalysis`, artifact-size measurement for that Rust-owned list, final `producer-performance.json` construction, and final audit-run closeout/write through `finalize-audit-run` / `finalize-audit-run-with-companions`. JS still observes ordinary JSON artifact reads outside the migrated manifest-evidence and companion closeout wrappers, passes runtime observations, and owns JS/TS pre-write lifecycle observations.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | Move remaining lifecycle observations and other live artifact-read observation only with explicit Rust owners.                                                                      |
| Raw lifecycle blocks (`preWrite`, `postWrite`, `canonDraft`, `checkCanon`)                                           | Request-level lifecycle guard, pre-write intent body read, pre-write route selection, lifecycle execution, ordering, and strict exit policy: `cli/lifecycle.rs` through `execute-audit-lifecycle`; individual focused wrappers remain available; lifecycle manifest patch: `lifecycle.rs` through the `manifest-lifecycle-update` wrapper                                                                                                                    | Rust owns the checked request-level hard-stop projections for mutually exclusive `--pre-write`/`--post-write` and `--pre-write` without `--intent`, including the raw skipped blocks, exit code 2, and stderr text. Rust reads the pre-write intent only after those hard-stops clear and owns typed engine routing. Rust-engine pre-write may invoke `lumin-rust-analyzer`; JS/TS pre-write runs natively in audit-core from current-worktree OXC evidence, lookup/cue policy, advisory writes, and Markdown rendering without a Node child or stale artifact fallback. Both routes record `executionOwner: "lumin-audit-core"` and require exact current-run artifact contracts. Native post-write computes current evidence and deltas in-process. `canonDraft` and `checkCanon` preserve their checked lifecycle contracts. | Do not restore legacy `pre-write.mjs`/`post-write.mjs` execution, no-fresh reuse, or a JS fallback classifier. |
| Lifecycle strict exit policy                                                                                         | `lifecycle_exit_policy.rs` through the `lifecycle-exit-policy` wrapper and `execute-audit-lifecycle`                                                                                                                                                                                                                                                                                                                                                                                       | Rust owns the typed projection from the current orchestrator exit code plus raw post-write lifecycle block to strict post-write exit-code/stderr decisions. `execute-audit-lifecycle` applies it after pre/post/canon/check sequencing.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | Move additional lifecycle exit policies here only after their raw-block owner is typed.                                                                                             |
| Human companion artifact rendering (`audit-summary.latest.md`, `audit-review-pack.latest.md`, `topology.mermaid.md`) | `topology.mermaid.md`: `topology_mermaid.rs` through the `topology-mermaid-render` wrapper or `finalize-audit-run-with-companions`; `audit-review-pack.latest.md`: `audit_review_pack.rs` through the `audit-review-pack-render` wrapper or `finalize-audit-run-with-companions`; `audit-summary.latest.md`: `audit_summary.rs` through the `audit-summary-render` wrapper or `finalize-audit-run-with-companions`                                                                         | Rust owns the checked Markdown projection for `topology.mermaid.md` from already-produced `topology.json`, owns the checked Markdown projection for `audit-review-pack.latest.md` from already-produced audit artifacts, owns the checked Markdown projection and console-preview extraction for `audit-summary.latest.md`, owns companion input artifact reads for final audit closeout telemetry, and owns the checked companion request policy. A base-backed run may render all profile-appropriate companions; a pre-write/canon lifecycle-only run keeps its lifecycle summary but receives no reused base artifact inputs; a post-write-only run suppresses that summary because its delta is the primary evidence. JS passes the base-pipeline plan result and already-assembled manifest; it does not decide final companion classes. | Move remaining companion request inputs only when CLI parsing/input ownership moves into audit-core.                                                                                |
| Final `manifest.json` assembly                                                                                       | Initial root/evidence projection: `manifest_root.rs` through `manifest-root-with-evidence`; lifecycle sequencing: `cli/lifecycle.rs` through `execute-audit-lifecycle`; final audit-run closeout/write: `cli/manifest.rs` through `finalize-audit-run`; final companion render plus closeout/write: `cli/manifest.rs` through `finalize-audit-run-with-companions`; remaining request construction: `audit-repo.mjs`                                                                       | Rust audit-core now owns the first manifest object from JS-supplied run metadata, typed `commandsRun`/`skipped`, lifecycle sequencing and final lifecycle exit code, Rust-owned evidence reads with artifact-read telemetry, final companion request policy plus render sequencing from JS-supplied closeout context and manifest lifecycle blocks, final `producer-performance.json` construction, closeout patch application, and final `manifest.json` file emission. JS still constructs typed request inputs from CLI flags/files and passes observations/request flags into audit-core.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Migrate remaining request construction only when CLI parsing/input ownership moves into audit-core.                                                                                 |
| Lifecycle execution                                                                                                  | `preWrite` routing: `pre_write_routing.rs`; Rust-engine pre-write: `pre_write_lifecycle.rs`; native JS/TS pre-write: `pre_write_lifecycle/js_native.rs`; `canonDraft`: `canon_draft_lifecycle.rs`; `checkCanon`: `check_canon_lifecycle.rs`; native post-write: `post_write_lifecycle.rs`; combined sequencing: `cli/lifecycle.rs` through `execute-audit-lifecycle`                                                                 | Rust audit-core owns base audit profile execution, pre-write routing, native JS/TS write-gate execution, lifecycle sequencing, and final closeout coordination. Only the Rust-analyzer, canon-draft, and check-canon routes remain legitimate child-process boundaries. | Do not add a second JS/TS write-gate owner or compatibility child.                                                                                                      |

### Pre-write dependency ownership

JS/TS dependency intents may carry `ownerFile` (or the accepted `file` and
`targetFile` aliases). Dependency lookup resolves the nearest owning
`package.json`. Without a dependency-specific owner, it may infer an owner only
when every planned intent file resolves to one manifest. Multiple workspace
owners are `DEPENDENCY_OWNER_AMBIGUOUS`, not `NEW_PACKAGE`. A lockfile entry
alone never proves a direct dependency declaration because it may be
transitive.

## Canonical Rust Modules

| File                                                                           | Owns                                                                                                                                                                                                                                                                                                                                                                | Must not own                                                                                                                                                                                                                                                       |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs`              | Known artifact names, dynamic artifact filename matching, deterministic produced-artifact enumeration, and current Rust-analysis artifact usability from `manifest.json.rustAnalysis` blocks                                                                                                                                                                        | child process execution, JSON artifact parsing beyond artifact usability fields                                                                                                                                                                                    |
| `experiments/rust-main/lumin-audit-core/src/artifact_measurement.rs`           | Producer-performance artifact-size measurement for JS-supplied produced artifact names, including best-effort missing/non-file exclusion and largest-artifact projection                                                                                                                                                                                            | produced-artifact discovery, artifact JSON parsing, artifact-read timing, producer execution                                                                                                                                                                       |
| `experiments/rust-main/lumin-audit-core/src/artifact_read_metrics.rs`          | `artifact-read-metrics.v1` summary projection from JS-supplied artifact-read observations, Rust-supplied manifest-evidence artifact-read observations, and phase sidecar read observations: totals, parse failures, path naming, largest-read projection, slowest-parse projection, and per-artifact aggregation                                                    | live artifact-read observation, JSON artifact parsing, producer execution, final manifest file writing                                                                                                                                                             |
| `experiments/rust-main/lumin-audit-core/src/artifact_summaries.rs`             | `manifest.json.frameworkResourceSurfaces`, `manifest.json.unusedDependencies`, and `manifest.json.blockClones` projections from already-produced artifact JSON                                                                                                                                                                                                      | framework/resource scanning, unused-dependency analysis, block-clone detection                                                                                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/audit_review_pack.rs`              | Stable `audit-review-pack.latest.md` rendering facade: request validation, top-level companion section ordering, merge instructions, and focused owner sequencing through `audit-review-pack-render` | artifact-specific review-cue calculation, lane internals, audit artifact production, source walking, final manifest file writing, audit-summary console preview |
| `experiments/rust-main/lumin-audit-core/src/audit_review_pack/protocol.rs`     | Project-owned audit-review-pack request/result protocol shapes and schema constants | Markdown rendering, review-cue calculation, artifact reads, or file writing |
| `experiments/rust-main/lumin-audit-core/src/audit_review_pack/lanes.rs`        | Controller-lane construction and deterministic artifact/check ordering for topology, type/shape, dead/public-surface, and failure/blind-zone review | artifact-specific cue policy, request/result protocol, audit artifact production, or final file writing |
| `experiments/rust-main/lumin-audit-core/src/audit_review_pack/review_checks.rs` | Single owner of artifact-specific review-check projection from already-produced audit artifacts, including any-contamination, resolver, dependency, framework/resource, SFC, and unreachable-SCC cues | lane ordering, protocol shapes, audit artifact production, source walking, or final file writing |
| `experiments/rust-main/lumin-audit-core/src/audit_review_pack/support.rs`      | Shared JSON value conversion, count formatting, scan/Rust scope text, pluralization, and controller-lane Markdown framing used by the audit-review-pack renderer | artifact-specific review policy, lane selection/order, protocol shapes, or artifact production |
| `experiments/rust-main/lumin-audit-core/src/audit_summary.rs`                  | Stable `audit-summary.latest.md` rendering facade: request validation, top-level Markdown section ordering, read-first guidance, guardrails, and focused owner sequencing through `audit-summary-render` | protocol internals, lifecycle projection, measured-cue policy, artifact-map policy, console projection, audit artifact production, source walking, or final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/protocol.rs`         | Project-owned audit-summary request/result protocol shapes and schema constants | Markdown rendering, cue calculation, artifact reads, or file writing |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/support.rs`          | Shared JSON value conversion, count/percentage/plural formatting, unresolved-reason formatting, lifecycle-only base-evidence detection, and scan-range/confidence scope text | section ordering, artifact-specific cue policy, protocol shapes, or artifact production |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/lifecycle.rs`        | Pre-write, post-write, canon-draft, and check-canon command-result Markdown projection from the assembled manifest lifecycle blocks | lifecycle execution, section ordering, measured cues, console projection, or artifact production |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/sections.rs`         | Artifact-map, living-audit tracking, and expansion-hint section projection from already-produced companion inputs | measured-cue policy, lifecycle command projection, console extraction, or artifact production |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/console.rs`          | Blind-zone console summary, bounded console-line extraction, and final audit-summary console preview formatting | Markdown companion section ordering, measured cues, lifecycle projection, or artifact production |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/measured_cues.rs`    | Deterministic measured-cue selection/order, lifecycle-only base-evidence suppression, direct topology/function-size/catch/shape/function-clone/fix-plan/call-graph/blind-zone cues, and the no-cues fallback | artifact-specific helper internals, Markdown section ordering, lifecycle command projection, artifact maps, console projection, artifact production, or source walking |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/measured_cues/artifact_cues.rs` | Artifact-specific cue formatting for framework/resource, dependency, Rust-analysis, SFC, unreachable-SCC, generated-consumer, and type-escape evidence | cue ordering, resolver-specific cues, any-contamination owner projection, section orchestration, or artifact production |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/measured_cues/any_contamination.rs` | Identity-level any-contamination owner summarization and measured-cue formatting from `symbols.json` owner maps | other measured cues, cue ordering, section orchestration, or symbol artifact production |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/measured_cues/resolver_cues.rs` | Resolver root/scope, blocked-candidate example, and reason/family distribution cue formatting | resolver analysis, cue ordering, non-resolver cues, section orchestration, or artifact production |
| `experiments/rust-main/lumin-audit-core/src/barrel_discipline.rs`              | `barrels.json` artifact projection from JS-produced barrel discipline facts: request schema validation, single-package skip shape, monorepo `summary`/`byPackage` placement, and checked legacy `meta` shape preservation                                                                                                                                           | repo-mode detection, alias-map construction, source walking, JS/TS import parsing, root-barrel/subpath classification, and producer orchestration                                                                                                                  |
| `_engine/lib/block-clone-artifact.mjs`                                                | JS/TS OXC parsing, normalized block-clone token production, and request-side policy identifiers/defaults | suffix-array/LCP construction, repeated-region grouping, containment pruning, noise/cap policy, final artifact projection, or a JS fallback artifact owner |
| `experiments/rust-main/lumin-audit-core/src/block_clones.rs`                   | Stable `block-clones.json` construction facade from JS-tokenized normalized token streams: request validation, file-evidence partitioning, and focused owner sequencing | source walking, incremental snapshot/cache ownership, JS/TS parsing, tokenization/normalization, clone policy internals, or producer timing |
| `experiments/rust-main/lumin-audit-core/src/block_clones/protocol.rs`          | Project-owned block-clone request, tokenized-file, and token protocol shapes plus request schema constant | threshold policy, repeated-region detection, noise classification, or artifact projection |
| `experiments/rust-main/lumin-audit-core/src/block_clones/policy.rs`            | Checked block-clone artifact/policy/normalization/threshold/noise identifiers, threshold defaults and request normalization, and deterministic threshold policy projection | suffix-array construction, group extraction, noise classification, source parsing, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/block_clones/suffix_array.rs`      | Token-value compression with file sentinels, production SA-IS suffix-array construction, and Kasai LCP construction over already-tokenized inputs; prefix-doubling exists only under `#[cfg(test)]` as a differential oracle | clone thresholds, span materialization, noise policy, cap policy, JSON projection, or a production comparison-sort suffix-array path |
| `experiments/rust-main/lumin-audit-core/src/block_clones/groups.rs`            | Repeated-region span materialization, overlap filtering, group hashing, deterministic ranking, and contained-group pruning | tokenization, threshold invention, test-mirror noise classification, cap/status policy, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/block_clones/noise.rs`             | Checked same-file/test-scaffold/node-Vitest mirror classification, candidate/review/muted caps, visibility placement, and muted-reason counts | repeated-region detection, threshold defaults, source walking, or final artifact metadata |
| `experiments/rust-main/lumin-audit-core/src/block_clones/projection.rs`        | Deterministic final artifact, summary, noise-policy, group, instance, scan-range, and incremental metadata projection from focused owner results | request validation, token compression, group extraction, noise classification, or new policy |
| `experiments/rust-main/lumin-audit-core/src/block_clones/tests.rs`             | Focused artifact behavior fixtures for review groups, noise classification, independent caps, legacy total-cap projection, containment, SA-IS differential coverage, and schema rejection | production behavior or alternate policy implementations |
| `experiments/rust-main/lumin-audit-core/src/call_graph.rs`                     | `call-graph.json` artifact construction from JS-produced call graph facts: request schema validation, parse-warning/meta/support projection, fan-in map projection, `topCallees` projection, bounded member-call counters, module-call aggregation, prototype-owner aggregation, semi-dead list placement, and deterministic summary projection                     | source walking, OXC parsing, import/export/member-call extraction, resolver behavior, exported object/member matching, semi-dead import classification, prototype-call detection, producer phase timing                                                            |
| `_engine/lib/function-clone-artifact.mjs`                                             | JS/TS OXC parsing, top-level exported/file-local function discovery, normalized function-fact extraction, and per-file read/parse diagnostic payloads | exact/structure/signature grouping, near-candidate retrieval or scoring, threshold metadata, final artifact projection, source walking, incremental cache orchestration, or a JS fallback artifact owner |
| `experiments/rust-main/lumin-audit-core/src/function_clones.rs`                | Stable `function-clones.json` construction facade from JS-produced function facts: request validation, owner sequencing, and final artifact assembly from focused function-clone modules | source walking, JS/TS parsing, fact decoding internals, grouping/scoring internals, threshold invention, or producer timing |
| `experiments/rust-main/lumin-audit-core/src/function_clones/protocol.rs`       | Project-owned function-clone request shape and request schema constant | fact interpretation, grouping, scoring, sorting, or artifact projection |
| `experiments/rust-main/lumin-audit-core/src/function_clones/facts.rs`          | Checked JS fact decoding into the internal normalized function-fact representation, observed-at stamping, field coercion, and deterministic fact comparison | clone policy, grouping, pair scoring, or final artifact metadata |
| `experiments/rust-main/lumin-audit-core/src/function_clones/groups.rs`         | Exact-body, structure, and signature group construction from normalized facts, including shared-token/member projection | near-candidate pair scoring, threshold metadata, diagnostic sorting, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/function_clones/near.rs`           | JS/TS near-function bounded-retrieval orchestration: grouped-identity exclusion, retained-token postings, compatibility partitioning, deterministic pair generation, streaming top-N projection, candidate-generation diagnostics, and checked policy metadata | source parsing, exact/structure/signature grouping, pair-local score calculation, token normalization internals, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/function_clones/near/model.rs`     | Internal near-fact, compatibility-key, candidate-evidence, projection, and candidate-generation diagnostic structs | policy thresholds, token extraction, scoring algorithms, JSON artifact assembly, or public protocol types |
| `experiments/rust-main/lumin-audit-core/src/function_clones/near/tokens.rs`    | Significant call-token filtering, deterministic exported-name tokenization, and full-vs-retained token evidence projection | repository-local IDF calculation, pair generation, score thresholds, or artifact summary counts |
| `experiments/rust-main/lumin-audit-core/src/function_clones/near/scoring.rs`   | Repository-local call-token IDF, sorted token overlap, saturated shared-IDF score, range similarity, and score formatting | pair generation, policy metadata, candidate ordering, or JSON artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/function_clones/near/candidate.rs` | Pair-local compatibility checks, near-score evidence, candidate ordering comparison, and deterministic candidate JSON projection from checked evidence | postings construction, global pair dedupe, projection limits, policy metadata, or source parsing |
| `experiments/rust-main/lumin-audit-core/src/function_clones/projection.rs`     | Deterministic diagnostic and clone-group sorting, generated/review-visible counts, line/member helpers, and final projection utilities shared by the facade owners | fact decoding, pair scoring, threshold policy, or source discovery |
| `experiments/rust-main/lumin-audit-core/src/function_clones/tests.rs`          | Focused artifact behavior fixtures for exact/structure/signature grouping, near-candidate scoring, incomplete evidence, and schema rejection | production behavior or alternate policy implementations |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract.rs`                  | Stable JS/TS extraction facade and per-file orchestration: strict request validation, local Rayon pool ownership, semantic collector dispatch, and final file-result assembly                                                                                                                                                | JSON protocol shape, parser retry policy, top-level definition/import/re-export semantics, CJS policy, dynamic-import opacity policy, named-import precision, type-escape classification, shape/function-signature/inline-pattern normalization policy, pre-write local-operation classification, class-method projection, source walking, alias/tsconfig/package resolution, or graph projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/protocol.rs`         | Project-owned JS/TS extraction request, response, file-result, definition/use, opacity, CJS surface, re-export, class-method, type-escape, and Vue global-component registration JSON shapes                                                                                                                               | parser traversal, extraction policy, source IO, relative resolution, or graph projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/parser_support.rs`   | OXC source-type selection, checked JS-to-JSX retry, parse diagnostic projection, and byte-offset-to-line mapping                                                                                                                                                 | definition/use classification, source walking, resolver policy, or artifact projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/definitions.rs`      | Top-level declaration indexing, exported identity ranges, exported definition projection, alias-export identity binding, and deterministic definition IDs                                                                                                                               | import/re-export evidence, class/member surfaces, type-escape matching, resolver policy, or graph projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/module_uses.rs`      | Static import, side-effect import, namespace import, named/default import, re-export evidence, and relative-resolution annotation over already-collected source paths                                                                                                                    | dynamic import, CommonJS, named-import member precision, source discovery, alias/tsconfig/package resolution, or graph projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/ast_support.rs`      | Shared OXC AST identifier, member-object, assignment-target, property-key, visibility, declaration-kind, module-name, and definition-ID normalization used by extractor-owned semantic modules                                                                 | extraction policy, candidate classification, source walking, or artifact projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/code_shape.rs`       | Parser-free code-shape whitespace normalization and shape-type punctuation compaction shared by type-escape and shape-hash extraction                                                                                                                                                                                                                              | AST traversal, evidence classification, hashing, or artifact projection                                                                                                                                                                                             |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/cjs.rs`              | CommonJS export-surface, literal/dynamic `require`, namespace-member, side-effect-only, and opacity evidence                                                                                                                                                                                                                                                        | ESM import extraction, package resolution, graph fan-in/dead projection, or JS fallback policy                                                                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/dynamic_imports.rs`  | Literal/nonliteral dynamic-import evidence, member precision, template-prefix opacity, and `import.meta.glob` call facts discovered from the parsed program                                                                                                                                                                                                         | glob expansion, source inventory construction, resolver policy, or graph mutation                                                                                                                                                                                  |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/vue_global_components.rs` | Checked JS-compatible Vue app receiver discovery and review-only global `.component(...)` registration facts from the already-parsed OXC program, including import binding provenance, async-factory opacity, duplicate registration muting, component-name normalization, and deterministic source order | Repository walking, source IO, SFC file parsing, resolver policy, graph fan-in, safe-fix eligibility, or JS fallback classification |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/function_signature.rs` | Explicit JS/TS function-signature fact extraction and intent-literal normalization from the already-parsed OXC program, checked JS-compatible generic-parameter renaming, type-text normalization, display projection, and SHA-256 identity                                                                                                                          | function body similarity, semantic equivalence, clone grouping, source walking, parser orchestration, lookup cue policy, or fallback execution                                                                                                                      |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/inline_patterns.rs` | Checked JS-compatible catch-block inline-pattern occurrence extraction from the already-parsed OXC program, including bounded statement-shape normalization, occurrence locations, enclosing function labels, and SHA-256 identity                                                                                                                               | repository-wide grouping thresholds, semantic equivalence, extraction safety, cue tiering, source walking, parser orchestration, or fallback execution                                                                                                             |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/named_imports.rs`    | Named-import namespace-member and escape precision with lexical shadow tracking                                                                                                                                                                                                                                                                                     | base static-import facts, resolution, fan-in projection, or dead-export classification                                                                                                                                                                             |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/shape_hash.rs`       | Exported interface/object-alias/literal-union shape fact extraction from the already-parsed OXC program, checked JS-compatible normalization and SHA-256 identity, unsupported-shape diagnostics, declaration-merge refusal, alias-export handling, and generated-file evidence                                                                                       | repository walking, parse orchestration, shape-index grouping/meta projection, function-signature shapes, cue language, or fallback execution                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/type_escape.rs`      | AST/comment type-escape detection, exported-owner association, and occurrence identity using the shared code-shape normalizer                                                                                                                                                                                                                                       | downstream contamination ranking, source walking, or fallback selection                                                                                                                                                                                            |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/surfaces.rs`         | Pre-write local-operation and class-method surface projection from an already-parsed program                                                                                                                                                                                                                                                                        | top-level definition extraction, dead-export ranking, or edit-safety policy                                                                                                                                                                                        |
| `experiments/rust-main/lumin-audit-core/src/relative_source_resolver.rs`       | The single inventory-bounded relative JS/TS source-target matcher shared by Rust JS/TS extraction and source-use assembly: slash/path-segment normalization, exact/file/index candidate ordering, compiled-JS-to-TS source fallback ordering, and root-relative/absolute inventory aliases                                                                          | source walking, filesystem probing outside caller-supplied inventories, alias/tsconfig/package resolution, unresolved policy, or graph projection                                                                                                                  |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly.rs`            | Stable source-use assembly facade: public protocol and build-entrypoint re-exports only                                                                                                                                                                                                                                                                             | request decoding, graph projection, resolver policy, filesystem access, or duplicate helper implementations                                                                                                                                                        |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/protocol.rs`   | Project-owned source-use request/response JSON shapes and schema constants, including resolved/external/non-source/generated terminal record identities used by linked projections                                                                                                                                                                                  | compact transport decoding, resolution, graph projection, or filesystem access                                                                                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/input.rs`      | Strict request transport normalization: path/string table lookup, compact row decoding, synthetic record IDs, compact type-only state, and normalized internal records                                                                                                                                                                                              | graph policy, source resolution, glob expansion, namespace traversal, or artifact projection                                                                                                                                                                       |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/path.rs`       | Shared source-use path text projection: normalized root-relative paths, basename extraction, lexical root containment, and relative scope text                                                                                                                                                                                                                      | filesystem probing, source walking, symlink interpretation, or resolver policy                                                                                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/glob.rs`       | Literal `import.meta.glob` pattern validation and deterministic expansion only against the caller-supplied source inventory and cap                                                                                                                                                                                                                                 | source-text discovery, repository walking, cap selection, alias resolution, or graph mutation                                                                                                                                                                      |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/namespace.rs`  | Namespace/named re-export map derivation from normalized current-request re-export records, merge of explicit standalone-call facts, and deterministic re-export-chain resolution                                                                                                                                                                                   | import extraction, repository walking, alias/tsconfig/package resolution, graph mutation, or diagnostic projection                                                                                                                                                 |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/assembly.rs`   | Stable source-use assembly facade: request schema validation, normalized input preparation, standalone/embedded build-mode selection, build-state initialization, and deterministic record iteration | resolver-stage branch policy, graph/evidence projection internals, request transport decoding, source walking, parsing, arbitrary filesystem probing, or producer timing |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/assembly/record.rs` | Per-record resolver-stage dispatch, supported-stage validation, projection-only target routing, relative target selection, and delegation to terminal/glob/generated/internal projection owners | response initialization, compact transport decoding, graph mutation details owned by delegated branches, source walking, or parsing |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/assembly/internal.rs` | Resolved internal and namespace re-export graph projection: edge construction, direct/broad consumer evidence, namespace diagnostics, out-of-band counters, and resolved target transport | target resolution, external/unresolved/generated/glob projection, namespace map derivation, or response initialization |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/assembly/terminal.rs` | External dependency, non-source asset, unresolved-internal, and missing-relative terminal projection, including dependency consumers, unresolved prefix/specifier evidence, and preserved terminal record identities | relative resolution, internal edge construction, glob/generated projection, or source discovery |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/assembly/glob_record.rs` | Graph and unresolved-evidence projection from the checked `glob.rs` expansion result, including namespace-user deduplication and glob branch counters | glob pattern validation/expansion policy, source inventory ownership, alias resolution, or cap selection |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/assembly/generated.rs` | Generated-virtual surface deduplication, export matching, resolved generated consumer projection, and unresolved generated evidence | generated-surface discovery, source parsing, target resolution, or general internal-edge projection |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/assembly/support.rs` | Per-build assembly state plus shared handled/skipped transport, branch/counter mutation, projection-source predicates, source-extension checks, and canonical edge-kind lowering used by branch owners | branch dispatch, artifact policy unique to one branch, request decoding, source walking, or parsing |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph.rs`                   | `symbols.json` orchestration from a prepared strict v2 request: source-use assembly execution, SFC/fan-in/dead/any-contamination cross-projection sequencing, deterministic summary, and final artifact assembly | request validation/path-table decoding, ancillary evidence projection policy, wire-shape declarations, SFC policy, fan-in/dead algorithms, any-contamination policy, v1 compatibility, legacy precomputed result acceptance, source walking, parsing, resolver policy, incremental cache ownership, producer timing, or cached artifact mutation |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/prepare.rs`           | strict v2 request validation and preparation: schema/context checks, object-shape validation, source-use/SFC input validation, root agreement, compact path-table decoding, and project-owned prepared file/definition records | source-use execution, evidence classification/projection, fan-in/dead/SFC/any-contamination policy, final artifact assembly, source walking, parsing, resolver policy, or compatibility fallback |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/evidence.rs`          | Thin ancillary-evidence facade that declares focused projection owners and re-exports the stable symbol-graph evidence API | Evidence algorithms, request validation, source-use resolution, SFC policy, fan-in/dead/any-contamination algorithms, final artifact assembly, source walking, parsing, or compatibility fallback |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/evidence/ordering.rs` | Deterministic ordering and key projection for dependency/internal/SFC/generated/unresolved evidence plus generated-virtual surface export ordering | Evidence classification, path normalization, source-use resolution, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/evidence/paths.rs`    | Symbol-evidence path text projection: normalized root-relative paths, lexical segment normalization, absolute-like path recognition, and dynamic-import prefix target projection | Filesystem probing, source walking, symlink interpretation, resolver policy, or scan-scope classification |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/evidence/generated_blind_zones.rs` | Generated-consumer blind-zone projection from unresolved source-use records, including candidate/package scope derivation, current scan-scope qualification, deduplication, and review-only evidence shape | Generated artifact discovery, source-use resolution, scan-scope policy ownership, fan-in/dead ranking, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/evidence/unresolved.rs` | Deterministic unresolved-specifier top examples and per-reason count/space/stage/hint summaries from prepared unresolved records | Resolver execution, alias policy, source walking, absence claims, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/evidence/surfaces.rs` | Dynamic-import opacity, CJS export/require surfaces, and parse-error path projection from prepared extraction facts | JS/TS parsing, dynamic-import or CJS classification, source-use resolution, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/evidence/indexes.rs`  | Deterministic class-method, pre-write local-operation, and file-level re-export index projection from prepared extraction facts | Definition extraction, local-operation classification, fan-in/dead policy, pre-write cue language, or final artifact assembly |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/protocol.rs`          | Strict `lumin-symbol-graph-producer-request.v2` project-owned wire types grouped into required `context`, `extraction`, `sourceUseAssembly`, and `graph` sections, including compact path IDs and typed SFC/fan-in/dead inputs                                                                                                                                      | artifact projection, compatibility adapters, defaulting missing evidence to empty values, filesystem probing, or third-party parser types                                                                                                                          |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/sfc.rs`               | Typed SFC style/template/global/generated-manifest/framework-convention projection, source-use target linkage, and status/reason selection from explicit source-use terminal outcomes                                                                                                                                                                               | SFC discovery, source walking, source-use resolution, fan-in eligibility, or safe-fix policy                                                                                                                                                                       |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/reachability.rs`      | Source-use fan-in merge, identity/space fan-in projection, top fan-in ranking, and dead-candidate partitioning                                                                                                                                                                                                                                                      | parsing, resolution, final artifact assembly, or compatibility fallback                                                                                                                                                                                            |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/any_contamination.rs` | Type-escape-to-owner association, contamination labels/measurements, owner indexes, and annotated definition projection                                                                                                                                                                                                                                             | type-escape extraction, edit-safety policy, or final artifact assembly                                                                                                                                                                                             |
| `experiments/rust-main/lumin-audit-core/src/js_ts_pre_write.rs`               | Thin JS/TS pre-write evidence facade: public request/response constants, request type re-exports, and prepare-then-project orchestration | Request validation, source discovery, cache storage, parser traversal, evidence classification, or JSON projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_pre_write/protocol.rs`      | Project-owned JS/TS pre-write evidence request shapes, optional cross-host transport descriptor, and schema constants | Source discovery, filesystem access, helper selection/execution, cache policy, extraction, or evidence projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_pre_write/input.rs`         | Fail-closed request validation and input preparation: artifact/dependency/file ordering contracts, canonical explicit-source containment, checked JS/TS source discovery, source-inventory/path-map construction, cache-backed extraction invocation, and exact extracted-row scope/cardinality checks | OXC parser traversal, cache storage format, fan-in classification, absence claims, or final JSON projection |
| `experiments/rust-main/lumin-audit-core/src/js_ts_pre_write/projection.rs`    | Deterministic compact evidence projection from prepared current-run extraction rows: definitions, class/local-operation indexes, exact/broad/type fan-in, dependency consumers, unresolved relative imports, topology, parse failures, any-contamination annotations, type-escape inventory, and embedded `shapeIndex` construction from the same parse pass | Request validation, source discovery, filesystem access, parser traversal, cache persistence, shape normalization/hash policy, cue language, or JS fallback execution |
| `experiments/rust-main/lumin-audit-core/src/js_ts_pre_write/cache.rs`         | Strict per-file OXC fact cache for the shared JS/TS pre-write/post-write evidence pass: repository and source-set identity, exact current-worktree byte SHA-256 identity, parsing from the same bytes used for identity, cache load/store validation, and incremental observations | source discovery, parser traversal, final evidence reuse, absence claims, cue/ranking policy, Git index/blob identity, artifact-path aliases as source identity, transformed repository bytes in place of worktree bytes, or stat/mtime-only identity                                                         |
| `experiments/rust-main/lumin-audit-core/src/js_ts_pre_write/single_flight.rs` | Root-scoped cross-process admission for the JS/TS evidence pass using a host-local OS file lock, automatic crash/exit release through file-handle lifetime, and artifact-visible wait/held phase telemetry | source discovery, cache/result reuse, request coalescing, elapsed-time caps, stale-lock age guesses, cross-host coordination, parser policy, evidence projection, or JS fallback execution |

`build-symbol-graph.mjs` requires Rust JS/TS extraction for every JS-family file
in its changed set. An audit-core command failure, malformed batch response, or
missing per-file result is a producer hard-stop. It must not import or invoke
`_engine/lib/extract-ts.mjs` as a fallback. Rust-reported per-file parse errors remain
artifact-visible parse failures; they are not helper-contract failures.

`generate-canon-draft.mjs` and `check-canon.mjs` use the same Rust JS/TS fact
owner for fresh helper-registry and naming inventories. They must collect the
scoped JS-family file set once, invoke `js-ts-extract-artifact` in bounded
batches, and expose the resulting project-owned `defs` and `uses` rows through
the existing synchronous canon collector callback. `check-canon --source all`
must share one current-run extraction index across helper and naming checks.
An audit-core command failure, malformed response, duplicate/missing result, or
lookup outside that scoped index is a hard-stop; there is no JS parser fallback.
Rust per-file parse failures remain per-file canon diagnostics and must not be
converted to empty definitions or uses. `_engine/lib/extract-ts.mjs` is a compatibility
adapter only and must not load OXC, parse source text, or own extraction policy.

`_engine/lib/symbol-graph-discovery.mjs` owns source snapshot construction,
incremental extraction-cache classification, Rust JS/TS batch invocation,
Python/Go extraction input adaptation, and normalized per-file fact assembly.
It returns discovery facts and cache observations to `build-symbol-graph.mjs`.
It must not resolve import specifiers, mutate graph fan-in, classify dead
exports, construct source-use graph evidence, or project `symbols.json`.

`experiments/rust-main/lumin-audit-core/src/sfc_file_facts/` owns
deterministic per-file SFC fact extraction from caller-supplied source text:
Vue/Svelte script blocks and Astro frontmatter, static script imports,
relative `<script src>`, relative style `url()`/`@import` references, and
template component bindings grounded in those script imports. The same parse
pass also owns review-only per-file convention facts grounded in explicit
bindings: Vue `defineOptions({ components })` and Options API component
registrations, Astro `client:*` directives, Svelte `use:` actions, and Svelte
`$store` subscriptions. `sfc_file_facts/script.rs` owns script AST facts,
binding scopes, and raw convention candidates; `sfc_file_facts/template.rs`
owns template tag/expression projection; `sfc_file_facts/conventions.rs` owns
binding-grounded convention row construction and SFC component-name
normalization; `sfc_file_facts/protocol.rs` owns the strict project-owned wire
shapes. These lanes must preserve the checked SFC
policy: unresolved, computed, comment-only, non-function action, and non-store
bindings do not produce convention evidence, and emitted convention rows stay
muted review evidence with no graph fan-in or safe-fix eligibility. The strict
`lumin-sfc-file-facts-request.v1` request carries the complete scoped SFC file
set and exact source text; Rust must not walk the repository, resolve a
specifier, or infer repository-level framework conventions. A malformed
request, parser failure, duplicate/missing per-file result, or malformed result
is a producer failure. JS must not retry the file with an alternate SFC
classifier or convert the failure into empty facts.

`_engine/lib/symbol-graph-sfc-discovery.mjs` owns SFC framework-signal detection,
source loading for the already-scoped SFC file list, one batched invocation of
the Rust SFC file-fact owner, cached Vue global-registration row collection
from the existing JS/TS extraction `fileData`, and orchestration of the
remaining checked repository-level SFC collectors. It returns raw
script-import, script-src, style-asset, template-ref, global-registration,
generated-manifest, and framework-convention facts plus collection telemetry.
It must not parse JS/TS or SFC file contents, resolve those facts, assign graph
status/reason fields, construct source-use records, or project SFC artifact
rows. JS may collect repository-level Nuxt and unplugin
configuration/filesystem convention facts; it must not reparse source files or
recreate the Rust-owned Vue/Astro/Svelte per-file convention classifiers. A
Rust JS/TS parse failure remains an artifact-visible parse failure; JS must not
retry Vue global registration extraction or substitute empty classification.
Deterministic
resolution-linked projections remain Rust-owned by `symbol_graph/sfc.rs`. The
JS convention owner excludes Nuxt `#components` imports from the ordinary
script-consumer lane because it materializes them as framework-convention
evidence instead.

`_engine/lib/symbol-graph-sfc-inputs.mjs` is the raw-fact transport boundary between
SFC discovery and the strict Rust symbol-graph request. It may copy checked
collector fields and attach deterministic source-use record IDs. It must not
resolve specifiers, infer terminal status, synthesize resolved files, or replace
Rust-owned SFC reason selection.

`_engine/lib/symbol-graph-resolver.mjs` owns the remaining JS-side resolution boundary:
Python module resolution, Go module resolution, alias/package resolution,
external fast-path checks, unresolved diagnostic enrichment, and resolver
telemetry. It returns terminal resolver facts only. It must not construct
source-use records, mutate graph state, compute fan-in/dead candidates, or
project `symbols.json`.

`source_use_assembly/glob.rs` may expand an `import.meta.glob` record only after the
JS/TS extractor has already discovered a literal glob expression and supplied
the scan inventory plus cap metadata in the request. Rust does not scan source
text for glob calls, choose the cap policy, or construct the source-file
inventory for this lane.

For embedded symbol-graph assembly, normalized `reExportNamespace` and
`reExport` records are the re-export-map source of truth. Rust resolves their
relative or pre-resolved targets before processing namespace-member/escape
records, then derives the namespace/named maps once for the same request. The JS
producer must not call standalone source-use assembly to pre-resolve those maps
or send a second map projection in `sourceUseAssembly`.

`_engine/lib/source-use-assembly-request.mjs` is the JS transport boundary for this
request. It may compact paths, enum strings, names, specifiers, record IDs, and
rows, and may remap linked projection record IDs. It must not resolve imports,
derive re-export maps, classify source uses, or project graph evidence.

`_engine/lib/source-use-record-builder.mjs` owns conversion from already-discovered
use facts and already-observed resolver outcomes into strict source-use request
records. It may normalize transport fields and preserve terminal resolver
evidence. It must not walk files, resolve a specifier, mutate graph state, or
interpret the final Rust assembly response.

`_engine/lib/symbol-graph-source-use-planner.mjs` owns deterministic partitioning of
raw source-use facts into inventory-relative Rust resolution records and facts
that require an existing JS language/alias resolver. It also plans MDX/SFC
projection-only records. It must hard-stop rejected record conversions and must
not execute Rust, project graph evidence, or keep an alternate JS graph path.

`_engine/lib/symbol-graph-finalizer.mjs` owns the transport-only finalizer boundary:
request serialization, audit-core result-file execution, final artifact cache
lookup/store, and strict validation of the bounded artifact summary returned by
Rust. It must not alter graph inputs, recompute artifact projections, recover
from an invalid Rust result, or accept a legacy artifact shape.

`_engine/lib/symbol-graph-request.mjs` owns strict v2 request lowering: extraction
fact projection, parse-error inventory, shared path-table compaction, and path
ID remapping across source-use and graph inputs. It must not discover files,
resolve imports, infer graph outcomes, or accept missing required input groups.

### Symbol Graph Strict V2 Contract

`symbol-graph-artifact` accepts only
`lumin-symbol-graph-producer-request.v2`. The top-level request and every grouped
section reject unknown fields. `context`, `extraction`, `sourceUseAssembly`, and
`graph` are required; their evidence arrays must be present even when empty.
Missing evidence is not equivalent to zero evidence.

The v2 request contains inputs, never final `symbols.json` projections. The
following v1 result/precomputation families are forbidden: aggregate source-use
counters/maps/rows, precomputed SFC result arrays/counts, dead lists, fan-in
maps/lists, precomputed any-contamination maps, generated-consumer blind-zone
outputs, and cache-entry maps. JS-owned resolver outcomes must enter the single
`sourceUseAssembly` record stream once. Rust derives all counters, edges,
consumers, unresolved summaries, dead lists, fan-in maps, contamination facts,
and SFC outputs from the required typed sections.

There is no JS artifact-construction fallback. The retired
`_engine/lib/symbol-graph-artifact.mjs` builder must not be restored; compatibility
callers must invoke `symbol-graph-artifact` and accept its fail-closed contract.

Parse failures are supplied as explicit compact path IDs. Rust derives both the
parse warning count and `filesWithParseErrors` from that one list. Embedded
source-use assembly must consume every supplied record; any skipped record is a
hard contract failure, not an omitted row or an empty-success artifact.
JS-owned Python, Go, alias, package, and framework resolution may produce a
typed terminal source-use record before the Rust call. A malformed use or a
failed record conversion is not a resolver fallback lane and must hard-stop the
producer.

Dead-export projection is identity-level for every included source file,
including files reached from tests or entry setup. Rust emits production
candidates as `symbols.json.deadProdList` and included test-like candidates as
`symbols.json.deadTestList`; it must not reduce the test lane to a count-only
summary. `classify-dead-exports.mjs` consumes the production list first and,
when `includeTests` is true, appends the test list. Candidate-limit ordering
therefore preserves production work while still making test-only dead exports
review-visible. Its performance block records separate production and test
input counts.

The strict v2 cutover is atomic. There is no v1 adapter or dual-schema window.
The current runtime bridge contract version and required feature set are owned
by `_engine/lib/audit-core.mjs` and the Rust runtime-contract command. The strict
contract requires `symbolGraphStrictRequestV2`,
`symbolGraphDeadTestCandidates`,
`stalenessBatchPickaxe`,
`sourceUseAssemblyDerivedReExportMaps`, and
`sourceUseAssemblyTerminalRecordOutcomes`; an
older helper or a helper missing either feature is rejected before execution.
The contract probe must prove namespace-member fan-in through a re-export record
without supplying an explicit re-export map. Every advertised platform binary
and the packaged source fallback must be regenerated from the same contract.

| `experiments/rust-main/lumin-audit-core/src/dead_classify.rs` | `dead-classify.json` artifact construction from JS-produced dead-export candidate facts: request schema validation, occurrence-count C/A/B classification, aliased export-specifier bucket projection, excluded/unprocessed candidate materialization, provenance field forwarding, summary/performance placement, and deterministic artifact writing through `dead-classify-artifact` | JS/TS source walking, OXC AST reference counting, regex fallback counting, package/repo-mode discovery, public-surface and framework policy fact collection, resolver/provenance fact computation, file-cache management, and classify producer timing |
| `experiments/rust-main/lumin-audit-core/src/blind_zones.rs` | Typed `manifest.json.blindZones` projection from JS-owned producer artifacts, exposed through fixture/case parity mode and output-dir manifest wiring mode | JS/TS producer behavior, console summary rendering, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/canon_draft_lifecycle.rs` | `manifest.canonDraft` raw lifecycle block execution for `--canon-draft`: source selection, unknown-source failure, `generate-canon-draft.mjs` child spawning, per-source exit projection, fallback draft path projection, and advisory exit code result | canon draft source-specific content generation, markdown proposal rendering, check-canon drift reading, pre/post-write lifecycle execution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/check_canon_lifecycle.rs` | `manifest.checkCanon` raw lifecycle block execution for `--check-canon`: source selection, unknown-source failure, all/per-source `check-canon.mjs` child spawning, `canon-drift.json` per-source projection, logical per-source exit projection, and advisory/strict exit code result | canon drift detection, canonical parser semantics, drift report rendering, pre/post-write lifecycle execution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/checklist_facts.rs` | `checklist-facts.json` facade: request validation, stable section ordering, incremental observation pass-through, metadata projection, and final artifact assembly from typed request facts and already-produced artifacts | request JSON shape, section-specific gate math, topology/shape analysis, citation wording, deferred-item vocabulary, source walking, per-file cache selection, OXC parsing, or Markdown rendering |
| `experiments/rust-main/lumin-audit-core/src/checklist_facts/protocol.rs` | Project-owned checklist-facts request, optional input-artifact, function-size, silent-catch, and incremental-observation JSON shapes plus checked default vocabulary | gate computation, artifact reads, source extraction, cache selection, or rendering |
| `experiments/rust-main/lumin-audit-core/src/checklist_facts/function_size.rs` | `A2_function_size` role buckets, thresholds, stable ranking, capped examples, and role-local projections from already-collected AST facts | function discovery, OXC parsing, citation wording, or Markdown rendering |
| `experiments/rust-main/lumin-audit-core/src/checklist_facts/topology.rs` | `A5_decoupling_ratio` and `A6_circular_deps` projection, checked layered-edge treatment, full/top-list normalization, cycle summaries, and deterministic topology examples | topology production, path discovery, citation wording, or verdicts beyond checklist gates |
| `experiments/rust-main/lumin-audit-core/src/checklist_facts/shape_drift.rs` | `B1B2_shape_drift` exact-group and near-shape review-cue projection, field/name overlap scoring, generated-only separation, and deterministic candidate ordering | shape-fact production, semantic-equivalence claims, automatic refactors, citation wording, or Markdown rendering |
| `experiments/rust-main/lumin-audit-core/src/checklist_facts/projections.rs` | Artifact-backed duplicate-implementation, dead-code, normalized lint-enforcement projection with fail-closed unknown handling for degraded adapter evidence, barrel-amplification, and silent-catch checklist projections | lint-tool/config discovery, source extraction, topology/shape policy, citation wording, or Markdown rendering |
| `experiments/rust-main/lumin-audit-core/src/checklist_facts/presentation.rs` | Checklist result annotation, citation hints, unavailable projection, deferred-item vocabulary, and compact citation value formatting | section gate computation, artifact production, source extraction, or Markdown rendering |
| `experiments/rust-main/lumin-audit-core/src/checklist_facts/value_support.rs` | Shared checked JSON traversal, numeric coercion, generated-group filtering, deterministic string collection, and rounding helpers used by checklist projections | checklist policy, citation prose, artifact IO, or rendering |
| `experiments/rust-main/lumin-audit-core/src/compare_repos.rs` | `compare.json` artifact construction from two already-produced audit output directories: tolerant optional artifact reads, side summary projection, numeric delta projection, missing-artifact accounting, and deterministic artifact shape while preserving the checked `compare-repos.mjs` vocabulary | audit pipeline execution, source walking, artifact production, human console rendering |
| `experiments/rust-main/lumin-audit-core/src/discipline.rs` | `discipline.json` artifact construction from JS-supplied scan-scope file inventory: regex discipline counters for TS/JS, Python, Go, unreadable-file accounting, per-file offender projection, rates, and deterministic summary fields while preserving the checked `measure-discipline.mjs` vocabulary | source walking, include/exclude/test policy, parser-based fact extraction, source-language semantic classification beyond regex counters |
| `experiments/rust-main/lumin-audit-core/src/shape_index.rs` | `shape-index.json` artifact construction from supplied shape-hash facts: request schema validation, fact/diagnostic/error sorting, `groupsByHash` projection, generated-file fact counting, incremental metadata pass-through, and checked `meta.supports` placement | source walking, incremental snapshot/cache ownership, JS/TS parsing, shape-hash fact extraction, generated-file detection, and normalized hash policy |
| `experiments/rust-main/lumin-audit-core/src/pre_write_routing.rs` | Typed pre-write engine routing from requested engine, intent flag, and already-read intent JSON text: `auto`/`js`/`rust` selection, intent language validation, explicit mismatch hard-stops, normalized intent projection, and removal of route-only `language` before Rust analyzer stdin | intent file/stdin reading, JS/TS evidence or advisory semantics, Rust analyzer internals, child process execution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle.rs` | Thin `manifest.preWrite` lifecycle facade: public capture/streaming entrypoints and engine-specific delegation | Request/response shape definitions, child process mechanics, advisory artifact IO, engine-specific validation or result projection |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/protocol.rs` | Project-owned pre-write lifecycle request/result shapes, failure vocabulary, analyzer invocation projection, and Rust native artifact input shape | Child execution, artifact IO, advisory construction, engine routing |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/child.rs` | Shared pre-write child process execution contract: captured or inherited stdout/stderr, stdin forwarding, spawn/wait failure projection, and engine-specific argv construction | Advisory validation, producer semantics, lifecycle result projection |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/advisory.rs` | Shared current-run advisory artifact paths and IO: latest/specific path construction, exact JSON matching, atomic advisory writes, stale/invalid file removal, and required invocation-id validation | Rust native artifact semantics, JS/TS evidence semantics, child execution, lifecycle result projection |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/rust_engine.rs` | Rust engine lifecycle execution: source-commit fallback, stale output invalidation, exact native schema/policy/producer/coverage validation, native latest copy, advisory construction, JS-supplied inventory/failure pass-through, capability projection, and typed success/failure blocks | Rust analyzer internals, JS/TS producer semantics, generic child process mechanics |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_engine.rs` | Narrow JS/TS engine dispatch into the single native lifecycle owner | advisory semantics, JS/TS parsing, generic child process mechanics, or compatibility execution |
| `experiments/rust-main/lumin-audit-core/src/post_write_lifecycle.rs` | Thin native `manifest.postWrite` lifecycle facade for `--post-write`: request validation, current-run cleanup, native after-snapshot orchestration, delta artifact writing, lifecycle result projection, and failure projection without spawning Node | pre-write advisory construction, JS fallback execution, base-audit orchestration, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/post_write_lifecycle/protocol.rs` | Project-owned post-write request/result, advisory, inventory, delta, file-delta, status, summary, and failure shapes plus schema/vocabulary constants | repository walking, classification, rendering, artifact IO |
| `experiments/rust-main/lumin-audit-core/src/post_write_lifecycle/delta.rs` | Deterministic type-escape capability/scan parity, planned matching, baseline classification, duplicate diagnostics, required-acknowledgement projection, and summary semantics ported from the checked JS producer | filesystem IO, clock/random ID generation, Markdown rendering, file discovery |
| `experiments/rust-main/lumin-audit-core/src/post_write_lifecycle/file_delta.rs` | Deterministic repo-relative planned/before/after file-set classification and summary projection | source discovery, type-escape classification, policy judgement about whether an unexpected file is wrong |
| `experiments/rust-main/lumin-audit-core/src/post_write_lifecycle/render.rs` | Deterministic post-write Markdown projection from the typed delta artifact | classification, artifact IO, terminal policy |
| `experiments/rust-main/lumin-audit-core/src/pre_write_intent.rs` | JS/TS pre-write intent validation and normalization, including checked missing-key warnings, structured declarations, exact shape inputs, refactor-source paths, and planned type-escape vocabulary | repository evidence collection, lookup classification, advisory rendering |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native.rs` | Native JS/TS pre-write lifecycle orchestration: current-run compact evidence collection, artifact writes, lookup/cue composition, advisory writes, and lifecycle result projection without a Node child | OXC extraction policy, lookup classification details, cue policy details, Markdown wording, or fallback execution |
| `experiments/rust-main/lumin-audit-core/src/js_ts_pre_write/host_transport.rs` | Optional exact-contract Windows-host execution of the shared JS/TS evidence pass for x64 WSL mounted worktrees, result-file transport cleanup, host-response validation, and restoration of caller-local root/cache paths before lifecycle artifact writes | lifecycle routing, intent parsing, lookup/cue policy, advisory/delta construction, JS fallback execution, or selection of an unvalidated helper |
| `experiments/rust-main/lumin-audit-core/src/runtime_contract.rs` | Shared runtime-contract schema/version and host-transport feature constants consumed by both the CLI contract reporter and cross-host evidence validator | subcommand lists, result-output lists, helper discovery, fixture construction, or process execution |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/mod.rs` | JS/TS pre-write lookup coordination, stable intent-lane ordering, shared projection helpers, and drift projection | lane-specific lookup policy, source discovery, parser traversal, advisory IO, cue tiering, or renderer wording |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/name/mod.rs` | Exact/canonical name lookup coordination and stable name-result projection | evidence confidence details, candidate search scoring, sibling-operation policy, file topology, dependency declarations, cue tiering |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/name/evidence.rs` | Exact-name fan-in, identity-space, contamination, and resolver-confidence projection | candidate search, operation policy, cue tiering |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/name/search.rs` | Candidate indexing, near-name and semantic search hints, shared name tokenization, locality, and deterministic search ordering | exact identity claims, operation-family promotion, cue tiering |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/name/policy.rs` | Local and service sibling-operation promotion/muting from already-projected candidates | candidate discovery, exact identity claims, cue tiering |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/file.rs` | File existence, topology/domain-neighbor, and test-like path projection | name search, dependency declarations, source discovery, parser traversal |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/dependency.rs` | Package declaration and observed dependency-consumer projection | package installation, source parsing, dependency policy outside pre-write evidence |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/shape.rs` | Exact shape-hash and normalized function-signature projection | fuzzy structural inference, source parsing, cue tiering |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup/inline.rs` | Focused inline-pattern matching against current-run extracted evidence | source extraction, pattern discovery, cue tiering |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/cues.rs` | Deterministic JS/TS pre-write cue cards, muted cues, and unavailable-evidence projection from lookup results | lookup discovery, source parsing, artifact IO, or Markdown rendering |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/render.rs` | Deterministic JS/TS pre-write Markdown projection from the completed advisory plus the constant-shape result-file terminal handoff containing the invocation path and complete evidence counts | evidence collection, lookup classification, cue tiering, artifact IO, or per-candidate terminal streaming on the production result-file route |
| `experiments/rust-main/lumin-audit-core/src/living_audit.rs` | `manifest.json.livingAudit` projection from known living-audit document candidate paths under the audited root | audit document authoring, final answer policy, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/manifest_core.rs` | `manifest.json.scanRange`, `manifest.json.confidence`, and `manifest.json.sfcEvidence` projections from already-produced `triage.json` and `symbols.json` | blind-zone detection, living-audit document discovery, producer execution |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics.rs` | `manifest.json.resolverDiagnostics` projection from already-produced `symbols.json`, `resolver-capabilities.json`, and `resolver-diagnostics.json` | module resolution, blocked-hint production, blind-zone detection |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts.rs` | Resolver-diagnostics request validation and stable assembly of the capability and diagnostics artifacts from focused submodule projections | JS/TS module resolution, source parsing, symbol graph production, generated artifact discovery, filesystem scan-scope checks, resolver relevance matching against findings, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts/protocol.rs` | Resolver-diagnostics request shape and the internal read-only view over already-produced unresolved resolver records | Resolver policy, artifact projection, source discovery |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts/capabilities.rs` | Static `resolver-capabilities.json` condition-profile and resolver-family capability matrix projection | Runtime resolution, capability discovery, diagnostics aggregation |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts/classification.rs` | Existing resolver-family taxonomy, candidate/scope derivation, specifier-root parsing, and resolver/generated blind-zone relevance policy projection | Import resolution, filesystem probing, candidate matching against findings |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts/rows.rs` | Unresolved import, unsupported import, and diagnostic-only candidate-target row projection with stable ordering | Blind-zone projection, summary aggregation, graph-edge creation |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts/blind_zones.rs` | Resolver/generated blind-zone rows and candidate-relevant blocked-hint projection with stable deduplication | Resolver-family classification, summary aggregation, finding relevance matching |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts/summary.rs` | Artifact-local reason, family, package-scope, and specifier-root summary pivots | Row construction, resolver policy, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts/value_support.rs` | Shared compact-object, deterministic sorting/deduplication, field access, and row-key helpers used only by this resolver-diagnostics owner | Resolver semantics, source discovery, public protocol ownership |
| `experiments/rust-main/lumin-audit-core/src/runtime_evidence.rs` | `runtime-evidence.json` artifact construction from already-produced `symbols.json.deadProdList` and JS-supplied Istanbul coverage JSON: runtime status classification, hit-count projection, orphan static-file summary, deterministic summary projection, and preserved checked status vocabulary | coverage generation, test execution, coverage-file discovery, JS/TS dead-export classification, source parsing, safe-fix ranking, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs` | `rust-analyzer-health.latest.json` manifest summary projection, root mismatch, invalid-shape, complete/available status, and `manifest.json.rustAnalysis` merge projection from JS-observed Rust analyzer run state plus already-produced evidence summary | Rust source parsing, source-health analysis, Cargo oracle execution, child process execution |
| `experiments/rust-main/lumin-audit-core/src/sarif.rs` | Stable `lumin-repo-lens-lab.sarif` construction facade, request validation, focused owner sequencing, and source-readable `TOOL_VERSION` ownership for package drift checks | SARIF upload, producer execution, source parsing, rule projection internals, artifact file discovery, or manifest rendering |
| `experiments/rust-main/lumin-audit-core/src/sarif/protocol.rs` | Project-owned SARIF producer request shape and request schema constant | rule catalog, severity mapping, finding projection, or source analysis |
| `experiments/rust-main/lumin-audit-core/src/sarif/dead_exports.rs` | GA001 projection precedence across fix-plan, runtime/staleness, dead-classify, and symbols evidence, including tier/severity mapping and finding property projection | fix ranking, runtime/staleness analysis, rule catalog metadata, URI construction, or final SARIF envelope |
| `experiments/rust-main/lumin-audit-core/src/sarif/secondary.rs` | GA002–GA006 result projection from topology SCC/size/hotspot, discipline offender, and barrel importer evidence in checked output order | topology/discipline/barrel analysis, GA001 policy, rule catalog metadata, or final SARIF envelope |
| `experiments/rust-main/lumin-audit-core/src/sarif/rules.rs` | Stable GA001–GA006 SARIF rule catalog, help metadata, and rule-index mapping | finding collection, severity decisions, artifact selection, URI construction, or final envelope assembly |
| `experiments/rust-main/lumin-audit-core/src/sarif/support.rs` | SARIF result state plus deterministic common result, URI, JSON field, and property helpers shared only by SARIF owners | artifact-specific classification, rule descriptions, source analysis, or final run metadata |
| `experiments/rust-main/lumin-audit-core/src/sarif/projection.rs` | Final SARIF 2.1.0 envelope, level counts, artifact-used evidence, upstream warning propagation, invocation/base URI metadata, and generated-time placement | finding collection, rule classification, source parsing, SARIF upload, or producer execution |
| `experiments/rust-main/lumin-audit-core/src/sarif/tests.rs` | Focused artifact behavior fixtures for fix-plan tiers, secondary rules, and request rejection | production behavior or alternate SARIF policy implementations |
| `experiments/rust-main/lumin-audit-core/src/staleness.rs` | `staleness.json` temporal evidence orchestration from already-produced `symbols.json.deadProdList` plus Git observations: staleness tier classification, grounding/confidence projection, incremental staleness cache hit/miss behavior, deterministic summary projection, and preserved checked artifact vocabulary | JS/TS source parsing, dead-export classification, safe-fix ranking, Git pickaxe patch parsing, git repository discovery outside the configured root, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/staleness/pickaxe.rs` | Batched textual Git pickaxe evidence for eligible dead-export symbol names: deterministic command-size chunking, one patch-history scan per chunk, commit-local added/removed occurrence counting, and latest count-changing author-time projection | Dead-export discovery, staleness tiering, binary-payload interpretation, elapsed-time caps, symbol omission, or safe-fix ranking |
| `experiments/rust-main/lumin-audit-core/src/staleness/file_history.rs` | Batched latest-file-touch collection plus bounded local-pool Git blame collection for every tracked dead-candidate file, with deterministic path chunking and current line-author-time projection | Dead-export discovery, staleness tiering, elapsed-time caps, file omission, shared mutable worker state, or global Rayon pools |
| `experiments/rust-main/lumin-audit-core/src/topology.rs` | `topology.json` artifact assembly from JS-produced per-file topology entries: `nodes`/`edges` materialization, fan-in/fan-out summaries, runtime/static SCC projection from already-resolved edges, cross-submodule aggregation from JS-supplied submodule labels, largest-file projection, summary counts, and Rust metadata placement | source walking, JS/TS/Python/Go parsing, module resolution, alias-map construction, incremental cache ownership, Rust topology sidecar comparison, repository mode discovery, submodule label discovery, and manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/topology_mermaid.rs` | `topology.mermaid.md` Markdown companion rendering from already-produced `topology.json`: capped cross-submodule Mermaid graph, capped runtime cycle graph, hub-file notes, omitted-detail limits, citation contract text, and result-file metadata through `topology-mermaid-render` | topology analysis, source walking, module resolution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/generated_artifacts.rs` | `manifest.json.generatedArtifacts` projection from already-produced `symbols.json`, generated-artifact mode validation, generated miss grouping, blind-zone grouping, and present/prepared out-of-scope evidence | package resolution, generator execution, generated-artifact producer evidence construction |
| `experiments/rust-main/lumin-audit-core/src/framework_resource_surfaces.rs` | `framework-resource-surfaces.json` artifact construction from JS-supplied discovered files, package records, and sampled content: path normalization, nearest package selection, dependency grounding, framework/resource lane classification, deterministic sorting, and artifact-local summary projection | source walking, scan/exclude/include-tests policy, workspace discovery, package manager interpretation, content sampling, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/entry_surface.rs` | `entry-surface.json` artifact projection from JS-supplied public API, package-script, HTML, framework, config, known-file, parse-error, and submodule facts: entry-file union, evidence merge, unsupported/unresolved sample ordering, meta/support projection, global/submodule completeness projection, and deterministic field projection | public/package/html/script/framework/config discovery, JS/TS source parsing, package interpretation, re-export resolution, filesystem walking, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/export_action_safety.rs` | `export-action-safety.json` artifact projection from JS-produced edit-safety findings: request schema validation, `meta` projection, `total` derivation, warning placement, and `byId` materialization from non-empty finding ids | JS/TS source parsing, local reference counting, export/declaration matching, safe action selection, edit-range proof production, and producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/module_reachability.rs` | `module-reachability.json` artifact construction from already-produced `symbols.json` and `entry-surface.json` facts: known-file collection, runtime/type graph projection from `resolvedInternalEdges`, bounded BFS with checked JS counter semantics, unreachable file projection, runtime SCC review evidence, deterministic sorting, and artifact-local summary projection | JS/TS module resolution, source parsing, symbol graph production, entry-surface discovery, package interpretation, re-export resolution beyond consumed facts, safe-delete claims, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs` | Stable `fix-plan.json` construction facade: request validation, owner sequencing, and final artifact assembly from the focused rank-fixes modules | protocol shape definitions, proposal interpretation, evidence semantics, support classification, four-tier policy, or output grouping internals |
| `experiments/rust-main/lumin-audit-core/src/rank_fixes/protocol.rs` | Project-owned `fix-plan.json` request, artifact input, public deep-import risk, and output protocol shapes plus the request schema constant | proposal flattening, evidence lookup, ranking policy, sorting, or grouping |
| `experiments/rust-main/lumin-audit-core/src/rank_fixes/findings.rs` | Proposal bucket flattening, excluded-candidate materialization, canonical finding identity/path keys, and export-action-safety evidence merge/indexing | runtime/staleness evidence interpretation, support projection, ranking policy, or final artifact grouping |
| `experiments/rust-main/lumin-audit-core/src/rank_fixes/evidence.rs` | Runtime/staleness evidence indexing, resolver summary/blindness projection, public deep-import contract evidence, HTML entry blind-zone matching, and per-finding evidence assembly | proposal flattening, support claims, four-tier decisions, or output grouping |
| `experiments/rust-main/lumin-audit-core/src/rank_fixes/support.rs` | Checked `entry-unreachable` and `call-graph-no-observed-callers` support projection from existing artifact facts, including completeness, opacity, framework-callback, fan-in, and bounded-member-call guards | artifact discovery, absence defaults, tier decisions, or safe-action proof production |
| `experiments/rust-main/lumin-audit-core/src/rank_fixes/projection.rs` | Ranked-entry shape projection, deterministic tier partition/sorting, artifact-local summary and review-reason counts, and `safeFixGroups` projection | evidence discovery, support claims, or four-tier decision policy |
| `experiments/rust-main/lumin-audit-core/src/rank_fixes/policy.rs` | The single production owner of the four-tier ranking predicate: policy exclusion, runtime contradiction, hard/soft taint handling, safe-action proof requirements, declaration-binding checks, HTML/public-contract blockers, confidence projection, and structured `blockedBy` diagnostics | finding/evidence discovery, artifact I/O, support evidence construction, sorting/grouping, JS compatibility predicates, or SARIF rendering |
| `experiments/rust-main/lumin-audit-core/src/lifecycle_exit_policy.rs` | Strict lifecycle exit-code/stderr projection from the current orchestrator exit code, strict post-write flags, and already-built raw `postWrite` block | raw lifecycle block construction, producer execution, post-write delta semantics, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/lifecycle_request.rs` | Request-level lifecycle guard projection for `--pre-write`/`--post-write` mutual exclusion and `--pre-write` without `--intent`: raw skipped block shape, stderr text, and exit-code 2 | intent file/stdin reading, pre-write engine routing, child execution, producer semantics, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/lifecycle.rs` | `manifest.json` lifecycle patch projection from completed raw `preWrite`, `postWrite`, `canonDraft`, and `checkCanon` manifest blocks, including pass-through raw block placement and `manifest.json.lifecycle` summary | lifecycle child execution, advisory generation, post-write delta production, canon draft/check producer behavior, raw lifecycle block ownership or reinterpretation |
| `experiments/rust-main/lumin-audit-core/src/manifest_companion.rs` | `manifest.json.topologyMermaid`, `manifest.json.auditSummary`, and `manifest.json.reviewPack` block shape projection from already-rendered companion artifact paths | Markdown rendering, deciding whether companion files should be written, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_evidence.rs` | Composition of Rust-owned `manifest.json` evidence fields from already-produced artifacts, including `blindZones` through `blind_zones.rs` with current-run Rust-analysis gating and optional `rustAnalysis` run/evidence merge through `rust_analysis.rs`; source summary used by the manifest evidence refresh patch | producer orchestration, manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_final.rs` | Final pre-write `manifest.json` summary patch projection for `performance`, `orchestration`, and `artifactsProduced` from already-produced `producer-performance.json`, output artifact names, and the merged Rust analysis block; standalone `manifest.json.artifactsProduced` patch projection from output artifact names and the typed Rust-analysis block; closeout patch application for already-assembled manifest objects before final write | producer execution, producer-performance artifact writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_meta.rs` | `manifest.json.meta` shape projection from JS-provided run timestamp, profile, root, and output values | clock reading, profile flag parsing before CLI dispatch, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_root.rs` | Initial `manifest.json` root shell projection, combined initial root plus evidence-read assembly, and manifest evidence refresh patch projection from the same Rust-owned manifest evidence summary shape, typed JS-observed `commandsRun` / `skipped` runtime logs, and Rust-owned produced-artifact enumeration from the output directory plus the typed `rustAnalysis` block | producer execution, source artifact semantics for blind zones, lifecycle raw block construction, human companion renderers, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/orchestration_events.rs` | Typed `lumin-audit-orchestration-ledger.v1` input contract, typed audit-run-context plus runtime-observation projection for base audit runs, base producer phase timing sidecar reads with artifact-read metric merging through `artifact_read_metrics.rs`, and `producer-performance.json` construction from completed execution observations | child process execution, lifecycle telemetry collection, live artifact-read observation, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs` | Stable public facade and runtime-contract constants for base audit execution | JS/TS producer internals, lifecycle child execution, artifact-read timing, phase timing reads, human renderers, `blindZones`, final `manifest.json` writing |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor/protocol.rs` | Typed base/runtime executor requests, plan inputs, command/skip observations, Rust analyzer execution evidence, exit policy, and memory observation protocol shapes | Plan construction, validation, child execution, artifact rendering |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor/validation.rs` | Executor request, plan-step, path, run-id, profile, owner, and base-pipeline status validation with existing hard-stop wording | Plan construction, filesystem preconditions, child execution |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor/execution.rs` | Runtime-request plan projection, base pipeline sequencing, source-inventory handoff, existing filesystem preconditions, JS/MJS argv construction, required-step stop policy, and planned-skip placement | JS/TS producer semantics, lifecycle execution, event serialization internals |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor/child_process.rs` | Child spawn/status/wall/stderr observation, stale producer-timing cleanup, and safe producer timing filenames | Pipeline ordering, Rust analyzer policy, result projection |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor/observations.rs` | `commandsRun`, `skipped`, `LedgerEvent`, and final executor exit-policy projection from already-observed runs | Child execution, precondition policy, human rendering |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor/rust_analyzer.rs` | Rust analyzer request/skip/run lane, current source-inventory Rust count, invocation projection, source-commit fallback, and analyzer artifact result projection | Rust source analysis internals, source inventory production, base pipeline ordering |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor/memory.rs` | Platform-specific orchestrator RSS snapshots and before/after delta projection around child processes | Thread policy, child sequencing, product status decisions |
| `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs` | Typed audit profile command graph, lifecycle request plan, profile/SARIF/base-pipeline skip semantics, base-step `executionOwner` metadata consumed by `orchestration_executor.rs`, and lifecycle `executionOwner` metadata consumed by the JS wrapper | child process execution, filesystem precondition evaluation, command telemetry, producer-performance measurement |
| `experiments/rust-main/lumin-audit-core/src/orchestration_result.rs` | `manifest.json.orchestration` projection from the typed `producer-performance.json` source shape, including execution status counts, required/optional failure counts, skipped counts, and capped examples | child process execution, live telemetry collection, raw `commandsRun`/`skipped` value production, producer-performance artifact writing |
| `experiments/rust-main/lumin-audit-core/src/producer_performance.rs` | `manifest.json.performance` projection from already-produced `producer-performance.json` | producer execution, memory measurement, artifact read measurement, producer-performance artifact writing |
| `experiments/rust-main/lumin-audit-core/src/source_inventory.rs` | Typed validation of the current-run `source-inventory.json` produced by `triage-repo.mjs`, including run binding, schema/root/walk/analysis-scope/path safety checks, and language counts used by orchestration | repository walking, source parsing, file-content hashing, producer-specific filtering, or reuse across audit invocations |
| `experiments/rust-main/lumin-audit-core/src/scan_scope.rs` | Audit manifest scan-scope path inclusion policy used by migrated manifest summaries, matching the JS `scanScopeStatusForPath` contract, plus deterministic repository discovery for the Rust-owned JS/TS pre-write evidence command and deferred Rust pre-write file inventory. Independent sorted directory subtrees may be walked on a local Rayon pool with explicit thread/stack policy; each worker owns its directory job and directory results are merged in input order before final sort/dedup. Root `coverage` output remains pruned, while nested authored `coverage` source modules remain in scope; nested `target` is pruned only when its parent owns `Cargo.toml`. | base-audit source walking, parsing, producer orchestration outside pre-write, global Rayon pools, shared mutable walk state, Git-only discovery, or changed scan-scope semantics |
| `experiments/rust-main/lumin-audit-core/src/unused_deps.rs` | `unused-deps.json` artifact construction from JS-supplied package records and already-produced `symbols.json`: package identity normalization, package-script tool evidence, package-scope consumer matching, review-only dependency classification, deterministic summary projection | JS/TS symbol graph production, repo-mode/package discovery, package manager execution, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/cli/mod.rs` | CLI command dispatch for audit-core commands | producer orchestration, manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/cli/args.rs` | CLI-only parsed argument structs shared by audit-core command runners | product projection logic, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/cli/io_support.rs` | CLI stdin/file JSON reads, JSON stdout/file writes, and flag value extraction | product projection logic, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/cli/artifact.rs` | CLI runners for artifact registry, artifact summaries, generated artifact summaries, resolver diagnostics summaries, Rust-analysis summaries, and blind-zone parity summaries | product projection logic beyond delegating to owned audit-core modules |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest.rs` | Thin manifest CLI facade, command re-exports, and shared stdout/result-file JSON transport | manifest request shapes, evidence reads, base-pipeline evidence policy, command argument parsing, final manifest writes, companion sequencing, or producer-performance closeout |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/protocol.rs` | Project-owned manifest CLI request/result shapes and serde defaults shared by manifest command owners | command execution, artifact reads, evidence policy, or final writes |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/base_evidence.rs` | Planned/skipped base-pipeline evidence selection, required skip-reason validation, standard `scan-gap` projection, and stale base-evidence suppression while preserving current lifecycle artifacts | artifact discovery outside manifest evidence, lifecycle execution, final writes, or new blind-zone policy |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/root.rs` | CLI runners for manifest metadata, plain root construction, root-with-evidence assembly, and lifecycle evidence refresh | evidence artifact discovery policy, focused update commands, final writes, or lifecycle child execution |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/updates.rs` | Focused manifest evidence/companion/artifact/final-summary/closeout update commands and core-summary CLI projection | root assembly, evidence artifact discovery, final writes, companion render sequencing, or producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/evidence.rs` | Manifest-evidence summary/refresh commands, output-artifact reads, tolerant optional-artifact handling, and artifact-read observation for those reads | base-pipeline skip policy, root/lifecycle assembly, final writes, or artifact-read observation outside manifest-evidence inputs |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/finalize.rs` | Thin finalization CLI facade and command re-exports | request protocol, final writes, companion rendering, lifecycle artifact inventory, or evidence policy |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/finalize/protocol.rs` | Project-owned request/result shapes for manifest write, closeout, audit-run finalization, and companion policy/plan | command execution, artifact reads, render policy, or filesystem writes |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/finalize/write.rs` | CLI execution for `manifest-write`, `manifest-closeout-write`, and `finalize-audit-run`, including producer-performance and final manifest writes | companion render sequencing, lifecycle-only artifact inventory, or evidence discovery |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/finalize/companions.rs` | `finalize-audit-run-with-companions` orchestration, companion-plan selection, artifact-read summary handoff, producer-performance construction, closeout application, and bounded result projection | individual companion rendering, lifecycle child execution, source walking, or base evidence semantics |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/finalize/render.rs` | Topology Mermaid, audit-summary, and review-pack render sequencing plus tolerant reads of their already-produced input artifacts | companion-plan policy, producer-performance construction, lifecycle artifact inventory, or final manifest writes |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/finalize/lifecycle_artifacts.rs` | Current lifecycle-only artifact inventory from manifest-bound paths/IDs and generated companion paths, constrained to existing direct children of the output directory | repository walking, base artifact discovery, render policy, or artifact generation |
| `experiments/rust-main/lumin-audit-core/src/cli/lifecycle.rs` | CLI runners for lifecycle summary, focused lifecycle wrappers, and the combined `execute-audit-lifecycle` result-file command that sequences request guard, pre-write routing, pre/post/canon/check lifecycle execution, and strict post-write exit policy while preserving child stdout/stderr streaming | JS/TS producer semantics, public CLI argument parsing, intent file/stdin reading, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/cli/orchestration.rs` | CLI runners for orchestration plan/result, base-plan execution, producer-performance artifacts, and living-audit summary | product projection logic beyond delegating to owned audit-core modules |
| `experiments/rust-main/lumin-audit-core/src/cli/usage.rs` | CLI usage text for audit-core commands | command implementation or product projection logic |
| `experiments/rust-main/lumin-audit-core/src/lib.rs` | public library exports for audit manifest wrappers | ad hoc JSON shape construction outside owned modules |

### Staleness Pickaxe Contract

Staleness name-history evidence is a textual Git-diff claim. For every unique
safe identifier that passes the checked minimum-length policy,
`staleness/pickaxe.rs` must preserve `git log -S<symbol>` count-change
semantics by summing non-overlapping added and removed occurrences per commit.
The newest commit whose net count changes owns `symbolLastMentionedAt`.
A changed line that retains the same occurrence count is not a mention.
The patch stream is matched with `aho-corasick`; overlapping matches are
retained by the automaton and then reduced to non-overlapping counts separately
for each symbol so one candidate name cannot hide another.

The owner must query eligible symbols through combined `git log -G` patch
streams instead of launching one history walk per symbol. Regex chunks are
only a platform command-transport boundary: every eligible symbol remains in
exactly one deterministic chunk, and `summary.performance` reports the actual
Git call and eligible-symbol counts. Binary patch payloads are outside this
textual evidence claim. A failed batch must hard-stop the producer; it must not
be projected as `cold` negative evidence. `meta.pickaxeMode` identifies the
algorithm, and changing its semantics invalidates the staleness cache schema.

Latest file-touch evidence follows the same no-N-history-walk rule. The file
history owner reads newest-first `git log --name-only -z` output for all
candidate paths and may stop that child once every path in the chunk has a
timestamp. Per-line blame remains one Git operation per tracked file because
Git blame has no multi-file contract; those independent operations run on a
local Rayon pool with at most four workers and a 1 MiB worker stack. This is a
process-concurrency boundary, not an analysis cap: every tracked candidate file
that still exists as a safe in-root file is blamed, and any such blame failure
hard-stops the producer. Deleted, untracked, or unsafe stale-artifact paths keep
their available file-history evidence but have no line timestamp; performance
evidence reports that unavailable-file count instead of treating it as clean
line evidence. Performance evidence also reports file counts, Git calls, and
worker count.

### Packaged Runtime Contract

The JS bridge accepts an audit-core helper only when its reported bridge
contract version and required feature set match the bridge source. Adding a
required feature is a contract change even when the CLI schema remains
backward-compatible. That change must bump the bridge contract version in the
JS resolver, Rust runtime contract, package builder, and contract tests.

`_engine/lib/audit-core-contract.mjs` owns the declarative bridge version, required
features, required subcommands, and missing-input probe table.
`_engine/lib/audit-core.mjs` remains the public JS facade and the single owner of
runtime-contract validation and the executable result-output fixture probe.
`scripts/build-skill.mjs` must call that facade before copying the
current-platform helper; it must not maintain a second subcommand list, feature
list, fixture builder, or payload matcher.

Every checked-in platform binary advertised by
`_engine/bin/audit-core-platforms.json` must be rebuilt from that same contract
before the generated skill is committed. A stale binary must not retain the
new contract version or rely on Cargo auto-build to hide the mismatch. The
packaged binary is the normal installed runtime; the Cargo workspace is a
fallback for unsupported or missing platforms.

The source-checkout bridge follows the same rule. After explicit environment
overrides and the normal installed-package path, it must contract-probe the
checked-in generated-skill binary at
`skills/lumin-repo-lens-lab/_engine/bin/<platform>-<arch>/` before inspecting or
building a Cargo target. A source checkout must not pay a one-time Rust compile
merely because its normal installed-package-relative path does not exist. Cargo
remains the fail-closed fallback when the checked-in platform helper is absent
or does not satisfy the current runtime contract. Developers who need an
unpackaged helper under active modification use the existing explicit binary
override.

Linux release binaries must be built in a controlled compatibility-baseline
environment, not directly against the maintainer's current WSL glibc. The
packaging verification must inspect the maximum required GLIBC symbol version;
the current `linux-x64` GNU baseline is GLIBC 2.31 or older.

Native Node dependencies are runtime-platform artifacts too. A WSL install
must install the skill dependencies from a supported Node/npm toolchain inside
WSL and must not reuse a Windows `node_modules` tree. Missing native bindings
are an unavailable runtime dependency, not clean parser evidence.

Native JS/TS pre-write executes as one `execute-js-pre-write` lifecycle inside
the selected current-contract audit-core. The wrapper must not restore a JS
classifier or run an independent evidence/advisory pipeline. On x64 WSL mounted
worktrees only, it may attach an exact-contract Windows evidence transport; the
selected Linux audit-core may then invoke `js-ts-pre-write-evidence` on that
helper while retaining lifecycle, advisory, delta, and artifact-write ownership.
The host response must be validated and its absolute root/cache paths restored
to caller-local WSL spelling before projection. Once host execution starts,
failure is terminal and must not fall back to the Linux evidence pass.

Current-run advisory cleanup treats both `NotFound` and `NotADirectory` as an
absent stale file. The latter is Linux's result when the configured output
parent is itself a file. Cleanup must not mask the checked advisory write
failure; the subsequent write still hard-stops as `output-write-failed`.

### Shared Source Inventory

The base audit pipeline has one current-run source inventory owner.
`triage-repo.mjs` performs the repository walk and writes
`source-inventory.json` beside `triage.json`. The inventory is immutable for
that audit invocation and records its executor-supplied `runId`, normalized
repository root, walk scope, analysis scope, supported language set, per-language
counts, and sorted repo-relative paths.

The single walk always includes test-like source paths. `analysisScope` records
whether the requested audit includes tests, while consumers narrow the complete
walk in memory. This distinction is required because production dead-export
classification still reads test files for contract-pin evidence; production
scope must not erase those files from the run inventory.

Before triage, `orchestration_executor.rs` removes any fixed-name inventory from
an earlier run. After successful triage it loads the replacement through
`source_inventory.rs` and requires the exact executor `runId`. A missing,
malformed, stale-run, root-mismatched, scope-mismatched, unsafe, duplicate, or
miscounted path is a required pipeline contract failure. It must not be
converted to an empty inventory or trigger a second repository walk. Every
later JS/MJS base producer receives the validated path and run binding through
explicit `--source-inventory <path>` and `--source-inventory-run-id <id>`
arguments. A post-triage producer or Rust analyzer cannot run before this
validation succeeds. Standalone producer invocations without those arguments
retain their own source discovery contract; the orchestrated base pipeline does
not use that compatibility path.

Consumers may narrow the inventory by language, test policy, nested root, or
additional excludes in memory. They must not broaden it beyond the current-run
scope. File contents, mtimes, and hashes remain producer-owned and are read only
for the filtered paths. Repository mutations after triage belong to the next
audit invocation rather than silently changing the file set halfway through a
run.

When the validated inventory contains one or more Rust files and zero files in
every other supported source language, the executor records the remaining
JS/MJS base producers as skipped with a Rust-only inventory reason. Requested
Rust analysis still runs. This short circuit does not apply to empty or
non-Rust resource-only repositories, and it must remain visible in orchestration
events rather than fabricating empty JS artifacts.

### Symbol Finalizer Artifact Cache

The symbol producer may reuse a previously Rust-produced `symbols.json` only as
an opaque byte artifact at the existing JS-owned incremental cache boundary.
Cache selection is storage policy, not a second symbol-graph projector: JS must
not parse, patch, summarize, or otherwise reinterpret cached artifact bytes.

A strict cache identity covers the complete compact `symbol-graph-artifact`
request except the run timestamp in `generated`. It also covers the symbol
producer/fact/parser versions and the expected audit-core runtime bridge
contract version. It also includes filesystem signatures for every current
audit-core binary candidate, without starting a helper, so replacing a binary
invalidates the cache even when its bridge contract string is unchanged. The
`incremental` request block remains part of the identity, so a cold result
cannot satisfy an unchanged warm request. The cached artifact manifest records
the request identity, byte length, and SHA-256 of the exact Rust-produced file.

Artifact and manifest filenames are request-identity-scoped. This prevents two
concurrent audits from pairing one request identity with another run's bytes.
After a complete entry is published, stale identity-scoped entries may be
removed; a cleanup race may cause a later miss, never a false hit.

On a hit, the producer verifies the manifest and artifact hash and atomically
copies the bytes without starting the Rust finalizer. The cached
`meta.generated` remains the honest time at which those artifact bytes were
created. Current-run cache hit/miss status belongs in producer phase telemetry,
not in a patched `symbols.json` field. Missing, malformed, incompatible, or
hash-mismatched cache entries are visible misses and trigger the normal Rust
finalizer; they do not create absence evidence or hard-stop the audit. Cache
reuse is disabled with `--no-incremental`, and `--clear-incremental-cache`
removes this artifact cache together with per-file facts.

## Rules

- Most audit-core modules read already-produced artifacts. `orchestration_executor.rs`
  is the explicit exception for base audit profile child execution, and
  `canon_draft_lifecycle.rs` and `check_canon_lifecycle.rs` are explicit
  exceptions for lifecycle child execution. `pre_write_lifecycle.rs` may run
  `lumin-rust-analyzer pre-write` for Rust-language intents.
  Native post-write is owned by `post_write_lifecycle/`: it invokes the Rust
  JS/TS evidence library directly, computes type-escape and file deltas, writes
  latest and invocation-specific artifacts, and renders Markdown without a
  Node child. `pre-write.mjs`, `post-write.mjs`, JS write-gate computation,
  no-fresh reuse, and any fallback classifier are forbidden production paths.
- `js_ts_pre_write.rs` is the explicit JS/TS pre-write evidence exception. It
  may discover files through the checked `scan_scope.rs` mirror or consume an
  explicit path-backed file list, parse those files with OXC, and
  project compact symbol, file-topology, fan-in, parse-error, local-operation,
  request-scoped package-import evidence, and the pre-write type-escape baseline
  from one OXC pass. The baseline uses the canonical `type-escape` vocabulary
  and remains shape-compatible with post-write's `any-inventory.json`; the
  compact symbol projection must reuse `symbol_graph/any_contamination.rs` for
  identity annotations and owner maps. Broad namespace/dynamic consumers belong
  only to `fanInByIdentitySpace.broad`, not exact `fanInByIdentity` counts. The
  native post-write path reuses the same library response's `anyInventory` and
  `files` projections for the after-snapshot and file delta input. It writes the
  current `any-inventory.json` itself and must not run `any-inventory.mjs`, load
  Node `oxc-parser`, perform a second repository walk, or reuse stale output as
  a no-fresh fallback. `any-inventory.mjs` is reference-only until deleted with
  the legacy write-gate scripts. This
  Rust owner must not write `symbols.json` or `topology.json`, broaden the
  supplied scan scope, or invent fallback evidence. Native lookup, cue, and
  rendering policy lives under `pre_write_lifecycle/js_native/`, consuming only
  the current invocation's compact evidence. A malformed request, unreadable required source, omitted response
  row, result-file failure, or audit-core contract mismatch is a hard failure
  rather than a request to run the legacy JS extractor. Parser failures remain
  artifact-visible incomplete evidence and must prevent absence claims.
- A lifecycle-only run keeps the `base-audit` / `scan-gap` entry in
  `manifest.json.blindZones` because base-audit absence and freshness claims
  are unavailable. Human summaries and terminal closeout must present that
  entry as a base-evidence scope note, not as a failure or degradation of a
  current `preWrite` or `postWrite` block. Counts labelled as current analysis
  blind zones exclude this scope-only entry; the raw manifest remains the
  authoritative complete list. `audit-summary.latest.md` must also suppress
  measured cues derived from unavailable or stale base artifacts; lifecycle
  command evidence and the base-evidence scope note remain visible.
  The command may reuse strict per-file OXC facts, but it must discover the
  current scoped file set and rebuild every compact symbol, topology,
  type-escape, inventory, and summary projection on every invocation. A cache
  hit is a parser skip, never reuse of a before/after artifact or an absence
  claim. Every scoped file is read from the current worktree and identified by
  SHA-256 of those exact bytes. A cache miss must parse the same byte buffer
  that produced the identity. Git index/blob content is not a valid substitute:
  clean/smudge filters, Git LFS, and working-tree encodings can make repository
  bytes differ from the file the user is reviewing. The identity also covers
  the normalized repository root, relative path, complete source-set fingerprint,
  extractor/cache profile version, and scan context. The source-set fingerprint invalidates cached
  resolved relative edges when files are added, deleted, or renamed.
  Missing, malformed, incompatible, or partially written cache state is an
  artifact-visible miss and triggers current-source parsing; it must not become
  empty evidence or invoke a JS extractor fallback. `--no-incremental` disables
  reuse, and `--clear-incremental-cache` removes this producer cache before the
  current pass. The default cache root is the existing repository-scoped audit
  cache root. On an x64 WSL-mounted Windows worktree, the JS bridge may attach
  an exact-contract Windows evidence transport to the native lifecycle request.
  The selected Linux audit-core may route only its shared evidence pass through
  that helper so discovery, worktree reads, and hashing use native NTFS while
  lifecycle artifact writes retain caller-local WSL paths. That transport
  optimization must not change the current-worktree-byte identity or artifact
  semantics. Host route unavailability is distinct from an executed helper
  returning null or malformed evidence: only the former may select the Linux
  evidence pass. Temporary result transport uses the lifecycle output directory
  and must be removed on success and failure. Explicit
  source requests are canonicalized at the request boundary and rejected when
  they escape the canonical root. Warm cache hits release their worktree byte
  buffers immediately after hashing and identity comparison.
  Concurrent `js-ts-pre-write-evidence` runs for the same canonical root are
  serialized by a host-local OS file lock before source discovery. The lock is
  stored below a per-user private temp root so one account cannot own or block
  another account's lock directory. It is released after compact evidence
  projection and timing capture, before stdout or result-file transport.
  It is independent of incremental reuse and covers discovery, exact worktree-byte
  reads and hashing, parser/cache work, and compact evidence projection. It has
  no elapsed-time timeout; the operating system releases it when the owning
  process or file handle exits, so correctness does not depend on lock-file age
  guesses or PID polling. WSL mounted-checkout host routing acquires the lock in
  the Windows helper's temp domain; native Linux and native Windows runs use
  their own host temp domains. Different execution hosts are not falsely
  presented as coordinated. The evidence records lock wait, held scan time,
  discovery, cache load, source read/hash, parse, cache write, and projection
  timings under the incremental/runtime observation block. Lock waiting never
  permits artifact reuse or an absence claim: every admitted invocation still
  discovers the current source set and validates exact current worktree bytes.
- Deployable audit-core platform binaries are rebuilt with Cargo's release
  profile before packaging. Debug-profile helpers remain valid for source
  development but must not be shipped as the runtime performance path. Every
  packaged platform helper, including a Windows PE selected from WSL, carries
  executable mode so Linux-hosted package installs can probe and run it.
- Audit-core may own orchestration routing separately from execution. The plan
  is declarative profile/lifecycle evidence; `lifecycle_request.rs` owns only
  request-level hard-stop blocks before intent reading or child execution;
  `pre_write_routing.rs` owns only typed pre-write engine selection from an
  already-read intent payload; Rust child execution must live in a named owner
  module with an artifact-visible owner boundary.
- Audit-core may emit bounded JSON patches to stdout for JS compatibility, but
  repository-sized manifest projections must use the result-file bridge from
  JS. `manifest-root-with-evidence`, `manifest-lifecycle-evidence-refresh`,
  `manifest-evidence-summary-with-reads`, and
  `manifest-evidence-refresh-with-reads` are result-file-only from `_lib` so a
  large audit cannot fail on Node child-process stdout buffering after Rust has
  already produced the result. Direct CLI stdout for these commands is only a
  compatibility/debug surface, not the JS wrapper contract.
- Result-file pre-write lifecycle commands follow the same transport rule for
  human-readable output. The invocation-specific advisory JSON preserves every
  cue and lookup, while terminal stdout is a constant-shape handoff with the
  advisory path and complete counts. It must not grow with candidate or lookup
  cardinality; otherwise a completed run can block before its result file is
  recovered by the caller.
- JS/TS producer lanes remain JS-owned until a lane-specific Rust parity proof
  exists.
- For `build-symbol-graph` migration, JS may continue to own file collection,
  raw SFC/MDX fact production, and compatibility
  wrapper calls. Once a graph finalization step is Rust-owned, JS must not keep
  a second implementation of that step. Staged Rust graph finalization must
  preserve the checked `build-symbol-graph.mjs` artifact shape, ordering,
  counters, source-use handling, fan-in maps, dead-candidate projection, and
  artifact-visible degraded/omitted evidence before the JS mutation path is
  removed.
- Do not add elapsed-time caps, repository-size caps, or timeout logic.
- Unknown JSON fields in consumed artifacts must be ignored.
- Missing or malformed migrated inputs must become explicit status, not silent
  zero evidence.
