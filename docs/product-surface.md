# Product Surface Map

This document defines the intended product surface for the grounded
audit skill while the repository still contains research history and
lab artifacts.

The goal is not to erase evidence. The goal is to keep the public
interface small enough that a user can understand how to enter the
skill without having to reverse-engineer the engine layout.

For the maintainer-facing docs map that ties history, spec, and lab
surfaces together, start at [docs/README.md](README.md).
For the grouped root-script map, use
[docs/internal-engine.md](internal-engine.md).

## Public contract

These are the primary public entrypoints and truth surfaces:

- `SKILL.md`
- `SKILL.write-gate.md`
- `SKILL.canon.md`
- `audit-repo.mjs`
- `package.json`
- `.claude-plugin/plugin.json`
- `commands/`
- `canonical/` runtime spine
- `README.md`
- `templates/`
- `references/`

For distribution, do not zip the maintainer repo directly. Generate
the deployable skill surface with:

```bash
npm run build:skill
```

That writes three shared generated skill surfaces plus one thin Codex
wrapper from the current repo sources:

- `skills/lumin-repo-lens-lab/` — shared engine plus read-only audit
  and refactor-plan instructions
- `skills/lumin-repo-lens-lab-write-gate/` — lean pre-write/post-write transaction
  instructions
- `skills/lumin-repo-lens-lab-canon/` — lean canon-draft/check-canon maintainer
  instructions
- `skills/lumin-repo-lens-lab-codex/` — Codex-native wording over the same
  `skills/lumin-repo-lens-lab/` engine, without Claude Code slash-command
  assumptions

The audit directory contains public wrappers under `scripts/`, internal
runtime implementation under `_engine/`, the runtime canon spine,
templates, and selected references. The sibling skill directories point
back to that shared engine; the Codex wrapper contains no runtime copy.
The generated package intentionally excludes tests, history, lab
artifacts, corpora, generated review outputs, and maintainer self-audit
fact snapshots.

Stable user-facing capabilities:

- `audit`
- `pre-write`
- `post-write`
- `canon-draft`
- `check-canon`

These five are validation modes. They should preserve cold counts,
artifact paths, and scan ranges for users who want the tool as an
evidence generator. Their model-facing instructions are grouped by
lifecycle boundary: audit/refactor-plan, write-gate, and canon.

When loaded as a Claude Code plugin, the same capabilities are exposed
through root-level `commands/*.md` slash-command wrappers. Those
commands are prompt wrappers only; `scripts/audit-repo.mjs` remains the
single execution entrypoint.

The plugin may also expose assistant-facing coaching commands that use
the same audit evidence path without adding a new engine capability.
`refactor-plan` is the first such command: it runs audit evidence, keeps
raw gates internal, and writes a kind incremental refactoring plan. Its
contract is deliberately template-led: the audit supplies facts and the
model authors a semantic phase plan with Phase 1 scope, quick-audit
scope, non-goals, and closeout checks.

This split is the human-in-the-loop boundary: validation modes collect
and preserve evidence; coaching modes interpret it for the next human
decision without hiding the underlying artifacts.

## Internal engine surface

These are important and intentionally retained, but they are engine
entrypoints rather than the preferred user-facing interface:

- root sibling scripts such as `build-symbol-graph.mjs`,
  `measure-topology.mjs`, `classify-dead-exports.mjs`,
  `generate-canon-draft.mjs`, `check-canon.mjs`, `rank-fixes.mjs`
- `_lib/`
- `tests/`
- `scripts/`
- `test-harness/` for maintainer-only skill-triggering checks

Public docs should describe these as implementation and debugging
surfaces, not as the first thing a user should choose from.

## Keep / move / hide

### Keep at root

- `SKILL.md`
- `README.md`
- `audit-repo.mjs`
- `package.json`
- `.claude-plugin/`
- `commands/`
- `canonical/`
- `tests/`
- `scripts/`
- `test-harness/`
- `templates/`
- `references/`

### Keep visible, but describe as internal

- root engine scripts other than `audit-repo.mjs`
- `_lib/`

### Currently staged in history or lab surface

- `docs/history/phases/p1/` through `docs/history/phases/p6/`
- `docs/history/FP-41-regression.md`
- `docs/history/`
- `docs/spec/FP-41-sentinel-spec.md`
- `docs/spec/SPEC-canon-generator.md`
- `docs/spec/`
- `docs/lab/README.md`
- large retrospective or design-session notes that are not part of the
  current shipping contract

Current staging homes:

- `docs/history/` for closed phase notes and retrospectives
- `docs/spec/` for longer-lived design references that still matter to
  maintainers
- `templates/` for output and review templates that ship with the skill
- `references/` for detailed optional operating guides loaded only when
  needed
- `docs/lab/` for reproducible drafts, benchmark corpora, and local
  evidence stores that should not read like shipping entrypoints

### Hide from shipping surface

- `output/` and `review-output*/` (describe via `docs/lab/README.md`,
  not via onboarding)
- `p6-corpus/` (benchmark corpus and dogfood playground)
- generated draft markdown under `canonical-draft/`
- `audit-artifacts/` and `audit-artifacts-smoke/`
- optional local evidence stores such as `.audit/`
- local tool state such as `.claude/`
- live skill-triggering logs under `test-harness/logs/`
- large sample artifacts that exist to support research, dogfood, or
  benchmark work

The repo may still store these while productization continues, but
they should not read like part of the shipping interface.

## Migration order

To keep risk low, the migration should happen in this order:

1. Lock the public surface in docs and acceptance tests.
2. Add the generated skill-package boundary (`npm run build:skill`) so
   the shipping unit can be inspected separately from the maintainer
   repo.
3. Add history/spec destinations before moving large trees.
4. Rehome phase notes and retrospectives.
5. Reduce the visible `output/` / corpus / generated-draft footprint in
   the shipping narrative by anchoring it behind `docs/lab/README.md`.
6. Only then consider physical engine reshaping such as `_lib` →
   `engine/`.

## Non-goals for the next pass

These are intentionally out of scope for the current low-risk pass:

- mass-moving `_lib` into a new `engine/` tree
- renaming every root engine script
- deleting research artifacts
- changing canonical semantics

Those changes can happen later once the public entrypoint and shipping
contract are already stable.
