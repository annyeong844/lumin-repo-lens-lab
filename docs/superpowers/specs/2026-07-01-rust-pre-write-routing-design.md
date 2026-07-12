# Rust Pre-Write Routing Migration

## Goal

Move the pre-write engine-selection policy out of `audit-repo.mjs` and into
`lumin-audit-core`, without migrating JS/TS pre-write producer semantics.

This is an orchestration slice, not a source-language analyzer slice.

## Checked JS Behavior

The existing JS wrapper performed these decisions before running a pre-write
child:

- Read the intent JSON from the requested intent flag or stdin.
- Reject malformed JSON before choosing an engine.
- Accept only `language: "rust"` and `language: "js-ts"` when the field is
  present.
- `--pre-write-engine js` rejects Rust intents.
- `--pre-write-engine rust` rejects JS/TS intents.
- `auto` routes Rust intents to the Rust engine and missing-language intents to
  the JS engine.
- The route-only `language` field is stripped before forwarding Rust intent
  stdin.
- File-backed JS intents keep their original file path as the child intent flag;
  stdin-backed JS intents keep stdin input.

## New Rust Owner

`experiments/rust-main/lumin-audit-core/src/pre_write_routing.rs` owns the typed
projection from:

- requested engine: `auto`, `js`, or `rust`
- original intent flag
- already-read intent JSON text

to:

- selected engine
- child intent flag
- optional child intent stdin
- engine-selection evidence (`requested`, `selected`, `reason`,
  `intentLanguage`)

The CLI surface is:

```text
lumin-audit-core pre-write-route --input <path|->
```

The request schema is `lumin-pre-write-routing-request.v1`.

## Boundary

Rust owns route policy only. JS still owns original intent file/stdin reading
because `audit-repo.mjs` is still the top-level wrapper and the audit-core CLI
already receives its own request through stdin.

Rust must not use this module to:

- reinterpret JS/TS `pre-write.mjs` advisory semantics
- walk source inventory
- read scan-scope producer artifacts
- run child processes
- write the final manifest

Those belong to separate owners:

- Rust pre-write child execution: `pre_write_lifecycle.rs`
- JS/TS pre-write producer semantics: `pre-write.mjs`
- final manifest assembly: `audit-repo.mjs`

## Product Checks

The migration is covered by product-behavior checks:

- Rust auto route strips route-only `language` and selects the Rust engine.
- Missing language still defaults to JS without rewriting intent text.
- Explicit JS file intent preserves the file child flag and does not invent
  stdin.
- Explicit engine/language mismatch hard-stops.
- The CLI emits typed JSON and rejects malformed request shape.

No new timeouts, repository-size caps, or policy thresholds are introduced.
