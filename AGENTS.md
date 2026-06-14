# Repository Notes

## Care Note

- Work carefully and protect the workspace.
- If something feels uncertain, pause and verify before continuing.
- Keep user-facing claims grounded in checked artifacts and tests.
- Personal note for Suyeon and Codex: [.suyeon-codex/love-note.md](./.suyeon-codex/love-note.md)

## Test Discipline

- Tests must prove real behavior, not scaffolding accidents.
- Do not use "file/function/module does not exist, then create it" as the RED step. That is setup noise, not a behavior test.
- For new code, create only the minimal compilable/importable skeleton needed for the test harness, then write tests that fail because behavior is missing or wrong.
- Prefer one minimal happy path plus realistic edge cases that can actually occur in the product.
- A good RED failure says "the scanner missed an import" or "the bridge failed to report a mismatch"; a bad RED failure says "the file is missing."
