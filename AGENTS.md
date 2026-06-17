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
