# Maintainer Notes

This directory holds maintainer-only operating notes for the
`lumin-repo-lens-lab` repo itself. Files here are not part of the
deployable skill package and should not be loaded for ordinary user
repositories.

Use these notes only when dogfooding or changing the skill internals.

- `CLAUDE_MD_POLISH_BRIEF_2026-04-28.md` — guardrails for asking Claude
  Code to polish Markdown without changing the engine contract.
- `false-positive-patterns-ledger.md` — long historical FP case ledger.
  The shipping skill uses `references/false-positive-index.md` instead.
