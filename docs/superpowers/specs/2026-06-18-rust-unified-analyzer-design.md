# Rust Unified Analyzer Design

## Problem

The current Rust work has two executable surfaces:

- `rust-source-health` for syntax-only Rust parser signals.
- `rust-cargo-oracle` for Cargo semantic evidence.

That split was useful while M6 and M7 were being proven, but it is the wrong
product shape. It makes Rust look like two separate tools that must be manually
composed. The JS/TS analyzer does not work that way: parsing, review policy,
semantic evidence, and artifact assembly are one pipeline with shared
vocabulary.

Rust should follow the same product model.

## Decision

Build one Rust analyzer surface.

The final product boundary is a single Rust main CLI and one unified artifact.
`rust-source-health` and `rust-cargo-oracle` become internal phases, not
separate product tools.

```text
lumin-rust-analyzer
  syntax phase        -> Rust parser observations and syntax-only review signals
  policy phase        -> shared visibility, mute, and review policy vocabulary
  cargo phase         -> Cargo semantic diagnostics and clean/unavailable evidence
  merge phase         -> per-file and whole-run summary assembly
  artifact phase      -> one product artifact
```

Modules may stay separated internally. User-facing execution and product
artifacts must not stay separated.

## Why This Is The Right Shape

Separate tools create separate policy languages. That is the bug.

The Rust pipeline needs the same shape as the existing JS/TS pipeline:

1. Parse and collect cheap observations.
2. Apply policy to decide review vs muted visibility.
3. Add semantic evidence where available.
4. Preserve unavailable/partial coverage honestly.
5. Emit one artifact that downstream UX can render without guessing.

The cargo phase should not be a second product. It is evidence enrichment for
the Rust analyzer.

## JS/TS Structure Analysis

The Rust migration follows the existing JS/TS interpretation pipeline, not a new
parallel policy system.

The JS/TS path separates responsibilities this way:

1. `_lib/extract-ts.mjs` lowers TS/JS/JSX files into the shared
   `{filePath, defs, uses, reExports, loc}` shape. `build-symbol-graph.mjs`
   consumes that shape without switching on language again.
2. `_lib/parse-oxc.mjs` is the single parser entry point. It escalates
   `oxc-parser` `errors[]` into thrown errors so malformed files do not silently
   produce empty definitions or fake dead symbols.
   Rust mirrors this with the `rust-source-health` / `ra_ap_syntax` lane:
   syntax parsing emits facts, parse status, and AST opaque surfaces, but does
   not pretend to know macro expansion, trait resolution, or live `cfg` branches.
3. `_lib/resolver-core.mjs` distinguishes concrete local files,
   `NON_SOURCE_ASSET`, `EXTERNAL`, `UNRESOLVED_INTERNAL`, and unresolved
   relative misses. External imports are not resolver blindness; unresolved
   internal imports are finding-local taint candidates.
4. `classify-dead-exports.mjs` applies false-positive policies before proposal
   buckets, but policy exclusions are materialized as `excludedCandidates`
   instead of being dropped.
5. `_lib/finding-provenance.mjs` attaches `supportedBy`, `taintedBy`,
   `resolverConfidence`, and `parseStatus` to each finding. Blocking taint is
   local to the affected finding; soft taint demotes instead of suppressing.
   Rust mirrors this as `files[*].oracleBridge`: each file gets its parser
   `parseStatus`, Cargo/rustc support evidence, local opacity taint, oracle
   coverage taint, and `oracleConfidence`. The bridge is file-local evidence,
   not a global excuse to demote every Rust result. Rust semantic findings also
   carry finding-local `supportedBy`, `taintedBy`, `parseStatus`,
   `oracleConfidence`, and `actionTier`, matching the TS/JS provenance fields
   on the actual review candidate.
6. `_lib/ranking.mjs` is the public tier gate: `SAFE_FIX`, `REVIEW_FIX`,
   `DEGRADED`, and `MUTED`. `rank-fixes.mjs` turns `excludedCandidates` into
   auditable `MUTED` output.
7. `_lib/pre-write-cue-tiers.mjs` and `_lib/unused-deps-artifact.mjs` keep
   unavailable evidence explicit. Missing support is `UNAVAILABLE` or
   `unavailable`, not an empty clean result.
8. `_lib/rust-topology-prefer.mjs` promotes Rust sidecar output only after an
   artifact contract guard passes. Contract failure is surfaced as
   `blocked-artifact-contract`, not silently used as product evidence.

Rust parity means preserving that structure:

- raw evidence stays present;
- muted evidence stays auditable;
- unavailable coverage stays explicit;
- syntax-only evidence is candidate confidence, not a verified product claim;
- semantic Cargo evidence can be `verified`, `rule-backed`, `candidate`, or
  `unavailable` depending on its source and coverage.
- Rust AST facts are collected first. Macro expansion and `cfg` gates are
  preserved as opaque surfaces so the unified analyzer knows when Cargo/rustc
  oracle evidence is needed.
- Cargo/rustc is the Rust analogue of the TS `tsc` oracle. The TS/JS code uses
  `ts.readConfigFile` and `ts.parseJsonConfigFileContent` in
  `_lib/tsconfig-paths.mjs` so config/resolution evidence cannot drift from
  `tsc`. Rust uses Cargo metadata plus `cargo check --message-format=json` for
  the same class of compiler-owned evidence.
- The product default is AST plus Cargo metadata, not full `cargo check`.
  This mirrors the JS/TS fast path: parser facts and cheap config evidence run
  first, and the compiler oracle is explicit. `lumin-rust-analyzer` defaults to
  `semanticMode: "metadata-only"` and records `streamParseStatus: "not-run"` for
  Cargo-check coverage; callers opt into rustc diagnostics with
  `--semantic-mode cargo-check` / `--cargo-check`.
- Targeted Cargo-check mode is the Rust analogue of the JS/TS parser-first,
  oracle-on-demand path. `--semantic-mode targeted-cargo-check` lets the Rust AST
  phase choose compiler-resolvable review opaque file paths, currently review
  macro expansion and `cfg` opacity, then the Cargo oracle maps those paths
  through Cargo metadata to package-scoped `cargo check --package ...` calls.
  Syntax-only style review signals such as clone/unwrap/oversized functions
  stay review evidence; they do not wake the compiler oracle because rustc
  cannot decide those product claims. Review `cfg` opacity wakes the current
  Cargo scope oracle, but the opaque surface remains review evidence and the
  file confidence stays below high unless an explicit feature/target matrix is
  later added; one default `cargo check` run cannot honestly prove all inactive
  `cfg` branches. If the AST phase only finds muted evidence or style-only
  review signals, the oracle records `oraclePlan.status: "not-run"` and
  unavailable coverage instead of inventing a clean claim.
- Targeted mode preserves the ranked package order selected from AST evidence
  and Cargo metadata. It does not cap package execution by elapsed wall time or
  by a fixed package count: large repositories must complete, expose
  unavailable coverage with artifact-visible evidence, or hard-stop on a real
  contract failure. Callers can still narrow with `--package`, or request full
  `cargo-check` explicitly.
- Cargo target output never defaults to the analyzed repo's `target/`
  directory. The default `--cargo-target-dir-mode isolated-temp` gives each
  oracle run an owned temporary target directory that is removed on drop.
  `--cargo-target-dir-mode reusable-temp` is an explicit dogfooding/performance
  mode that reuses a Lumin-owned temp target cache keyed by analyzed root and
  toolchain. This is not semantic-result cache reuse: the artifact still marks
  `analysisInputSetComplete: false`, keeps the full coverage status, and records
  `meta.input.cargoTargetDirMode` plus `meta.input.cargoTargetDir` so reviewers
  can see exactly which execution mode produced the evidence.
  This guard is Rust-only because Cargo writes build products, incremental
  state, and debug artifacts while the JS/TS parser path emits JSON artifacts
  directly. When the oracle owns `CARGO_TARGET_DIR`, it also disables
  incremental and debug-symbol output for that subprocess. That is storage
  hygiene only: it must not skip packages, cap analysis time, hide diagnostics,
  or change the semantic coverage contract. Stale cleanup is restricted to
  Lumin-owned temp target directory prefixes and never removes the analyzed
  repo's `target/`.
- Rust opaque surfaces follow the same false-positive discipline as JS/TS
  findings: common low-review macro opacity is `muted` with a reason, while
  unknown or risky macro and non-test `cfg` opacity remains `review`.
- Safe-fix calibration follows the JS/TS P6 measurement shape: callers may pass
  `--calibration-adjudication <path>` with adjudication entries shaped like
  `{ "entries": [{ "tier": "SAFE_FIX", "verdict": "true_dead", "file": "src/lib.rs", "diagnosticCode": "unused_mut", "lineStart": 1 }] }`.
  `file` anchors the entry to an observed Rust cleanup candidate; when present,
  `diagnosticCode` and `lineStart` must also match the selected safe-action
  proof/edit, so same-file adjudications cannot calibrate unrelated findings.
  Without adjudication the bridge stays `pending`/`Red`. With a measured
  denominator, FP thresholds decide Red vs Yellow; one tiny local corpus remains
  Yellow with `benchmark-incomplete`, not Green.

## Target Artifact Shape

The unified artifact should expose both phase-specific evidence and a combined
summary.

```json
{
  "schemaVersion": "rust-analyzer-health.v1",
  "meta": {
    "producer": "lumin-rust-analyzer",
    "mode": "rust-main",
    "generated": "2026-06-18T00:00:00Z"
  },
  "summary": {
    "files": 0,
    "syntaxReviewSignals": 0,
    "syntaxMutedSignals": 0,
    "syntaxDefinitions": 0,
    "syntaxPathRefs": 0,
    "syntaxMethodCallSites": 0,
    "syntaxMethodCalls": 0,
    "syntaxOpaqueSurfaces": 0,
    "syntaxReviewOpaqueSurfaces": 0,
    "syntaxMutedOpaqueSurfaces": 0,
    "syntaxMutedOpaqueSurfacesByReason": {},
    "verifiedSemanticFindings": 0,
    "semanticCoverageUnavailableDiagnostics": 0,
    "actionTierSummary": {
      "SAFE_FIX": 0,
      "REVIEW_FIX": 0,
      "DEGRADED": 0,
      "MUTED": 0,
      "UNAVAILABLE": 0
    },
    "evidenceTierSummary": {
      "review": 0,
      "degraded": 0,
      "muted": 0,
      "unavailable": 0,
      "total": 0
    },
    "semanticSafeActions": 0,
    "semanticActionBlockedFindings": 0,
    "semanticReviewFindings": 0,
    "semanticDegradedFindings": 0,
    "semanticDegradedCoverageEntries": 0,
    "semanticClean": {
      "status": "ran",
      "clean": true,
      "cleanKind": "verified-rustc-error-absence"
    },
    "cacheReuse": {
      "status": "not-reusable"
    },
    "oracleBridgeStatus": "oracle-covered"
  },
  "actionPolicy": {
    "schemaVersion": "rust-action-tier.v1",
    "safeFixGate": {
      "status": "strict",
      "reason": "SAFE_FIX requires proofComplete safeAction with empty actionBlockers"
    },
    "reviewFixGate": {
      "status": "explicit",
      "reason": "REVIEW_FIX is selected-action blockers plus verified/rule-backed semantic findings without safe edit proof"
    },
    "summary": {
      "SAFE_FIX": 0,
      "REVIEW_FIX": 0,
      "DEGRADED": 0,
      "MUTED": 0,
      "UNAVAILABLE": 0,
      "total": 0
    },
    "evidenceTierSummary": {
      "review": 0,
      "degraded": 0,
      "muted": 0,
      "unavailable": 0,
      "total": 0
    },
    "semanticSafeActions": {
      "findings": 0,
      "sampleLimit": 5,
      "examples": []
    },
    "semanticActionBlockers": {
      "findings": 0,
      "sampleLimit": 5,
      "byReason": {},
      "examples": []
    },
    "semanticReview": {
      "findings": 0,
      "sampleLimit": 5,
      "byReason": {},
      "examples": []
    },
    "semanticDegraded": {
      "findings": 0,
      "coverageEntries": 0,
      "sampleLimit": 5,
      "byReason": {},
      "examples": []
    }
  },
  "oracleBridge": {
    "schemaVersion": "rust-oracle-bridge.v1",
    "status": "oracle-covered",
    "purpose": "connect AST syntax opacity to Cargo/rustc oracle coverage before accuracy calibration",
    "policy": {
      "opaqueSurfacesRemainEvidence": true,
      "doesNotPromoteSafeFix": true,
      "policyExclusionsRemainAuditable": true,
      "calibrationStatus": "pending",
      "calibration": {
        "status": "pending",
        "reason": "rust-safe-fix-calibration-corpus-not-measured",
        "candidateCounts": {
          "available": true,
          "safeFix": 0,
          "reviewFix": 0,
          "reviewVisibleCleanup": 0,
          "degraded": 0,
          "muted": 0,
          "unavailable": 0
        },
        "readiness": {
          "gate": "Red",
          "reasons": [
            {
              "code": "fp-rate-unknown",
              "severity": "red",
              "detail": "adjudication denominator is empty or incomplete"
            }
          ],
          "safeFix": {
            "falsePositives": 0,
            "trueDead": 0,
            "inconclusive": 0,
            "notApplicable": 0,
            "fpRate": null
          },
          "reviewVisibleCleanup": {
            "falsePositives": 0,
            "trueDead": 0,
            "inconclusive": 0,
            "notApplicable": 0,
            "fpRate": null
          }
        },
        "readinessPolicy": {
          "source": "_lib/p6-measurement.mjs::computeReadiness",
          "safeFixFpRedThreshold": 0.05,
          "reviewVisibleFpRedThreshold": 0.25,
          "reviewVisibleFpGreenThreshold": 0.1,
          "minNonTrivialCorpus": 2,
          "defaultMinAdjudicatedPerCorpus": 50
        },
        "requiredEvidence": [
          "non-empty-safe-fix-population",
          "known-safe-fix-fp-denominator",
          "readiness-gate-from-real-corpus"
        ],
        "jsTsPrecedent": {
          "measurementArtifact": "p6-measurement.json",
          "measurementOwner": "_lib/p6-measurement.mjs",
          "readinessGateOwner": "_lib/p6-measurement.mjs::computeReadiness",
          "calibrationCorpusRegistry": "_lib/calibration-corpora.mjs",
          "thresholdPolicyMetadata": "_lib/threshold-policies.mjs"
        }
      }
    },
    "syntax": {
      "reviewSignals": 0,
      "mutedSignals": 0,
      "reviewOpaqueSurfaces": 0,
      "mutedOpaqueSurfaces": 0,
      "mutedOpaqueSurfacesByReason": {}
    },
    "semantic": {
      "verifiedFindings": 0,
      "ruleBackedFindings": 0,
      "candidateFindings": 0,
      "coverageUnavailableDiagnostics": 0,
      "semanticClean": {
        "status": "ran",
        "clean": true
      }
    },
    "coverage": {
      "cargoEventStream": {
        "status": "ran"
      },
      "absenceClean": {
        "status": "ran",
        "clean": true,
        "cleanKind": "verified-rustc-error-absence"
      }
    }
  },
  "files": {
    "src/lib.rs": {
      "syntax": {
        "signalSummary": {
          "review": 0,
          "muted": 0
        },
        "reviewSignals": [],
        "mutedSignals": [],
        "astSummary": {
          "definitions": 0,
          "useTrees": 0,
          "pathRefs": 0,
          "methodCallNames": 0,
          "methodCallSites": 0,
          "methodCalls": 0,
          "macroCalls": 0,
          "cfgGates": 0,
          "opaqueSurfaces": 0,
          "reviewOpaqueSurfaces": 0,
          "mutedOpaqueSurfaces": 0,
          "mutedOpaqueSurfacesByReason": {}
        },
        "astExamples": {
          "definitions": [],
          "useTrees": [],
          "pathRefs": [],
          "methodCallCounts": [],
          "methodCalls": [],
          "macroCalls": [],
          "cfgGates": [],
          "reviewOpaqueSurfaces": []
        }
      },
      "semantic": {
        "diagnostics": []
      },
      "oracleBridge": {
        "schemaVersion": "rust-file-oracle-bridge.v1",
        "status": "oracle-covered",
        "parseStatus": "ok",
        "oracleConfidence": "high",
        "supportedBy": [],
        "taintedBy": [],
        "syntax": {
          "parseErrors": 0,
          "reviewSignals": 0,
          "mutedSignals": 0,
          "reviewOpaqueSurfaces": 0,
          "mutedOpaqueSurfaces": 0
        },
        "semantic": {
          "findings": 0,
          "diagnostics": 0,
          "safeActions": 0,
          "actionBlockedFindings": 0,
          "reviewFindings": 0,
          "candidateFindings": 0
        },
        "coverage": {
          "cargoEventStream": {
            "status": "ran"
          },
          "absenceClean": {
            "status": "ran",
            "clean": true
          }
        }
      }
    }
  },
  "artifactRefs": {
    "syntax": {
      "artifact": "rust-source-health",
      "rawEmbedded": false
    },
    "semantic": {
      "artifact": "rust-cargo-oracle",
      "rawEmbedded": false
    }
  },
  "phases": {
    "syntax": {
      "embedded": "brief",
      "rawEmbedded": false
    },
    "semantic": {
      "embedded": "brief",
      "rawEmbedded": false
    }
  },
  "coverage": [],
  "semanticFindings": []
}
```

The exact schema can evolve, but the product invariant cannot: one run, one
artifact, shared vocabulary, and no full raw phase embedded twice in the
product artifact.

Default product artifacts are compact JSON. Human-readable raw phase payloads
remain available through compatibility CLIs, but the unified product artifact
must carry counts, capped per-file examples, and raw-lane references rather
than pretty-printed raw AST/semantic lanes. This follows the JS/TS artifact
pattern where manifests and review surfaces expose counts plus bounded examples
while larger raw evidence stays in explicit side artifacts.

## Policy Model

Rust must not invent a second false-positive policy layer.

Shared concepts should be named once:

- visibility: `review`, `muted`
- mute reasons: test path, generated path, cfg-test, test attribute, dependency
  scope, unavailable semantic scope, assertion macro, collection macro, data
  literal macro, formatting macro, IO formatting macro, logging macro
- confidence tiers: syntax-only candidate, rule-backed, verified, unavailable
- clean scope: absence of verified rustc error diagnostics for the declared
  Cargo-check scope
- action tiers: `SAFE_FIX`, `REVIEW_FIX`, `DEGRADED`, `MUTED`, and
  `UNAVAILABLE`, matching JS/TS naming. `actionTierSummary` counts semantic
  action/finding candidates only. Rust must emit `SAFE_FIX` only when it has a
  proof-carrying edit action with empty action blockers. The first safe action
  class is rustc `MachineApplicable` suggestions for rule-backed warning
  diagnostics. The selected edit proof is the safety boundary; lint-code
  allowlists are too narrow for the TS/JS SAFE_FIX model.
- evidence tiers: syntax-only signals, AST opaque surfaces, muted syntax
  evidence, and coverage gaps are not action candidates. They lower into
  `evidenceTierSummary` and detailed evidence reason maps. This prevents
  clone-call or test macro counts from looking like hundreds of fix tasks.
  `DEGRADED` evidence is reserved for explicit contradiction or insufficient
  coverage classes, not a prettier spelling of review.
- degraded semantics: mirror `_lib/ranking.mjs`'s "globally insufficient or
  unclassified bucket" rule. The first Rust degraded classes are semantic
  `candidate` findings (`candidate.rust.unclassified-cargo-diagnostic`) and
  evidence entries for coverage with `status: "unavailable"` when the oracle
  produced a usable artifact but cannot support a clean/absence claim.
  Coverage-unavailable diagnostics remain `UNAVAILABLE`; they describe
  diagnostics excluded from user-facing findings, not reviewable action
  candidates.
- action blockers: mirror `_lib/export-action-safety.mjs` and
  `_lib/ranking.mjs`. If a rustc suggestion is otherwise in a safe-action
  family but has macro expansion, non-user-code primary spans, invalid ranges,
  overlapping edits, missing replacement text, non-warning level,
  non-rule-backed confidence, or no `MachineApplicable` suggestion,
  `safeAction` stays absent and `actionBlockers[]` records the hard stop.
  Unified ranking counts those findings as `REVIEW_FIX`, never `SAFE_FIX`.
  Blockers for stronger unselected actions must not block a selected safe
  action; that is the core TS/JS `safeAction` model.
- review semantics: `REVIEW_FIX` is explicit, not the trash bin for every
  non-safe signal. It has two semantic sources: selected-action blockers, and
  verified/rule-backed diagnostics that have no complete safe edit proof.
  Candidate semantic findings remain `DEGRADED`; coverage-unavailable
  diagnostics remain `UNAVAILABLE`; syntax evidence remains in
  `evidenceTierSummary`. `actionPolicy.semanticReview.byReason` preserves the
  claim kind, such as `verified.rust.rustc-error-diagnostic`, so reviewers can
  see why a finding stayed reviewable instead of safe.
- oracle bridge: AST opaque surfaces are connected to Cargo/rustc oracle
  coverage before they are calibrated. `oracleBridge` reports syntax review and
  muted opacity next to event-stream and absence-clean coverage status. It does
  not promote `SAFE_FIX`, keeps policy-excluded evidence auditable, and does
  not clear review evidence by itself. It is the evidence joint for the next
  accuracy pass. The per-file `files[*].oracleBridge` mirrors
  `_lib/finding-provenance.mjs`: it carries `supportedBy`, `taintedBy`,
  `parseStatus`, and `oracleConfidence` so partial compiler coverage is scoped
  to the relevant file instead of poisoning the whole run. The semantic finding
  projection then narrows this further: `semanticFindings[*].taintedBy` only
  receives AST opaque-surface taint when a review-visible macro or `cfg` surface
  overlaps that finding's rustc span. File-level opacity by itself must not
  downgrade every finding in the file. `SAFE_FIX` remains controlled by
  `safeAction.proofComplete` and empty action blockers, not by unrelated syntax
  opacity.
- action examples: top-level `summary` carries counts and blocker reason
  distribution only. `actionPolicy.semanticSafeActions.examples[]`,
  `actionPolicy.semanticActionBlockers.examples[]`,
  `actionPolicy.semanticReview.examples[]`, and
  `actionPolicy.semanticDegraded.examples[]` are capped review samples, not a
  second raw semantic lane. Full semantic finding evidence remains in
  `semanticFindings[]`; full coverage evidence remains in `coverage[]`.
- artifact contract: mirror `_lib/rust-topology-prefer.mjs`'s
  `blocked-artifact-contract` guard. The unified Rust analyzer must hard-stop
  before writing if `summary.actionTierSummary` or `summary.evidenceTierSummary`
  drifts from `actionPolicy`, or if full raw phase lanes are embedded in the
  product artifact.

Language-specific code may detect different facts, but it must lower into the
same policy vocabulary. Rust-specific parser and Cargo details are allowed.
Rust-specific near-miss policy names are not.

## Component Boundaries

### `syntax`

Owns Rust parser traversal, syntax-only facts, and raw syntax signal creation.
This is the current `rust-source-health` analyzer logic.

It must not know Cargo semantic results.

### `policy`

Owns review/mute visibility, shared signal vocabulary, and summary counters.
This phase is where JS/TS false-positive discipline is mirrored in Rust.

It must not parse Rust syntax or run Cargo.

### `cargo_semantic`

Owns `cargo check --message-format=json`, Cargo metadata, ownership resolution,
coverage ledger entries, and rustc diagnostic classification.

It must not decide syntax signal visibility.

### `merge`

Owns combining syntax and semantic evidence by repo-relative file path and
run-level scope.

It must preserve partial evidence. For example, a non-zero Cargo exit or
incomplete Cargo JSON stream can still keep already emitted diagnostics while
semantic clean stays unavailable.

### `artifact`

Owns the final public JSON shape. No phase hand-builds final artifact fragments
outside its protocol structs.

## Migration Plan

This should be done incrementally, without throwing away the tested M6/M7 code.

1. Add a unified Rust CLI crate under `experiments/rust-main`.
2. Move the source-health wrapper/analyzer modules into the unified crate as
   the syntax phase.
3. Move the cargo-oracle modules into the unified crate as the cargo semantic
   phase.
4. Add a policy module that owns shared visibility, mute reasons, and summary
   vocabulary.
5. Add one unified artifact builder.
6. Keep old binaries only as temporary compatibility shims if needed, and mark
   them deprecated in docs.
7. Remove the old standalone surfaces once the unified CLI dogfoods cleanly.

## Migration Ownership Map

The Rust unified analyzer owns Rust evidence assembly for the product artifact.
It replaces any future JS `.mjs` glue that would otherwise merge
`rust-source-health` and `rust-cargo-oracle` outputs.

Current owner mapping:

- `experiments/rust-main/lumin-rust-analyzer/src/main.rs` owns CLI/run
  orchestration only: parse options, run syntax health, ask the targeting
  policy whether the semantic oracle should receive `target_paths`, run the
  oracle, call the typed product artifact builder, and write JSON or stdout. It
  must not own projection policy or artifact shape logic.
- `experiments/rust-main/lumin-rust-analyzer/src/oracle_targeting.rs` owns the
  AST-to-oracle wake-up rule: review-visible AST macro-expansion and `cfg`
  opacity become `target_paths`, while muted-only syntax evidence and style-only
  review signals do not wake the compiler oracle.
- `experiments/rust-main/lumin-rust-analyzer/src/product_artifact/*` owns the
  typed top-level Rust product artifact assembly: schema/meta/phase refs,
  phase timing briefs, raw lane placement, and the final `UnifiedArtifact`
  shape that replaces would-be JS `.mjs` merge glue.
- `experiments/rust-main/lumin-rust-analyzer/src/product_files/*` owns per-file
  product projection: merging syntax status, semantic findings, semantic
  diagnostics, capped examples, and `files[*].oracleBridge` provenance.
- `experiments/rust-main/lumin-rust-analyzer/src/product_summary.rs` owns the
  typed top-level summary counts and links them to the same artifact fields
  that policy and file projections consume.
- `experiments/rust-main/lumin-rust-analyzer/src/policy.rs` is a module index.
  `experiments/rust-main/lumin-rust-analyzer/src/policy/*` owns the Rust product
  projection policy that replaces would-be JS `.mjs` artifact glue: syntax
  signals get candidate confidence, raw AST lanes are not embedded, and AST
  opaque surfaces are exposed as review/muted summaries plus capped examples.
  It also owns the Rust action-tier projection that mirrors
  `_lib/ranking.mjs`: `SAFE_FIX` requires `safeAction.proofComplete === true`
  and empty selected-action plus finding-level action blockers, while
  `REVIEW_FIX` is explicit selected-action blockers plus verified/rule-backed
  diagnostics without safe edit proof. Muted, degraded, and unavailable
  evidence stays separate. It owns `oracleBridge`, which joins AST opaque
  surface counts to Cargo/rustc coverage status without changing fix tiers. It
  also owns the product artifact contract guard that mirrors
  `_lib/rust-topology-prefer.mjs`: summary/action-policy drift,
  summary/oracle-bridge drift, file/oracle-bridge drift, missing file bridge
  provenance, missing semantic finding provenance, and raw lane embedding are
  hard-stop failures before artifact write.
- `experiments/rust-sidecar/rust-source-health/src/signals.rs` remains the
  temporary Rust owner for syntax signal review/mute policy until that phase is
  moved into `lumin-rust-analyzer`.
- `experiments/rust-sidecar/rust-source-health/src/analyzer.rs` remains the
  temporary Rust owner for AST fact extraction: definitions, use trees, macro
  calls, `cfg` gates, and opaque surfaces that require oracle follow-up,
  including the practical review/muted classification for macro and `cfg`
  opacity.
- `experiments/rust-main/rust-cargo-oracle/src/artifact.rs` remains the
  temporary Rust owner for Cargo coverage, findings, diagnostics, and clean
  evidence until that phase is moved into `lumin-rust-analyzer`. It also owns
  the current Rust `safeAction` and `actionBlockers` construction for rustc
  `MachineApplicable` rule-backed warning suggestions. This mirrors
  `_lib/export-action-safety.mjs`: selected-action blockers block SAFE_FIX,
  while stronger unselected action blockers stay explanatory. Metadata-only
  mode deliberately emits unavailable Cargo-check coverage with
  `streamParseStatus: "not-run"` instead of pretending the skipped oracle proved
  a clean build.
- `experiments/rust-main/rust-cargo-oracle/src/driver.rs` owns one semantic
  oracle run: metadata loading, package selection, `cargo check` execution, and
  optional artifact writing.
- `experiments/rust-main/rust-cargo-oracle/src/target_selection.rs` owns
  targeted Cargo-check package selection: it maps `target_paths` to the nearest
  Cargo package root, ranks packages by relevance, and preserves that order for
  package-scoped Cargo checks.
- `experiments/rust-main/rust-cargo-oracle/src/oracle_plan.rs` owns `oraclePlan`
  evidence: selected packages, selected path examples, unmatched paths, run
  status, and the reason the oracle did or did not run.
- `experiments/rust-main/rust-cargo-oracle/src/lib.rs` is the public facade and
  re-export boundary for the temporary semantic phase. It must stay thin.
- TS/JS policy modules such as `_lib/finding-provenance.mjs`,
  `_lib/ranking.mjs`, `_lib/pre-write-cue-tiers.mjs`, and
  `_lib/block-clone-artifact.mjs` are precedent, not runtime owners, for Rust.
  Rust must mirror their evidence vocabulary: `review`, `muted`,
  `candidate`, `rule-backed`, `verified`, and `unavailable`.

The old standalone Rust crates are compatibility/internal phase surfaces during
migration. They are not the final product boundary.

## Testing Strategy

Tests must prove product behavior, not module existence.

Required tests:

- A temp Rust repo with only syntax issues produces syntax review signals in
  the unified artifact.
- Test/generated Rust files are muted through the shared policy vocabulary.
- A repo with a Cargo `E0308` error produces a verified semantic finding in the
  same artifact as syntax evidence, counted through
  `actionPolicy.semanticReview` rather than as a vague fallback bucket.
- A repo with AST opaque surfaces and partial Cargo coverage exposes that link
  through `oracleBridge` without clearing review evidence.
- File projections expose `files[*].oracleBridge` with parser provenance,
  Cargo/rustc support evidence, local opacity taint, coverage taint, and
  `oracleConfidence`, matching the TS/JS `finding-provenance` pattern.
- Semantic findings expose finding-local provenance. A file with review opaque
  surfaces does not automatically taint every finding in that file; only
  overlapping AST opaque surfaces, semantic action blockers, parser failure, or
  oracle coverage gaps enter `semanticFindings[*].taintedBy`.
- A warning-only repo can have rule-backed lint findings while semantic clean
  remains true for verified rustc error absence.
- Rustc machine-applicable rule-backed warning suggestions promote to
  `SAFE_FIX` when the selected edit proof is complete and blockers are empty;
  macro-expanded, invalid range, and overlapping-edit suggestions remain
  `REVIEW_FIX` with `actionBlockers[]`.
- Product artifact contract validation accepts the projected artifact shape and
  rejects summary/action-policy drift, raw syntax AST/signals, and full raw
  phase payload embedding before write.
- Metadata unavailable keeps root user-code diagnostics when ownership can be
  resolved conservatively, while absence-clean remains unavailable.
- Non-zero Cargo exits and incomplete Cargo JSON streams preserve already
  emitted diagnostics and block clean coverage.

No file-existence tests. No tests that only prove a module was created.

## Non-Goals

- Do not add new `.mjs` wrappers for Rust source health.
- Do not keep TS/JS and Rust false-positive policy as two unrelated policy
  systems.
- Do not make Cargo clean evidence a product claim without syntax context.
- Do not require cache reuse before the analysis input set is complete.

## Open Follow-Up

The TS/JS policy vocabulary still lives mostly in JS modules and tests. The
first Rust unification slice should mirror the vocabulary needed for Rust
artifacts, not attempt a full cross-language policy crate in one jump.

The bridge should be vocabulary-compatible first. Code sharing can come later
only if it reduces maintenance rather than creating a third abstraction.
