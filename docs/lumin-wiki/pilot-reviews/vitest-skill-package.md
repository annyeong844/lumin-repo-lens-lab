# Vitest Skill Package Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-skill-package.mjs`.

---

## Purpose

This review decides whether `tests/test-skill-package.mjs` can move as a narrow
Lane G Vitest mirror. It does not add the Vitest suite.

The suite protects the deployable skill package produced by
`scripts/build-skill.mjs`: public wrapper scripts, shared engine relocation,
packaged skill surfaces, packaged references/templates/canonical spine, Codex
metadata, generated package metadata, smoke test, and dependency setup behavior.

This suite is acceptable as a single-suite mirror because it builds into a
temporary output directory, checks generated package contents, and does not
classify repository code or rank analyzer findings. It should stay separate
from `test-skill-surface.mjs`, which checks maintainer-checkout text contracts,
and from `test-plugin-package.mjs`, which checks the outer Claude Code plugin
root.

## Reviewed Evidence

| Suite                          | Preserved Node Command              | Proposed Focused Vitest Command     | Surface Under Review                           |
| ------------------------------ | ----------------------------------- | ----------------------------------- | ---------------------------------------------- |
| `tests/test-skill-package.mjs` | `node tests/test-skill-package.mjs` | `npm run test:vitest:skill-package` | `scripts/build-skill.mjs` generated skill tree |

Current Node evidence checked for this review:

```text
node tests/test-skill-package.mjs # 38 passed, 0 failed
```

Goal lane: Lane G, public package/plugin/hooks. This review covers only the
generated skill-package output subset of that lane.

## Result

This suite is acceptable as one narrow Vitest mirror.

The future implementation PR should preserve the same generated package
contracts without changing plugin packaging, public publish workflows, hook
runtime, resolver behavior, deadness/ranking behavior, or performance
measurement.

## Protected Invariants

The future Vitest mirror must preserve these generated skill-package contracts:

- `scripts/build-skill.mjs` exits successfully when writing to a temporary
  `--out` directory;
- the generated skill exposes only the public wrapper scripts plus smoke test;
- generated skill package includes the shared audit, Codex, write-gate, and
  canon skill surfaces;
- generated package includes public README, canonical spine, templates, and
  references needed by the public skill contract;
- maintainer-only templates, self-audit notes, lab docs, and history/spec paths
  stay out of the generated package;
- generated refactor-plan policy and template keep behavior guidance separate
  from output shape;
- generated short and long review checklists preserve chat-facing and formal
  report boundaries;
- generated `SKILL.md` stays slim enough for progressive disclosure and keeps
  workflow detail in references/templates;
- generated command-routing preserves full-baseline then quick-incremental
  audit cadence and feature-discovery tail wording;
- generated package keeps the historical false-positive ledger out of normal
  public context;
- generated canonical spine omits maintainer patch-note metadata while
  retaining runtime canon invariants;
- generated package explains first-run parser dependency setup without making
  users hand-write install steps;
- implementation files move under `_engine`, and generated markdown rewrites
  maintainer `_lib/` references to `_engine/lib/`;
- generated `package.json` and `package-lock.json` expose only the public
  `lumin-repo-lens-lab` bin, parser-supported Node engines, and shipping
  dependencies;
- generated producer imports are rewritten from `./_lib` to `../lib`;
- generated public wrapper and rewritten producers pass `node --check`;
- generated audit wrapper reaches the engine help text;
- generated package metadata drives skill-safe dependency setup;
- generated check-canon skips absent self-audit fact canon instead of shipping
  it;
- generated smoke test is runnable;
- generated package can resolve tree-sitter WASM dependencies from package
  root and extract Go symbols;
- generated engine comments do not point at maintainer-only history/spec docs;
- plugin wrapper uses default skill/command discovery and marketplace source;
- generated command files delegate to the expected packaged skill surfaces;
- Codex wrapper stays thin and does not duplicate the engine;
- generated public English docs use the English `unknown` evidence label;
- generated README keeps marketplace install instructions before Codex link
  install and warns that `.audit/` artifacts may be commit-sensitive.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- missing public wrapper scripts or smoke test must fail;
- shipping maintainer-only self-audit or lab/history/spec material must fail;
- generated SKILL surfaces growing too large or embedding hidden workflow
  detail must fail;
- stale `_lib/` paths in generated package markdown or producer imports must
  fail;
- package metadata exposing legacy `lumin-audit` or `grounded-audit` bins must
  fail;
- package lock drift that pulls in maintainer-only dev dependencies must fail;
- rewritten producer syntax errors must fail;
- public wrapper failing to reach engine help must fail;
- dependency setup that ignores `luminRepoLens` package metadata must fail;
- missing smoke-test output must fail;
- tree-sitter WASM dependency resolution or Go symbol extraction from the
  generated package must fail;
- generated comments pointing at maintainer-only history/spec paths must fail;
- command routing that delegates to the wrong packaged skill surface must fail;
- Codex wrapper duplication of the engine must fail;
- generated English public docs using Korean uncertainty labels must fail;
- README install order or `.audit/` privacy warning regressions must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is the temporary generated skill package output and any
  setup-only dependency copies needed by the existing suite.
- The mirror may run `scripts/build-skill.mjs` and generated wrapper/smoke
  scripts inside the temporary output tree.
- The mirror must not mutate the maintainer checkout or generated
  `skills/lumin-repo-lens-lab` tree.
- The mirror must not absorb `test-skill-surface.mjs`,
  `test-plugin-package.mjs`, `test-publish-public-plugin.mjs`,
  `test-github-actions-ci-policy.mjs`, hook runtime suites, analyzer behavior,
  resolver behavior, generated/framework surfaces, deadness/ranking, or
  performance/incremental cache behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/skill-package.test.mjs`,
2. `npm run test:vitest:skill-package`,
3. candidate-board updates moving `tests/test-skill-package.mjs` from
   `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups represented
as named Vitest `it(...)` blocks. It may share local setup helpers inside the
test file, but no shared helper should decide generated package policy,
shipping surface boundaries, dependency metadata, or command-routing semantics.

Run the preserved Node command, the focused Vitest command, `npm run
test:vitest`, and the doc-script checks when changing this batch.
