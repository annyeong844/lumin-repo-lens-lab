# M6 Rust Source Health Design

Date: 2026-06-16

## Decision

M6 should add a new lab-only Rust source health track.

The correct first parser is `ra_ap_syntax`. It is the rust-analyzer syntax tree
library, and this track is about Rust source syntax health. `oxc-parser` stays
where it already belongs: JavaScript and TypeScript. `tree-sitter` stays where
it already works: the existing WASM language extraction path. The M2-M5 Rust
topology sidecar stays where it belongs: a Rust implementation of the JS/TS
module-edge scanner policy.

M6 is addition, not replacement.

## Current Parser Landscape

The repo already has multiple parser and scanner surfaces:

- JS/TS production analysis uses `oxc-parser` through `parseOxcOrThrow`.
- Go and other tree-sitter-backed languages use the existing WASM
  `tree-sitter-langs.mjs` path.
- M2-M5 `experiments/rust-sidecar/topology-scanner` is a Rust sidecar for the
  JS/TS module-edge scanner policy. It is not a Rust source analyzer.

Those surfaces are assets. M6 must not delete, replace, or quietly reroute
them.

## Goal

Create the design for a Rust source health analyzer that:

- reads Rust source files;
- parses them with `ra_ap_syntax`;
- reports syntax-level facts and review signals;
- writes a separate `rust-health.json` artifact;
- stays lab-only;
- does not claim semantic, type, trait, or borrow-check truth.

The first win is not a large rule set. The first win is a clean product-grade
vertical slice with boundaries that can survive future signals.

## Non-Goals

M6 must not:

- remove or replace JS/TS `oxc-parser` parsing;
- remove or replace tree-sitter WASM language extraction;
- remove or replace the M2-M5 Rust topology sidecar;
- change topology, symbol graph, prefer, quorum, cache, SARIF, or markdown
  artifact contracts;
- route Rust source health findings into existing risk gates by default;
- claim semantic/type/borrow-check truth;
- run `cargo check`, rust-analyzer HIR, or rustc internals;
- add broad speed claims.

If a future step needs semantic truth, that is M7 or later.

## Why `ra_ap_syntax`

`ra_ap_syntax` is the right M6 core because it gives Rust syntax trees from the
rust-analyzer ecosystem without pulling in full semantic analysis.

Rejected first-core options:

- `oxc-parser`: excellent JS/TS parser; wrong language for Rust source health.
- `tree-sitter-rust`: useful and already adjacent to the repo, but not the
  best long-term Rust-specific health spine.
- `syn`: strong for procedural macro style parsing, but not the best
  repo-wide full-fidelity health boundary.
- `ra_ap_hir` or rust-analyzer IDE crates: too much semantic surface for M6.
- `rustc_private`: accurate but heavy, nightly-bound, and closer to compiling
  than scanning.
- `cargo check --message-format=json`: practical later diagnostic ingestion,
  but too coarse as the first health analyzer core.

The take: M6 should start with syntax. Do not pretend syntax signals are
semantic truth.

## Architecture

M6 should introduce a separate sidecar:

```text
experiments/rust-sidecar/rust-source-health
```

The shape:

```text
Node lab wrapper or future producer
  -> Rust sidecar: rust-source-health
    -> file collection input
    -> ra_ap_syntax parser adapter
    -> syntax facts
    -> review signals
  -> rust-health.json
```

The parser adapter is a deliberate seam:

```text
source text -> ParsedRustFile -> facts/signals
```

Everything outside that adapter should talk in M6-owned facts, not raw
`ra_ap_syntax` internals. That keeps the first implementation from hardwiring
every downstream decision to one crate's node API.

### Input Protocol

The Node wrapper owns file collection, path policy, hashing, and source reads.
The Rust sidecar must not walk the repository on its own in M6. That keeps
ignore policy, symlink handling, and snapshot provenance in one place.

Sidecar stdin should be JSON-only:

```json
{
  "schemaVersion": 1,
  "root": "/absolute/repo/root",
  "files": [
    {
      "path": "src/lib.rs",
      "sha256": "sha256:...",
      "text": "fn main() {}\n"
    }
  ],
  "pathPolicy": {
    "include": ["**/*.rs"],
    "exclude": ["**/target/**", "**/vendor/**"]
  },
  "parser": {
    "editionPolicy": "fixed",
    "edition": "2021",
    "editionSource": "m6-policy-default"
  },
  "runtime": {
    "threadCount": 2,
    "workerStackBytes": 16777216
  }
}
```

Rules:

- `root` is absolute.
- `files[].path` is root-relative slash form.
- the wrapper reads raw bytes and computes `files[].sha256` over those bytes.
- the wrapper decodes raw bytes as UTF-8 without normalization.
- invalid UTF-8 is recorded as a skipped-file finding with reason
  `invalid-utf8`; it is not silently repaired.
- `files[].text` is the decoded UTF-8 text supplied to the sidecar.
- byte offsets in the output are offsets into the UTF-8 bytes of
  `files[].text`.
- the sidecar may verify `files[].sha256` by re-encoding `files[].text` as
  UTF-8 and hashing those bytes.
- M6 uses a fixed parser edition policy. The product-slice default is Rust
  2021, recorded as `editionPolicy: "fixed"`, `edition: "2021"`, and
  `editionSource: "m6-policy-default"`.
- M6 does not infer per-file editions from Cargo manifests. That would make
  Cargo workspace loading a hidden dependency.
- missing or unsupported input `schemaVersion` is a hard failure.
- stdin is request JSON only; stdout is response JSON only; stderr is
  diagnostic text only.

This is a little stricter than letting the sidecar crawl the tree. Good. The
sidecar is a parser/analyzer, not a second file discovery engine.

The sidecar emits artifact JSON to stdout. The wrapper validates that JSON,
appends wrapper-owned skipped-file evidence for matched files it could not
decode, validates the final artifact, and writes `rust-health.json`. The
sidecar does not accept an output path or write files in the product slice.

The wrapper also owns path policy. Product-slice paths containing a `target` or
`vendor` segment are traversal-pruned and recorded in `meta.input.pathPolicy`
as `**/target/**` and `**/vendor/**`, not expanded into per-file
`skippedFiles` entries. `skippedFiles` is reserved for matched Rust source
files that could not be analyzed after collection, such as invalid UTF-8.

## Artifact Contract

M6 writes a new artifact:

```text
rust-health.json
```

Product contract shape:

```json
{
  "schemaVersion": 1,
  "meta": {
    "generated": "2026-06-16T00:00:00.000Z",
    "producer": "rust-source-health",
    "mode": "syntax-only",
    "parser": {
      "kind": "ra_ap_syntax",
      "version": "<crate version>",
      "editionPolicy": "fixed",
      "edition": "2021",
      "editionSource": "m6-policy-default"
    },
    "sidecar": {
      "name": "rust-source-health",
      "version": "0.1.0",
      "sourceCommit": "<git sha>",
      "binarySha256": "sha256:..."
    },
    "policy": {
      "version": "m6-rust-source-health-syntax-v1",
      "thresholds": {
        "maxFunctionLines": 80,
        "maxImplLines": 200
      }
    },
    "runtime": {
      "threadCount": 2,
      "workerStackBytes": 16777216
    },
    "input": {
      "pathPolicy": {
        "include": ["**/*.rs"],
        "exclude": ["**/target/**", "**/vendor/**"]
      }
    },
    "limits": [
      "syntax-only",
      "no-type-info",
      "no-trait-solving",
      "no-borrow-check"
    ]
  },
  "summary": {
    "files": 0,
    "skippedFiles": 0,
    "parseErrorFiles": 0,
    "parseErrors": 0,
    "functions": 0,
    "unsafeBlocks": 0,
    "unsafeFunctions": 0,
    "signals": 0,
    "signalsByKind": {}
  },
  "skippedFiles": [],
  "files": {}
}
```

Skipped-file evidence is wrapper-owned and should be recorded separately from
the empty contract shape:

```json
[
  {
    "path": "src/not-rust.rs",
    "reason": "invalid-utf8"
  }
]
```

Initial file entries should separate facts from signals:

```json
{
  "src/lib.rs": {
    "facts": {
      "items": 12,
      "functions": 4,
      "maxFunctionLines": 42,
      "unsafeBlocks": 0,
      "unsafeFunctions": 0
    },
    "signals": [
      {
        "kind": "unwrap-call",
        "severity": "review",
        "claim": "syntax-only",
        "location": {
          "line": 18,
          "column": 12,
          "endLine": 18,
          "endColumn": 20,
          "byteStart": 412,
          "byteEnd": 420
        }
      }
    ],
    "parse": {
      "ok": true,
      "errors": []
    },
    "path": {
      "classifications": ["source"],
      "suppressed": false
    }
  }
}
```

Malformed Rust source is still a valid M6 artifact when the parse failure is
recorded as data:

```json
{
  "src/lib.rs": {
    "facts": {
      "items": 0,
      "functions": 0,
      "maxFunctionLines": 0,
      "unsafeBlocks": 0,
      "unsafeFunctions": 0
    },
    "signals": [],
    "parse": {
      "ok": false,
      "errors": [
        {
          "message": "expected expression",
          "claim": "syntax-only",
          "location": {
            "line": 7,
            "column": 14,
            "endLine": 7,
            "endColumn": 14,
            "byteStart": 133,
            "byteEnd": 133
          }
        }
      ]
    },
    "path": {
      "classifications": ["source"],
      "suppressed": false
    }
  }
}
```

Signals are review prompts, not proof of a bug.

Locations should use both human-readable line/column and byte ranges. Byte
ranges make regression checks and editor integrations less fragile; line/column
makes the artifact usable by a reviewer.

Output ordering is deterministic:

- `files` keys are sorted by root-relative path.
- each file's `signals` are sorted by `location.byteStart`, then `kind`.
- `skippedFiles` are sorted by root-relative path.

## Initial Signals

M6 should start small:

- parse errors;
- oversized functions or impl blocks;
- `unsafe` block/function count;
- `.clone()` call count;
- `.unwrap()` call count;
- `.expect(...)` call count;
- `panic!`, `todo!`, `unimplemented!` macro count;
- generated/vendor/test path classification as metadata, not automatic
  suppression.

Do not add scoring in M6. Scores fossilize fast and usually lie early.

Signal caveats must be explicit:

- `.unwrap()` means a syntactic method call whose method token is `unwrap`. It
  does not prove the receiver is `Option`, `Result`, or panic-capable.
- `.expect(...)` has the same limitation as `.unwrap()`.
- `.clone()` means a syntactic method call named `clone`. It does not prove an
  allocation, deep copy, or performance issue.
- `panic!`, `todo!`, and `unimplemented!` are macro-call syntax only. M6 does
  not expand macros or resolve renamed imports.
- `unsafe` counts syntax. M6 does not prove the operation is unsound or sound.

## Error Handling

M6 should fail loudly for tool failures and record syntax failures as data.

Hard failures:

- sidecar binary missing;
- invalid input JSON;
- unsupported input schema version;
- invalid output JSON;
- unreadable root;
- unsupported artifact schema version.

Recorded file-level findings:

- Rust source parse errors;
- unsupported syntax shapes;
- matched files skipped after collection, such as invalid UTF-8.

Do not swallow sidecar errors. If the sidecar cannot run, the M6 artifact should
not pretend to be complete.

Exit contract:

- exit 0 when the sidecar produced a valid `rust-health.json`, even if some
  Rust files contain parse errors recorded as data;
- exit 1 when the sidecar ran but returned invalid output or a hard analysis
  failure;
- exit 2 for wrapper usage/configuration failures such as missing binary,
  invalid input schema, or unreadable configured paths.

## Testing

Tests must be behavior checks, not scaffolding checks.

Required product-slice coverage:

- happy path: a small Rust crate with two functions produces `rust-health.json`
  with parser metadata, facts, and zero hard failures;
- realistic edge case: a file with `unwrap`, `clone`, `unsafe`, and `todo!`
  produces review signals with line numbers;
- syntax edge case: malformed Rust source records parse errors without claiming
  semantic truth;
- hard stop: missing or invalid sidecar output causes the wrapper to fail
  loudly;
- schema check: input and output schema versions are enforced;
- summary invariant check: summary counts match the artifact body:
  `summary.files === Object.keys(files).length`,
  `summary.skippedFiles === skippedFiles.length`,
  `summary.parseErrorFiles` matches files with `parse.ok === false`,
  `summary.parseErrors` matches total parse errors,
  `summary.signals` matches total signals, and `summary.signalsByKind` matches
  signal counts by kind;
- boundary check: M6 does not change topology, prefer, quorum, SARIF, or
  existing JS/TS parser artifacts.

Do not add tests that only prove a file, function, or module exists.

## Rollout

M6 should be lab-only.

First acceptable integration surface:

- a local script or lab producer that runs `rust-source-health`;
- no stable plugin command;
- no public package default behavior;
- no effect on existing audit output unless explicitly requested.

If M6 graduates, it should do so through the same evidence pattern as M2-M5:
compare evidence first, then gate, then explicit use.

## M7 And Later

Possible later tracks:

- per-file Rust edition provenance if corpus evidence shows edition-sensitive
  parsing matters. Do not make Cargo manifest inference a hidden M6 dependency;
- semantic-lite diagnostics through `cargo check --message-format=json`;
- rust-analyzer HIR exploration for selected questions;
- `rustc_private` only if the project deliberately accepts nightly and compile
  frontend cost;
- corpus expansion for real Rust projects;
- calibrated policies for clone/unwrap/unsafe signals.

Those are later. M6 should not sneak them in.

## Success Criteria

M6 design is ready for implementation when review agrees that:

- `rust-source-health` is separate from `topology-scanner`;
- existing `oxc-parser`, tree-sitter, and M2-M5 topology paths remain untouched;
- `ra_ap_syntax` is the first Rust source parser;
- output is a separate `rust-health.json`;
- sidecar input/output schemas are explicit;
- the fixed parser edition policy is recorded in input and artifact metadata;
- artifact policy version and thresholds are explicit;
- invalid UTF-8 handling is deterministic and recorded as skipped-file data;
- wrapper-owned path policy is recorded in artifact metadata;
- output ordering is deterministic;
- signal and parse-error locations include ranges;
- artifact provenance records sidecar source and binary identity;
- signals are syntax-only review prompts;
- semantic analysis is explicitly out of scope;
- tests cover real behavior and hard stops.

That is the product-grade vertical slice. Anything smaller is too vague;
anything bigger is trying to build a compiler frontend, which is not M6.
