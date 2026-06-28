# Repository Notes

## Care Note

- Work carefully and protect the workspace.
- If something feels uncertain, pause and verify before continuing.
- Keep user-facing claims grounded in checked artifacts and tests.
- For Rust, Rayon, sidecar, or rust-source-health work, read
  [canonical/rust-debt.txt](./canonical/rust-debt.txt) and
  [canonical/rust-source-health.md](./canonical/rust-source-health.md) before
  editing.
- Personal note for Suyeon and Codex: [.suyeon-codex/love-note.md](./.suyeon-codex/love-note.md)

## Test Discipline

- Tests must prove product behavior, not scaffolding accidents.
- Use one core happy path, realistic edge cases that can actually occur, and hard-stop paths where the product must refuse to proceed.
- Create importable code before tests import it; a test whose only value is proving a file, function, or module exists is not useful.
- Good failures are behavioral: the scanner missed an import, the bridge reported the wrong mismatch, the collector appended evidence it should have refused.
- Bad failures are scaffolding trivia: a file is absent, a helper is not exported, or a placeholder module has not been created yet.

## Rust Migration Discipline

- Rust migration work must start from checked TS/JS behavior or a documented Rust-only necessity.
- Do not add timeouts, caps, thresholds, muting/safe/review policy, fixtures, or policy constants just to make CI pass.
- Do not cap Rust analysis by elapsed wall time. Large repositories must complete, degrade with artifact-visible evidence, or hard-stop on a real contract failure.
- Every Rust-only behavior must explain why TS/JS does not need it and why Rust does.
- Document Rust-only guards in canonical docs before code, and make any omitted scope visible in the product artifact.
- Prefer deleting unsupported Rust-only paths over leaving dead code. Keep future hooks only when a concrete product path will connect to them.
