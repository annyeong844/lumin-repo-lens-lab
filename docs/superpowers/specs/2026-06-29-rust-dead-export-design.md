# Rust Dead Export Analysis Design

## Objective

Port the checked TS/JS dead-export model into the Rust source-health lane without
copying TS assumptions that do not hold for Rust.

The goal is not to add a "zero references means delete it" lint. That rule is
wrong for Rust. The goal is to produce proof-carrying unused-definition and
dead-export evidence with explicit false-positive gates, action-safety blockers,
and artifact-visible scan limits.

## Source Model

The TS/JS producer has three useful pieces to preserve:

- reachability: exported identity plus observed consumers
- false-positive gates: public API, barrel, dynamic import, test consumer,
  framework, config, and entrypoint exclusions
- action safety: deadness and safe edit proof are separate

Rust keeps that skeleton, but the gates are language-specific. Rust `pub` does
not mean "locally used only if this repository references it." It can be an
external crate API, a trait contract surface, a macro-visible symbol, an FFI
entrypoint, or a cfg-specific symbol that this syntax pass did not see in the
active branch.

## Current Rust Inputs

`rust-source-health` already emits the first layer of syntax evidence:

- `ast.definitions[]`: named item definitions with `kind`, `name`,
  `visibility`, and `location`
- `ast.pathRefs[]`: qualified expression- and type-position path references
- `ast.useTrees[]`: import and re-export syntax
- `ast.impls[]`: impl blocks, trait paths, and method owner evidence
- `ast.macroCalls[]`, `ast.cfgGates[]`, and `ast.opaqueSurfaces[]`: places where
  AST-only analysis must not claim semantic certainty

Those facts are not yet enough for deletion or demotion. The dead-export lane
needs additional fact enrichment before producing review-visible candidates:

- stable definition identity: file, module path if available, kind, name, span
- owner context: top-level, module, impl block, trait impl, inherent impl
- attribute context: `no_mangle`, `export_name`, `link_name`, `extern`,
  `derive`, custom attributes, `cfg`, and test attributes
- re-export exposure: `pub use` alias/glob evidence and crate-root surface
- scan completeness: parse errors, skipped files, opaque surfaces, and cargo
  package scope status

## Measured Risk

A naive ripgrep experiment counted Rust `pub` and `pub(crate)` definitions with
zero observed `pathRefs` inside the analysis artifact:

| Measurement | Count |
|---|---:|
| all definitions | 3379 |
| `pub` + `pub(crate)` definitions | 889 |
| zero observed references among `pub` + `pub(crate)` | 668 |

That is a 75% naive candidate rate. Samples include public builders and matcher
APIs that are intentionally exported. This proves the important product rule:

```text
Rust public visibility is not deadness evidence.
```

Public Rust items must never become remove candidates from syntax reachability
alone. At most they become review or demotion candidates, and only with
artifact-visible blockers.

## Ownership

`rust-source-health` owns raw unused-definition evidence because it already owns
AST facts, visibility, path policy, and opaque syntax evidence.

Planned source-health owner:

- `src/dead_exports.rs`
- implementation detail modules under `src/dead_exports/`

`lumin-rust-analyzer` owns product policy projection, action tiers, and any
future safe-action surface. It must consume typed Rust source-health evidence;
it must not recompute reachability from JSON strings.

The first code change must add protocol types before emitting findings. Do not
hand-build `serde_json::Value` surfaces for this lane.

## Candidate Tiers

Rust mirrors the TS proposal shape, but with stricter action safety:

| Tier | Meaning | Safe action |
|---|---|---|
| `remove-candidate` | non-public item has zero observed references under complete local evidence | blocked until edit proof exists |
| `demote-to-restricted` | `pub` appears used only inside the current crate/package | always review unless a Rust public-surface oracle clears external API risk |
| `review` | deadness is plausible but gated by trait, macro, cfg, re-export, or incomplete scope evidence | none |
| `degraded` | parse errors, skipped files, opaque overload, missing package scope, or unsupported owner shape prevents a claim | none |
| `muted` | test-only or generated-only evidence not product-actionable | none |

The first implementation slice should not emit `SAFE_FIX`. It may emit review
evidence and blocked action candidates. `SAFE_FIX` requires a later edit-proof
layer equivalent to TS `export-action-safety.mjs`.

## Rust False-Positive Gates

Rust gates use a separate namespace from TS gates.

### RUST-FP-A: External Crate Public Surface

Any externally reachable `pub` item is not a remove candidate. This includes
crate-root exports, `pub use` re-exports, public library targets, and package
surface that may be consumed outside the current repository.

Action:

- block remove
- allow only review/demote evidence
- include `actionBlockers: ["rust-fp-a-external-public-surface"]`

### RUST-FP-B: Trait Impl And Trait Contract Surface

Trait impl methods can be called through trait objects, generic bounds, method
resolution, and downstream crates without a direct `pathRef` to the method name.

Action:

- block remove for trait impl methods
- review inherent impl methods until owner and method-call evidence are complete
- include trait path and impl target evidence when present

### RUST-FP-C: Macro And Opaque Syntax

Macros can generate definitions or references that AST-only analysis cannot see.
Definitions inside, adjacent to, or named inside review-visible opaque macro
surfaces must not be treated as cleanly dead.

Action:

- demote to review or degraded
- include relevant `opaqueSurfaces[]` / `macroCalls[]` examples

### RUST-FP-D: FFI And Linker Surface

Items with `#[no_mangle]`, `#[export_name]`, `#[link_name]`, `extern "C"`, or
similar linker-facing attributes may be used outside Rust source code.

Action:

- exclude from remove candidates
- serialize blocker evidence

### RUST-FP-E: Derive And Generated Trait Requirements

Derive macros and trait contracts may require items or fields that are not
referenced by ordinary path refs.

Action:

- review only when derive/trait context touches the candidate
- do not auto-remove derived public type surface

### RUST-FP-F: Cfg-Gated Definitions

Definitions under `cfg` or `cfg_attr` may be live for build targets/features
not represented by the current syntax pass.

Action:

- degrade or review
- include cfg expression evidence

### RUST-FP-G: Test-Only Reachability

Items referenced only inside `#[cfg(test)]` or `#[test]` contexts are not clean
production dead-code evidence. They are test-pinned evidence.

Action:

- product action remains blocked
- report test-only support separately from production support

### RUST-FP-H: Generated Source

Generated Rust files mirror the TS/JS generated-source exclusion: repeated or
unreferenced generated helpers are not product cleanup candidates.

Action:

- mute generated-source candidates
- keep the blocker visible as `rust-fp-h-generated-source`

### RUST-FP-I: Rust Entrypoints

Cargo build scripts and binary `main.rs` entrypoints are called by Cargo or the
runtime without ordinary Rust path references.

Action:

- block remove for `build.rs` `main` and `main.rs` `main`
- serialize `rust-fp-i-rust-entrypoint`

## Reachability Model

Rust reachability must be separated by evidence scope:

- same-file references
- same-module references
- crate-local references
- re-export exposure
- test-only references
- opaque or generated references
- unsupported external reachability

The artifact must say which scopes were searched. A count of zero only means
"zero observed references in searched scopes." It must not be phrased as
"definitely unused" when external, macro, cfg, or trait scope is unavailable.

## First Implementation Slice

The first Rust slice should be deliberately narrow:

1. Enrich definition facts with owner context, attributes, cfg/test context, and
   stable spans.
2. Build a typed raw analysis surface under `rust-source-health`.
3. Emit review/degraded unused-definition evidence with RUST-FP blockers.
4. Do not emit `SAFE_FIX`.
5. Do not remove or demote `pub` items automatically.

The first product-positive candidate may be a non-public item only if all of
these are true:

- the definition has a supported owner shape
- it is not `pub`, `pub(crate)`, or restricted-public
- it is not an impl trait method or trait contract item
- it has no FFI/linker/export attributes
- it is not under cfg/test/generated/opaque syntax
- it is not a Rust entrypoint such as `build.rs` `main` or `main.rs` `main`
- it is a supported module-owned function, const, or static
- the containing file parsed successfully
- all skipped-file and package-scope limits are visible in the artifact
- zero references are observed in the supported local scope

Even then, safe action remains blocked until edit proof exists.

## Artifact Shape

Proposed raw source-health surface:

```json
{
  "unusedDefinitionAnalysis": {
    "policy": {
      "policyId": "rust-unused-definition-policy-v1",
      "tsModel": "dead-export-reachability-plus-action-safety",
      "rustFpGateNamespace": "RUST-FP",
      "candidateCountScope": "observed-references-in-supported-rust-syntax-scopes",
      "safeActionScope": "none-without-edit-proof"
    },
    "summary": {
      "definitionCount": 3379,
      "candidateCount": 0,
      "reviewCount": 0,
      "degradedCount": 0,
      "blockedPublicSurfaceCount": 0,
      "blockedTraitImplCount": 0,
      "blockedOpaqueCount": 0,
      "blockedCfgCount": 0,
      "blockedFfiCount": 0,
      "testOnlySupportCount": 0
    },
    "findings": [],
    "excludedCandidates": [],
    "degradedScopes": []
  }
}
```

Candidate shape:

```json
{
  "kind": "rust-unused-definition",
  "tier": "review",
  "action": "demote-to-restricted",
  "definition": {
    "file": "src/lib.rs",
    "name": "Thing",
    "kind": "struct",
    "visibility": "public",
    "owner": "module"
  },
  "observedReferences": {
    "production": 0,
    "testOnly": 0,
    "searchedScopes": ["crate-local-name-and-qualified-path-refs"]
  },
  "fpGates": ["RUST-FP-A"],
  "actionBlockers": ["rust-fp-a-external-public-surface"],
  "safeAction": null,
  "evidence": []
}
```

Names may change during implementation, but these semantics must not:

- deadness evidence is separate from edit safety
- public Rust surface blocks removal
- unsupported scope is serialized, not silently ignored
- candidate counts are not grounded absence claims outside searched scopes

## Product Projection

`lumin-rust-analyzer` may later map raw evidence into action policy tiers:

- `remove-candidate` with complete local proof and edit proof -> possible
  `SAFE_FIX` in a later slice
- any missing edit proof -> `REVIEW_FIX`
- any RUST-FP blocker -> `REVIEW_FIX` or `DEGRADED`
- public surface blocker -> no safe action

This mirrors TS ranking: a deadness claim does not imply the edit is safe.

## Tests

Tests must prove product behavior, not that helper modules exist.

Required fixtures:

1. private unused item, complete local scope -> review/remove candidate with
   `safeAction: null`
2. public API item with zero local refs -> no remove candidate, RUST-FP-A blocker
3. trait impl method with zero path refs -> RUST-FP-B blocker
4. `#[no_mangle]` / `#[export_name]` item -> RUST-FP-D blocker
5. cfg-gated item -> RUST-FP-F degraded/review
6. macro-adjacent or opaque item -> RUST-FP-C degraded/review
7. test-only reference -> production count stays zero but test-only support is
   visible and action is blocked
8. parse/skipped-file incomplete artifact -> no grounded absence claim

Dogfood checks:

- ripgrep: naive `pub` zero-ref candidates must be blocked or reviewed, not
  remove candidates
- clap/serde: builder/public API patterns must not become remove candidates
- codex-rs: large workspace must complete without wall-time timeout or repo-size
  cap; any omitted scope must be artifact-visible

## Non-Goals

- no Rust dead-export `SAFE_FIX` in the first slice
- no wall-time timeout
- no repository-size cap
- no public `pub` removal from syntax reachability alone
- no JSON stringly-typed policy reconstruction in `lumin-rust-analyzer`
- no fake fixtures that only prove module plumbing

## Open Questions

1. Whether package public surface should be owned by `rust-cargo-oracle`,
   `rust-source-health`, or `lumin-rust-analyzer`.
2. Whether `pub(crate)` demotion/removal should wait for a typed module tree.
3. Whether the first slice should expose only raw source-health evidence, with
   product action projection deferred until dogfood false positives are
   reviewed.

Recommended answer for the first implementation: defer safe actions, expose raw
review/degraded evidence, and validate RUST-FP gates on real repositories before
promoting anything into cleanup policy.
