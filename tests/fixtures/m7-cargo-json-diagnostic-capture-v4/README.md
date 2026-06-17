# M7 Cargo JSON Diagnostic Capture v4

Purpose: empirical cargo `--message-format=json` evidence for M7 cargo diagnostic classification.

This packet is not a repository implementation patch. It is a review/evidence bundle and Rust test corpus for the cargo diagnostic classifier contract.

## Included

- Controlled fixture crates under `fixtures/`.
- Raw cargo stdout JSONL and stderr for each run.
- Portable `runs.json` using paths relative to the capture root.
- Fixed `toolchain.json` captured at collection time.
- `capture-root.env` with redacted packet root and raw-capture provenance note.
- `summary.json` with raw code shape, normalized derived fields, classifier output, stream parse status, coverage entries, scoped clean semantics, primary span classes, and toolchain metadata.
- `summary.md` with the human-readable result table.
- `verification.txt` with commands and limitations.

## Classifier Behavior Captured

- `failure-note`, `note`, and `help`-style diagnostics are non-findings, not candidate fallback material.
- Non-user-code primary diagnostics are not user-facing findings; non-user-code primary errors make `absence-clean` coverage unavailable.
- Denied rustc lint can be `level:error`; non-E code names must be classified before level-based verified classification.
- Rule-backed lint findings use `rule-backed.rust.rustc-lint-diagnostic`, not a warning-only claim name.
- Codeless rustc errors are represented as `code: null`, not omitted code fields.
- E-code rustc errors are represented as `code.code` matching `^E[0-9]+$`.
- `build-finished { success: true }` plus complete JSONL parsing is required for absence/clean coverage.
- `absence-clean.clean` means only absence of verified rustc error claim kinds for the scoped package/target/features/profile; rule-backed findings can coexist.
- Feature-gated code confirms coverage must include feature set.

## Path Policy

Raw cargo JSONL intentionally preserves cargo's original Windows path spelling. `summary.json` keeps fixture-local diagnostic spans because path spelling is part of the classifier contract. `runs.json` itself is portable and uses capture-root-relative paths.

## Production Caveat

The original capture summarizer was fixture-aware evidence code, not production code. Production user-code ownership must use cargo metadata/package ownership, selected workspace members, and resolved source roots.
