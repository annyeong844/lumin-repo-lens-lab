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
    "verifiedSemanticFindings": 0,
    "semanticClean": {
      "status": "ran",
      "clean": true,
      "cleanKind": "verified-rustc-error-absence"
    },
    "cacheReuse": {
      "status": "not-reusable"
    }
  },
  "files": {
    "src/lib.rs": {
      "syntax": {
        "signals": []
      },
      "semantic": {
        "diagnostics": []
      }
    }
  },
  "coverage": [],
  "semanticFindings": []
}
```

The exact schema can evolve, but the product invariant cannot: one run, one
artifact, shared vocabulary.

## Policy Model

Rust must not invent a second false-positive policy layer.

Shared concepts should be named once:

- visibility: `review`, `muted`
- mute reasons: test path, generated path, cfg-test, test attribute, dependency
  scope, unavailable semantic scope
- confidence tiers: syntax-only candidate, rule-backed, verified, unavailable
- clean scope: absence of verified rustc error diagnostics for the declared
  Cargo-check scope

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

It must preserve partial evidence. For example, Cargo timeout can still keep
already emitted diagnostics while semantic clean stays unavailable.

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

## Testing Strategy

Tests must prove product behavior, not module existence.

Required tests:

- A temp Rust repo with only syntax issues produces syntax review signals in
  the unified artifact.
- Test/generated Rust files are muted through the shared policy vocabulary.
- A repo with a Cargo `E0308` error produces a verified semantic finding in the
  same artifact as syntax evidence.
- A warning-only repo can have rule-backed lint findings while semantic clean
  remains true for verified rustc error absence.
- Metadata unavailable keeps root user-code diagnostics when ownership can be
  resolved conservatively, while absence-clean remains unavailable.
- Timeout preserves already emitted diagnostics and blocks clean coverage.

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
