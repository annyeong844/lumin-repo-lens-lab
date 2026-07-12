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
| Raw lifecycle blocks (`preWrite`, `postWrite`, `canonDraft`, `checkCanon`)                                           | Request-level lifecycle guard, pre-write intent body read, pre-write route selection, lifecycle child execution, ordering, and strict exit policy: `cli/lifecycle.rs` through `execute-audit-lifecycle`; individual compatibility wrappers remain available for focused calls; lifecycle manifest patch: `lifecycle.rs` through the `manifest-lifecycle-update` wrapper                                                                                                                    | Rust owns the checked request-level hard-stop projections for mutually exclusive `--pre-write`/`--post-write` and `--pre-write` without `--intent`, including the raw skipped blocks, exit code 2, and stderr text. Rust reads the pre-write intent file or stdin only after those hard-stops clear, then owns the typed engine route decision from requested engine plus intent JSON `language`, including explicit mismatch hard-stops and route-only `language` stripping before the Rust child. Rust pre-write records `executionOwner: "lumin-audit-core"` and preserves the checked JS helper contract for the Rust engine: analyzer argv/stdin, Rust-owned source-commit fallback (`git rev-parse HEAD`, otherwise `unknown`), current-run native artifact validation, native artifact latest copy, advisory path projection, JS-supplied file inventory and failure pass-through, rustPreWrite capability fields, child/artifact failure block projection, and product-mode streaming stdout/stderr through the Rust CLI result-file bridge. A Rust pre-write success requires the exact native schema, policy, producer, intent-lane coverage, and required evidence arrays; stale or malformed native output cannot become an available advisory. The JS/TS pre-write lifecycle wrapper records `executionOwner: "lumin-audit-core"` and preserves the checked JS helper contract for the JS/TS engine: `pre-write.mjs` argv/stdin, current-run latest/specific advisory readback, advisory path/invocation projection, child/output failure block projection, and product-mode streaming stdout/stderr through the Rust CLI result-file bridge. `pre-write.mjs` still owns JS/TS producer semantics. Post-write succeeds only when the child exits successfully and writes matching current-run latest/specific delta artifacts with the expected schema and advisory invocation binding; child failure or missing/malformed/mismatched delta evidence is a non-zero lifecycle failure, never advisory success. `canonDraft` and `checkCanon` preserve their checked lifecycle contracts. Rust now owns the lifecycle sequencing and exit-code updates that choose which blocks are present before manifest refresh; it does not reinterpret JS/TS producer semantics. | Migrate JS/TS pre-write producer semantics only after their own parity plan; do not make audit-core reinterpret JS/TS pre-write artifacts beyond the wrapper-owned lifecycle block. |
| Lifecycle strict exit policy                                                                                         | `lifecycle_exit_policy.rs` through the `lifecycle-exit-policy` wrapper and `execute-audit-lifecycle`                                                                                                                                                                                                                                                                                                                                                                                       | Rust owns the typed projection from the current orchestrator exit code plus raw post-write lifecycle block to strict post-write exit-code/stderr decisions. `execute-audit-lifecycle` applies it after pre/post/canon/check sequencing.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | Move additional lifecycle exit policies here only after their raw-block owner is typed.                                                                                             |
| Human companion artifact rendering (`audit-summary.latest.md`, `audit-review-pack.latest.md`, `topology.mermaid.md`) | `topology.mermaid.md`: `topology_mermaid.rs` through the `topology-mermaid-render` wrapper or `finalize-audit-run-with-companions`; `audit-review-pack.latest.md`: `audit_review_pack.rs` through the `audit-review-pack-render` wrapper or `finalize-audit-run-with-companions`; `audit-summary.latest.md`: `audit_summary.rs` through the `audit-summary-render` wrapper or `finalize-audit-run-with-companions`                                                                         | Rust owns the checked Markdown projection for `topology.mermaid.md` from already-produced `topology.json`, owns the checked Markdown projection for `audit-review-pack.latest.md` from already-produced audit artifacts, owns the checked Markdown projection and console-preview extraction for `audit-summary.latest.md`, owns companion input artifact reads for final audit closeout telemetry, and owns the checked companion request policy. A base-backed run may render all profile-appropriate companions; a pre-write/canon lifecycle-only run keeps its lifecycle summary but receives no reused base artifact inputs; a post-write-only run suppresses that summary because its delta is the primary evidence. JS passes the base-pipeline plan result and already-assembled manifest; it does not decide final companion classes. | Move remaining companion request inputs only when CLI parsing/input ownership moves into audit-core.                                                                                |
| Final `manifest.json` assembly                                                                                       | Initial root/evidence projection: `manifest_root.rs` through `manifest-root-with-evidence`; lifecycle sequencing: `cli/lifecycle.rs` through `execute-audit-lifecycle`; final audit-run closeout/write: `cli/manifest.rs` through `finalize-audit-run`; final companion render plus closeout/write: `cli/manifest.rs` through `finalize-audit-run-with-companions`; remaining request construction: `audit-repo.mjs`                                                                       | Rust audit-core now owns the first manifest object from JS-supplied run metadata, typed `commandsRun`/`skipped`, lifecycle sequencing and final lifecycle exit code, Rust-owned evidence reads with artifact-read telemetry, final companion request policy plus render sequencing from JS-supplied closeout context and manifest lifecycle blocks, final `producer-performance.json` construction, closeout patch application, and final `manifest.json` file emission. JS still constructs typed request inputs from CLI flags/files and passes observations/request flags into audit-core.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Migrate remaining request construction only when CLI parsing/input ownership moves into audit-core.                                                                                 |
| Lifecycle child process execution                                                                                    | `preWrite` Rust engine selection: `pre_write_routing.rs`; `preWrite` Rust engine execution: `pre_write_lifecycle.rs`; `preWrite` JS/TS child execution: `pre_write_lifecycle.rs` spawning `pre-write.mjs`; `canonDraft`: `canon_draft_lifecycle.rs`; `checkCanon`: `check_canon_lifecycle.rs`; `postWrite`: `post_write_lifecycle.rs`; combined lifecycle sequencing: `cli/lifecycle.rs` through `execute-audit-lifecycle`                                                                 | Rust audit-core owns the base audit profile executor, pre-write routing, lifecycle child execution, sequencing, and final closeout coordination. When a lifecycle-only plan skips the base profile, JS passes the plan's exact skip reason; Rust marks base evidence unavailable with `scan-gap`, avoids stale base artifact reads, and scopes both producer performance and `artifactsProduced` to lifecycle paths proven by the current manifest block. An explicit `--profile` or `--rust-analyzer` keeps requested base work. | Migrate JS/TS pre-write producer semantics only after a dedicated parity plan.                                                                                                      |

## Canonical Rust Modules

| File                                                                           | Owns                                                                                                                                                                                                                                                                                                                                                                | Must not own                                                                                                                                                                                                                                                       |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs`              | Known artifact names, dynamic artifact filename matching, deterministic produced-artifact enumeration, and current Rust-analysis artifact usability from `manifest.json.rustAnalysis` blocks                                                                                                                                                                        | child process execution, JSON artifact parsing beyond artifact usability fields                                                                                                                                                                                    |
| `experiments/rust-main/lumin-audit-core/src/artifact_measurement.rs`           | Producer-performance artifact-size measurement for JS-supplied produced artifact names, including best-effort missing/non-file exclusion and largest-artifact projection                                                                                                                                                                                            | produced-artifact discovery, artifact JSON parsing, artifact-read timing, producer execution                                                                                                                                                                       |
| `experiments/rust-main/lumin-audit-core/src/artifact_read_metrics.rs`          | `artifact-read-metrics.v1` summary projection from JS-supplied artifact-read observations, Rust-supplied manifest-evidence artifact-read observations, and phase sidecar read observations: totals, parse failures, path naming, largest-read projection, slowest-parse projection, and per-artifact aggregation                                                    | live artifact-read observation, JSON artifact parsing, producer execution, final manifest file writing                                                                                                                                                             |
| `experiments/rust-main/lumin-audit-core/src/artifact_summaries.rs`             | `manifest.json.frameworkResourceSurfaces`, `manifest.json.unusedDependencies`, and `manifest.json.blockClones` projections from already-produced artifact JSON                                                                                                                                                                                                      | framework/resource scanning, unused-dependency analysis, block-clone detection                                                                                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/audit_review_pack.rs`              | `audit-review-pack.latest.md` Markdown companion rendering from already-produced audit artifacts: controller-lane text, review checks, artifact list projection, Rust-analysis cue placement, resolver/dependency/framework/SFC/unreachable-SCC review cue formatting, merge instructions, and result-file metadata through `audit-review-pack-render`              | audit artifact production, source walking, final manifest file writing, audit-summary console preview                                                                                                                                                              |
| `experiments/rust-main/lumin-audit-core/src/audit_summary.rs`                  | `audit-summary.latest.md` request/result protocol, Markdown section orchestration, lifecycle command notes, read-first guidance, scan-range/confidence scope text, artifact map projection, living-audit/expansion hints, guardrails, blind-zone console summary, console-preview extraction, and result-file metadata through `audit-summary-render` | measured-cue selection or formatting, audit artifact production, source walking, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/audit_summary/measured_cues.rs`    | single owner of measured-cue selection and formatting from already-produced audit artifacts, including lifecycle-only base-evidence suppression, topology/type-escape/any-contamination/fix-plan/reachability/generated-consumer/framework/dependency/SFC/Rust/call-graph/resolver cues, and the no-cues fallback | Markdown section ordering, lifecycle command projection, artifact maps, living-audit/expansion hints, blind-zone console projection, audit artifact production, or source walking |
| `experiments/rust-main/lumin-audit-core/src/barrel_discipline.rs`              | `barrels.json` artifact projection from JS-produced barrel discipline facts: request schema validation, single-package skip shape, monorepo `summary`/`byPackage` placement, and checked legacy `meta` shape preservation                                                                                                                                           | repo-mode detection, alias-map construction, source walking, JS/TS import parsing, root-barrel/subpath classification, and producer orchestration                                                                                                                  |
| `experiments/rust-main/lumin-audit-core/src/block_clones.rs`                   | `block-clones.json` artifact construction from JS-tokenized normalized token streams: request schema validation, threshold normalization, suffix-array/LCP repeated-region grouping, contained-group pruning, noise classification, cap/status projection, summary/noise-policy projection, and incremental metadata placement                                      | source walking, incremental snapshot/cache ownership, JS/TS parsing, block-clone tokenization/normalization, generated/bundled file detection, producer phase timing                                                                                               |
| `experiments/rust-main/lumin-audit-core/src/call_graph.rs`                     | `call-graph.json` artifact construction from JS-produced call graph facts: request schema validation, parse-warning/meta/support projection, fan-in map projection, `topCallees` projection, bounded member-call counters, module-call aggregation, prototype-owner aggregation, semi-dead list placement, and deterministic summary projection                     | source walking, OXC parsing, import/export/member-call extraction, resolver behavior, exported object/member matching, semi-dead import classification, prototype-call detection, producer phase timing                                                            |
| `experiments/rust-main/lumin-audit-core/src/function_clones.rs`                | `function-clones.json` artifact construction from JS-produced function facts: request schema validation, fact stamping/sorting, exact-body/structure/signature group projection, near-function candidate scoring/projection, threshold policy metadata projection, diagnostic sorting, incremental metadata placement, and deterministic summary/support projection | source walking, incremental snapshot/cache ownership, JS/TS parsing, function extraction, function body normalization/hashing, function signature hashing, generated-file detection, call-token extraction, producer phase timing                                  |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract.rs`                  | Stable JS/TS extraction facade and per-file orchestration: strict request validation, local Rayon pool ownership, parser selection, top-level definition/import/re-export extraction, relative-resolution annotation, and final file-result assembly                                                                                                                | CJS policy, dynamic-import opacity policy, named-import precision, type-escape classification, pre-write local-operation classification, class-method projection, source walking, alias/tsconfig/package resolution, or graph projection                           |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/cjs.rs`              | CommonJS export-surface, literal/dynamic `require`, namespace-member, side-effect-only, and opacity evidence                                                                                                                                                                                                                                                        | ESM import extraction, package resolution, graph fan-in/dead projection, or JS fallback policy                                                                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/dynamic_imports.rs`  | Literal/nonliteral dynamic-import evidence, member precision, template-prefix opacity, and `import.meta.glob` call facts discovered from the parsed program                                                                                                                                                                                                         | glob expansion, source inventory construction, resolver policy, or graph mutation                                                                                                                                                                                  |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/named_imports.rs`    | Named-import namespace-member and escape precision with lexical shadow tracking                                                                                                                                                                                                                                                                                     | base static-import facts, resolution, fan-in projection, or dead-export classification                                                                                                                                                                             |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/type_escape.rs`      | AST/comment type-escape detection, exported-owner association, code-shape normalization, and occurrence identity                                                                                                                                                                                                                                                    | downstream contamination ranking, source walking, or fallback selection                                                                                                                                                                                            |
| `experiments/rust-main/lumin-audit-core/src/js_ts_extract/surfaces.rs`         | Pre-write local-operation and class-method surface projection from an already-parsed program                                                                                                                                                                                                                                                                        | top-level definition extraction, dead-export ranking, or edit-safety policy                                                                                                                                                                                        |
| `experiments/rust-main/lumin-audit-core/src/relative_source_resolver.rs`       | The single inventory-bounded relative JS/TS source-target matcher shared by Rust JS/TS extraction and source-use assembly: slash/path-segment normalization, exact/file/index candidate ordering, compiled-JS-to-TS source fallback ordering, and root-relative/absolute inventory aliases                                                                          | source walking, filesystem probing outside caller-supplied inventories, alias/tsconfig/package resolution, unresolved policy, or graph projection                                                                                                                  |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly.rs`            | Stable source-use assembly facade: public protocol and build-entrypoint re-exports only                                                                                                                                                                                                                                                                             | request decoding, graph projection, resolver policy, filesystem access, or duplicate helper implementations                                                                                                                                                        |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/protocol.rs`   | Project-owned source-use request/response JSON shapes and schema constants, including resolved/external/non-source/generated terminal record identities used by linked projections                                                                                                                                                                                  | compact transport decoding, resolution, graph projection, or filesystem access                                                                                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/input.rs`      | Strict request transport normalization: path/string table lookup, compact row decoding, synthetic record IDs, compact type-only state, and normalized internal records                                                                                                                                                                                              | graph policy, source resolution, glob expansion, namespace traversal, or artifact projection                                                                                                                                                                       |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/path.rs`       | Shared source-use path text projection: normalized root-relative paths, basename extraction, lexical root containment, and relative scope text                                                                                                                                                                                                                      | filesystem probing, source walking, symlink interpretation, or resolver policy                                                                                                                                                                                     |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/glob.rs`       | Literal `import.meta.glob` pattern validation and deterministic expansion only against the caller-supplied source inventory and cap                                                                                                                                                                                                                                 | source-text discovery, repository walking, cap selection, alias resolution, or graph mutation                                                                                                                                                                      |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/namespace.rs`  | Namespace/named re-export map derivation from normalized current-request re-export records, merge of explicit standalone-call facts, and deterministic re-export-chain resolution                                                                                                                                                                                   | import extraction, repository walking, alias/tsconfig/package resolution, graph mutation, or diagnostic projection                                                                                                                                                 |
| `experiments/rust-main/lumin-audit-core/src/source_use_assembly/assembly.rs`   | Deterministic source-use graph/evidence projection from normalized records: checked target handling, pre-resolved/internal/external/unresolved/generated/SFC/glob/namespace branches, handled/skipped partitioning, counters, standalone response construction, and embedded response construction for `symbol-graph-artifact`                                      | request transport decoding, source walking, incremental cache ownership, JS/TS parsing, extractor fallback selection, SFC/MDX/generated fact discovery, alias/repo-mode discovery, arbitrary filesystem probing, or producer timing                                |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph.rs`                   | `symbols.json` orchestration from the strict v2 protocol: request preparation, source-use assembly execution, cross-projection sequencing, file-index/evidence projection, deterministic summary, and final artifact assembly                                                                                                                                       | wire-shape declarations, SFC policy, fan-in/dead algorithms, any-contamination policy, v1 compatibility, legacy precomputed result acceptance, source walking, parsing, resolver policy, incremental cache ownership, producer timing, or cached artifact mutation |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/protocol.rs`          | Strict `lumin-symbol-graph-producer-request.v2` project-owned wire types grouped into required `context`, `extraction`, `sourceUseAssembly`, and `graph` sections, including compact path IDs and typed SFC/fan-in/dead inputs                                                                                                                                      | artifact projection, compatibility adapters, defaulting missing evidence to empty values, filesystem probing, or third-party parser types                                                                                                                          |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/sfc.rs`               | Typed SFC style/template/global/generated-manifest/framework-convention projection, source-use target linkage, and status/reason selection from explicit source-use terminal outcomes                                                                                                                                                                               | SFC discovery, source walking, source-use resolution, fan-in eligibility, or safe-fix policy                                                                                                                                                                       |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/reachability.rs`      | Source-use fan-in merge, identity/space fan-in projection, top fan-in ranking, and dead-candidate partitioning                                                                                                                                                                                                                                                      | parsing, resolution, final artifact assembly, or compatibility fallback                                                                                                                                                                                            |
| `experiments/rust-main/lumin-audit-core/src/symbol_graph/any_contamination.rs` | Type-escape-to-owner association, contamination labels/measurements, owner indexes, and annotated definition projection                                                                                                                                                                                                                                             | type-escape extraction, edit-safety policy, or final artifact assembly                                                                                                                                                                                             |
| `experiments/rust-main/lumin-audit-core/src/js_ts_pre_write/cache.rs`         | Strict per-file OXC fact cache for the shared JS/TS pre-write/post-write evidence pass: repository and source-set identity, exact current-worktree byte SHA-256 identity, parsing from the same bytes used for identity, cache load/store validation, and incremental observations | source discovery, parser traversal, final evidence reuse, absence claims, cue/ranking policy, Git index/blob identity, artifact-path aliases as source identity, transformed repository bytes in place of worktree bytes, or stat/mtime-only identity                                                         |

`build-symbol-graph.mjs` requires Rust JS/TS extraction for every JS-family file
in its changed set. An audit-core command failure, malformed batch response, or
missing per-file result is a producer hard-stop. It must not import or invoke
`_engine/lib/extract-ts.mjs` as a fallback. Rust-reported per-file parse errors remain
artifact-visible parse failures; they are not helper-contract failures.

`_engine/lib/symbol-graph-discovery.mjs` owns source snapshot construction,
incremental extraction-cache classification, Rust JS/TS batch invocation,
Python/Go extraction input adaptation, and normalized per-file fact assembly.
It returns discovery facts and cache observations to `build-symbol-graph.mjs`.
It must not resolve import specifiers, mutate graph fan-in, classify dead
exports, construct source-use graph evidence, or project `symbols.json`.

`_engine/lib/symbol-graph-sfc-discovery.mjs` owns SFC framework-signal detection and
the orchestration of the checked SFC collectors. It returns raw script-import,
script-src, style-asset, template-ref, global-registration,
generated-manifest, and framework-convention facts plus collection telemetry.
It must not resolve those facts, assign graph status/reason fields, construct
source-use records, or project SFC artifact rows. Those deterministic
resolution-linked projections remain Rust-owned by `symbol_graph/sfc.rs`.

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

The strict v2 cutover is atomic. There is no v1 adapter or dual-schema window.
The current runtime bridge contract version and required feature set are owned
by `_engine/lib/audit-core.mjs` and the Rust runtime-contract command. The strict
contract requires `symbolGraphStrictRequestV2`,
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
| `experiments/rust-main/lumin-audit-core/src/checklist_facts.rs` | `checklist-facts.json` artifact construction from JS-collected AST facts plus already-produced audit artifacts: checklist gate projection, citation hints, deferred-item vocabulary, artifact-backed shape/function/dead-code/topology/barrel/lint summaries, and deterministic output shape | JS/TS source walking, OXC AST parsing, source-level function/catch extraction, optional artifact production, checklist Markdown rendering |
| `experiments/rust-main/lumin-audit-core/src/compare_repos.rs` | `compare.json` artifact construction from two already-produced audit output directories: tolerant optional artifact reads, side summary projection, numeric delta projection, missing-artifact accounting, and deterministic artifact shape while preserving the checked `compare-repos.mjs` vocabulary | audit pipeline execution, source walking, artifact production, human console rendering |
| `experiments/rust-main/lumin-audit-core/src/discipline.rs` | `discipline.json` artifact construction from JS-supplied scan-scope file inventory: regex discipline counters for TS/JS, Python, Go, unreadable-file accounting, per-file offender projection, rates, and deterministic summary fields while preserving the checked `measure-discipline.mjs` vocabulary | source walking, include/exclude/test policy, parser-based fact extraction, source-language semantic classification beyond regex counters |
| `experiments/rust-main/lumin-audit-core/src/shape_index.rs` | `shape-index.json` artifact construction from JS-extracted shape-hash facts: request schema validation, fact/diagnostic/error sorting, `groupsByHash` projection, generated-file fact counting, incremental metadata pass-through, and checked `meta.supports` placement | source walking, incremental snapshot/cache ownership, JS/TS parsing, shape-hash fact extraction, generated-file detection, and normalized hash policy |
| `experiments/rust-main/lumin-audit-core/src/pre_write_routing.rs` | Typed pre-write engine routing from requested engine, intent flag, and already-read intent JSON text: `auto`/`js`/`rust` selection, intent language validation, explicit mismatch hard-stops, child intent flag/input projection, and removal of route-only `language` before Rust engine stdin | intent file/stdin reading, JS/TS `pre-write.mjs` producer semantics, Rust analyzer internals, child process execution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle.rs` | `manifest.preWrite` raw lifecycle block execution for both pre-write engines: Rust engine `lumin-rust-analyzer pre-write` child spawning, source-commit fallback when JS omits the value, intent stdin forwarding, stale current-output invalidation, exact native schema/policy/producer/coverage/required-array validation, native artifact latest copy, advisory JSON construction, JS-supplied file inventory and failure pass-through, Rust pre-write capability fields, typed child/artifact failure projection, and product-mode inherited stdout/stderr with result JSON written out-of-band; JS/TS engine `pre-write.mjs` child spawning, routed intent stdin forwarding, stale latest invalidation, matching current-run latest/specific advisory readback, advisory path/invocation projection, typed child/output failure projection, and product-mode inherited stdout/stderr with result JSON written out-of-band | JS/TS `pre-write.mjs` producer semantics, scan-scope walking or source inventory interpretation, Rust analyzer internals, post-write delta semantics, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/post_write_lifecycle.rs` | `manifest.postWrite` raw lifecycle block execution for `--post-write`: missing/malformed advisory hard-stop, existing `post-write.mjs` child spawning, optional delta-out/no-fresh-audit/scan/incremental argv forwarding, stale latest invalidation, non-zero child failure propagation, exact current-run delta schema/advisory-id/specific-copy validation, typed failure projection, delta summary projection, and product-mode inherited stdout/stderr with result JSON written out-of-band. In implicit quick post-write-only mode the orchestration plan skips the unrelated base profile because this child owns fresh delta-required inventory production; manifest base evidence is explicitly unavailable rather than loaded from a reused output directory. | post-write delta computation, type-escape/file-delta classification semantics, markdown rendering, pre-write advisory construction, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/living_audit.rs` | `manifest.json.livingAudit` projection from known living-audit document candidate paths under the audited root | audit document authoring, final answer policy, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/manifest_core.rs` | `manifest.json.scanRange`, `manifest.json.confidence`, and `manifest.json.sfcEvidence` projections from already-produced `triage.json` and `symbols.json` | blind-zone detection, living-audit document discovery, producer execution |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics.rs` | `manifest.json.resolverDiagnostics` projection from already-produced `symbols.json`, `resolver-capabilities.json`, and `resolver-diagnostics.json` | module resolution, blocked-hint production, blind-zone detection |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics_artifacts.rs` | `resolver-capabilities.json` and `resolver-diagnostics.json` artifact construction from already-produced `symbols.json` unresolved resolver facts: static capability matrix projection, unresolved import rows, unsupported import rows, diagnostic-only candidate targets, resolver/generated blind-zone rows, blocked-candidate hints, deterministic sorting, and artifact-local summary pivots | JS/TS module resolution, source parsing, symbol graph production, generated artifact discovery, filesystem scan-scope checks, resolver relevance matching against findings, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/runtime_evidence.rs` | `runtime-evidence.json` artifact construction from already-produced `symbols.json.deadProdList` and JS-supplied Istanbul coverage JSON: runtime status classification, hit-count projection, orphan static-file summary, deterministic summary projection, and preserved checked status vocabulary | coverage generation, test execution, coverage-file discovery, JS/TS dead-export classification, source parsing, safe-fix ranking, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs` | `rust-analyzer-health.latest.json` manifest summary projection, root mismatch, invalid-shape, complete/available status, and `manifest.json.rustAnalysis` merge projection from JS-observed Rust analyzer run state plus already-produced evidence summary | Rust source parsing, source-health analysis, Cargo oracle execution, child process execution |
| `experiments/rust-main/lumin-audit-core/src/sarif.rs` | `lumin-repo-lens-lab.sarif` SARIF 2.1.0 projection from already-produced `fix-plan.json`, `runtime-evidence.json`, `staleness.json`, `dead-classify.json`, `symbols.json`, `topology.json`, `discipline.json`, and `barrels.json`: rule catalog, severity mapping, result properties, artifact-used evidence, warning propagation, and deterministic URI/message projection | SARIF upload, producer execution, source parsing, fix ranking, topology/discipline/barrel analysis, artifact file discovery, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/staleness.rs` | `staleness.json` temporal evidence construction from already-produced `symbols.json.deadProdList` plus git log/blame/pickaxe observations: staleness tier classification, grounding/confidence projection, incremental staleness cache hit/miss behavior, deterministic summary projection, and preserved checked artifact vocabulary | JS/TS source parsing, dead-export classification, safe-fix ranking, git repository discovery outside the configured root, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/topology.rs` | `topology.json` artifact assembly from JS-produced per-file topology entries: `nodes`/`edges` materialization, fan-in/fan-out summaries, runtime/static SCC projection from already-resolved edges, cross-submodule aggregation from JS-supplied submodule labels, largest-file projection, summary counts, and Rust metadata placement | source walking, JS/TS/Python/Go parsing, module resolution, alias-map construction, incremental cache ownership, Rust topology sidecar comparison, repository mode discovery, submodule label discovery, and manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/topology_mermaid.rs` | `topology.mermaid.md` Markdown companion rendering from already-produced `topology.json`: capped cross-submodule Mermaid graph, capped runtime cycle graph, hub-file notes, omitted-detail limits, citation contract text, and result-file metadata through `topology-mermaid-render` | topology analysis, source walking, module resolution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/generated_artifacts.rs` | `manifest.json.generatedArtifacts` projection from already-produced `symbols.json`, generated-artifact mode validation, generated miss grouping, blind-zone grouping, and present/prepared out-of-scope evidence | package resolution, generator execution, generated-artifact producer evidence construction |
| `experiments/rust-main/lumin-audit-core/src/framework_resource_surfaces.rs` | `framework-resource-surfaces.json` artifact construction from JS-supplied discovered files, package records, and sampled content: path normalization, nearest package selection, dependency grounding, framework/resource lane classification, deterministic sorting, and artifact-local summary projection | source walking, scan/exclude/include-tests policy, workspace discovery, package manager interpretation, content sampling, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/entry_surface.rs` | `entry-surface.json` artifact projection from JS-supplied public API, package-script, HTML, framework, config, known-file, parse-error, and submodule facts: entry-file union, evidence merge, unsupported/unresolved sample ordering, meta/support projection, global/submodule completeness projection, and deterministic field projection | public/package/html/script/framework/config discovery, JS/TS source parsing, package interpretation, re-export resolution, filesystem walking, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/export_action_safety.rs` | `export-action-safety.json` artifact projection from JS-produced edit-safety findings: request schema validation, `meta` projection, `total` derivation, warning placement, and `byId` materialization from non-empty finding ids | JS/TS source parsing, local reference counting, export/declaration matching, safe action selection, edit-range proof production, and producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/module_reachability.rs` | `module-reachability.json` artifact construction from already-produced `symbols.json` and `entry-surface.json` facts: known-file collection, runtime/type graph projection from `resolvedInternalEdges`, bounded BFS with checked JS counter semantics, unreachable file projection, runtime SCC review evidence, deterministic sorting, and artifact-local summary projection | JS/TS module resolution, source parsing, symbol graph production, entry-surface discovery, package interpretation, re-export resolution beyond consumed facts, safe-delete claims, manifest summary rendering |
| `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs` | `fix-plan.json` request/artifact protocol, proposal flattening, finding identity and evidence indexing, action-safety merge, support evidence projection, deterministic sorting, artifact-local summary, and `safeFixGroups` projection | four-tier decision policy, JS/TS parsing, package export/deep-import policy discovery, edit safety proof production, SARIF rendering, and producer orchestration |
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
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs` | Base audit child-process execution for planned base pipeline steps, runtime executor request projection that builds the base plan inside audit-core, filesystem precondition evaluation using the existing plan reasons, JS/MJS child argv construction, typed `commandsRun` / `skipped` value production, `LedgerEvent` value production from the same observations, Rust analyzer source-commit fallback when JS omits the value, child status/wall/stderr observation, and orchestrator memory snapshots before and after base children | JS/TS producer internals, lifecycle child execution, artifact-read timing, phase timing reads, human renderers, `blindZones`, final `manifest.json` writing |
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
| `experiments/rust-main/lumin-audit-core/src/cli/manifest.rs` | CLI runners for manifest metadata, root/evidence assembly and refresh, lifecycle evidence refresh, focused update/core-summary commands, manifest-evidence artifact reads, and shared JSON result transport. For base-skipped lifecycle runs it validates the plan skip reason, emits a standard `scan-gap`, and suppresses stale base reads. | final manifest writes, companion policy/render sequencing, producer-performance closeout, current lifecycle artifact inventory, producer orchestration, lifecycle child sequencing before closeout, blind-zone owner migration before parity, or artifact-read observation outside manifest-evidence summary/refresh/root/finalize inputs |
| `experiments/rust-main/lumin-audit-core/src/cli/manifest/finalize.rs` | single CLI owner for `manifest-write`, `manifest-closeout-write`, `finalize-audit-run`, and `finalize-audit-run-with-companions`: final manifest and producer-performance writes, companion request policy and render sequencing, companion artifact read observation, current lifecycle-only artifact inventory from manifest-bound paths/IDs, closeout update application, and bounded result projection | manifest evidence discovery/summary policy, lifecycle child execution, producer orchestration before closeout, source walking, or blind-zone semantics |
| `experiments/rust-main/lumin-audit-core/src/cli/lifecycle.rs` | CLI runners for lifecycle summary, focused lifecycle wrappers, and the combined `execute-audit-lifecycle` result-file command that sequences request guard, pre-write routing, pre/post/canon/check lifecycle execution, and strict post-write exit policy while preserving child stdout/stderr streaming | JS/TS producer semantics, public CLI argument parsing, intent file/stdin reading, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/cli/orchestration.rs` | CLI runners for orchestration plan/result, base-plan execution, producer-performance artifacts, and living-audit summary | product projection logic beyond delegating to owned audit-core modules |
| `experiments/rust-main/lumin-audit-core/src/cli/usage.rs` | CLI usage text for audit-core commands | command implementation or product projection logic |
| `experiments/rust-main/lumin-audit-core/src/lib.rs` | public library exports for audit manifest wrappers | ad hoc JSON shape construction outside owned modules |

### Packaged Runtime Contract

The JS bridge accepts an audit-core helper only when its reported bridge
contract version and required feature set match the bridge source. Adding a
required feature is a contract change even when the CLI schema remains
backward-compatible. That change must bump the bridge contract version in the
JS resolver, Rust runtime contract, package builder, and contract tests.

`_engine/lib/audit-core.mjs` is the single JS owner of runtime-contract validation and
the executable result-output fixture probe. `scripts/build-skill.mjs` must call
that owner before copying the current-platform helper; it must not maintain a
second subcommand list, feature list, fixture builder, or payload matcher.

Every checked-in platform binary advertised by
`_engine/bin/audit-core-platforms.json` must be rebuilt from that same contract
before the generated skill is committed. A stale binary must not retain the
new contract version or rely on Cargo auto-build to hide the mismatch. The
packaged binary is the normal installed runtime; the Cargo workspace is a
fallback for unsupported or missing platforms.

Linux release binaries must be built in a controlled compatibility-baseline
environment, not directly against the maintainer's current WSL glibc. The
packaging verification must inspect the maximum required GLIBC symbol version;
the current `linux-x64` GNU baseline is GLIBC 2.31 or older.

Native Node dependencies are runtime-platform artifacts too. A WSL install
must install the skill dependencies from a supported Node/npm toolchain inside
WSL and must not reuse a Windows `node_modules` tree. Missing native bindings
are an unavailable runtime dependency, not clean parser evidence.

For `js-ts-pre-write-evidence` only, the JS bridge may prefer the packaged
Windows x64 audit-core while running under x64 WSL against a Windows-mounted
repository. `_engine/lib/js-ts-rust-evidence.mjs` owns command-specific request and
response path translation; `_engine/lib/audit-core.mjs` owns current-contract
validation, Windows helper selection, mounted temporary result transport, and
fail-closed execution. Explicit Linux/generic binary overrides disable this
optimization. An unavailable or incompatible Windows candidate may select the
normal Linux helper before execution, but a started Windows command must never
fall back after failure. This route must not spread to other subcommands
without a separate owner/contract review.

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
  exceptions for lifecycle child execution. `pre_write_lifecycle.rs` is the
  explicit Rust pre-write exception: it may run `lumin-rust-analyzer pre-write`
  and project the checked Rust advisory block, but it must not own JS/TS
  `pre-write.mjs` producer semantics. `post_write_lifecycle.rs` is the explicit
  post-write exception: it may run the existing `post-write.mjs` entrypoint and
  project the checked raw lifecycle block, but it must not own post-write delta
  producer semantics. These modules run existing producer entrypoints but do
  not reinterpret source-language producer semantics.
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
  normal fresh post-write reuses the same command's `anyInventory` and `files`
  projections for the after-snapshot and file delta input. The JS bridge may
  write `any-inventory.json` and pass those returned files to the existing
  delta owner, but it must not run `any-inventory.mjs`, load Node `oxc-parser`,
  or perform a second repository walk as a fallback. `any-inventory.mjs`
  remains only the legacy/reference producer and explicit no-fresh paths may
  retain parser-free file-delta discovery. This
  Rust owner must not write `symbols.json` or `topology.json`, broaden the
  supplied scan scope, invent fallback evidence, or own cue-tier/rendering
  policy. A malformed request, unreadable required source, omitted response
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
  cache root. On an x64 WSL-mounted Windows worktree, the JS bridge may route
  this command through the exact-contract packaged Windows helper so discovery,
  worktree reads, and hashing use native NTFS. That transport optimization must
  not change the current-worktree-byte identity or artifact semantics. Host
  route unavailability is distinct from an executed helper returning null or
  malformed evidence: only the former may select the Linux helper. With
  incremental reuse disabled, host result transport uses a shared Windows temp
  directory and must not create or require the configured cache root. Explicit
  source requests are canonicalized at the request boundary and rejected when
  they escape the canonical root. Warm cache hits release their worktree byte
  buffers immediately after hashing and identity comparison.
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
