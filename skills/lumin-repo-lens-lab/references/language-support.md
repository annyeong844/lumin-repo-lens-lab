# Language Support

Use this reference when wording precision, blind zones, or marketplace
claims by language.

## TypeScript, TSX, And JavaScript

Primary support is TypeScript/TSX/JS through oxc-parser and selected
TypeScript compiler APIs.

Current strengths:

- import/export graph construction
- JSX reference counting
- scoped `tsconfig` path discovery
- workspace fallback aliases
- Node `#imports` subpath and suffix wildcard handling
- direct namespace and dynamic-import member precision
- generated/framework policy sentinels where configured

Known boundaries:

- local binding is not a full TypeScript checker-grade symbol binder
- public API protection is still partly file-level
- aliased, computed, or dynamic-path namespace uses degrade
  conservatively
- checker-grade binding is opt-in/future work, not the default path
- shared AST caching is not implemented yet

Do not claim "perfect TypeScript semantic analysis." Say TypeScript is
the primary target with explicit precision boundaries.

## Python

Python has L1 support through a Python subprocess using stdlib `ast`.
It covers topology, discipline, and symbol graph extraction when
`python3` or `python` is on PATH and the repo contains `.py` files.

Supported:

- absolute imports
- relative imports
- package submodules
- `__init__.py` package entries
- `__all__`
- common decorators and dunder policy

Boundary:

- dynamic method resolution remains blind
- `__getattr__` lazy export maps may over-report

## Go

Go has L1 support through tree-sitter WASM.

Supported:

- `go.mod` module path mapping
- package import mapping to directory-level package files
- selector-expression uses such as `pkg.Symbol`
- triage/topology/discipline counting alongside JS/TS/Python

Boundary:

- within-package plain references are not fully tracked
- `main` entry functions can appear unconsumed, similar to TS entry files

## Rust

Rust files are counted by triage and surfaced in manifest language and
blind-zone evidence. The JS/TS symbol graph does not own Rust absence
claims.

Supported in the generic audit route:

- `.rs` file counting in `triage.json.shape.rustFiles`
- `byLanguage.rs` and manifest language preservation
- Cargo root detection through `triage.json.buildSystem.rust`
- `manifest.json.blindZones[]` entry with `area: "rust"` when the Rust
  analyzer artifact is not registered for that audit run
- opt-in `audit-repo.mjs --rust-analyzer`, which writes
  `rust-analyzer-health.latest.json` and records `manifest.rustAnalysis`,
  including `manifest.rustAnalysis.scanScope` copied from the native Rust
  artifact's source-health input metadata

Boundary:

- read the unified `lumin-rust-analyzer` artifact before making Rust
  syntax, semantic, dead-definition, clone, or absence claims
- use `manifest.rustAnalysis.scanScope` as the quick range cue, but use the
  native Rust artifact for exact Rust evidence
- do not use JS/TS `symbols.json` absence as Rust evidence
- if `manifest.rustAnalysis.status !== "complete"`, keep Rust claims at
  scan-range/blind-zone level

## Other Languages

Java, C#, Ruby, PHP, C++, and Bash grammars may exist through tree-sitter
packages, but extractors are not registered. Treat these as scan gaps unless
a project-specific extractor has been added.
