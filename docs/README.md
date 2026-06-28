# Docs Map

This directory is the maintainer-facing map for documentation that
supports the grounded audit skill without expanding the public entry
surface.

If you are entering the repo for the first time, start with the public
contract:

- `README.md`
- `SKILL.md`
- `audit-repo.mjs`
- `.claude-plugin/plugin.json`
- `commands/`
- `canonical/` runtime spine
- `templates/`
- `references/`

If you are navigating the documentation tree itself, use these staging
areas:

- [product surface map](product-surface.md) — current public vs
  internal vs lab boundary
- [internal engine map](internal-engine.md) — grouped role map for
  root engine scripts that remain available for debugging and repros
- [history staging area](history/README.md) — closed phase notes,
  retrospectives, and session history
- [Lumin wiki](lumin-wiki/index.md) — maintainer synthesis layer for
  workstreams, evidence concepts, and test reform rules. It summarizes
  source-of-truth docs/tests/lab notes; it is not a public skill surface.
- [spec staging area](spec/README.md) — maintainer-facing design
  references; user-facing templates and operating references live at
  `templates/` and `references/`
- [maintainer notes](maintainer/README.md) — dogfood-only operating
  notes that should not ship as user-facing skill templates
- [lab staging area](lab/README.md) — reproducible drafts, benchmark
  corpora, and local evidence stores

The intent is simple: keep the public surface small, keep design
history discoverable, and keep reproducible lab artifacts available
without treating them as onboarding entrypoints.
