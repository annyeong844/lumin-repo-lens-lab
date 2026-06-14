# Vitest Public/Workspace Surfaces Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-public-surface.mjs`
> - `tests/test-public-deep-import-risk.mjs`
> - `tests/test-workspace-no-exports.mjs`
> - `tests/test-mdx-consumers.mjs`

---

## Purpose

This review decides whether the public/workspace consumer surface suites can
move together as one Lane D Vitest mirror batch. It does not add the Vitest
suites.

The batch is acceptable because all four suites protect the same boundary:
package, workspace, script, HTML, and MDX surfaces must constrain import
absence claims without becoming broader deadness proof.

The future mirror must remain a behavior-preserving runner mirror. It must not
change package public-surface collection, deep-import risk classification,
workspace alias resolution, MDX consumer extraction, deadness ranking, or
action-safety promotion.

## Reviewed Evidence

| Suite                                    | Preserved Node Command                        | Proposed Focused Vitest Command               | Surface Under Review                                  |
| ---------------------------------------- | --------------------------------------------- | --------------------------------------------- | ----------------------------------------------------- |
| `tests/test-public-surface.mjs`          | `node tests/test-public-surface.mjs`          | `npm run test:vitest:public-surface`          | package, script, and HTML entry public surfaces       |
| `tests/test-public-deep-import-risk.mjs` | `node tests/test-public-deep-import-risk.mjs` | `npm run test:vitest:public-deep-import-risk` | public deep-import risk and package files policy      |
| `tests/test-workspace-no-exports.mjs`    | `node tests/test-workspace-no-exports.mjs`    | `npm run test:vitest:workspace-no-exports`    | legacy workspace subpath and output-to-source aliases |
| `tests/test-mdx-consumers.mjs`           | `node tests/test-mdx-consumers.mjs`           | `npm run test:vitest:mdx-consumers`           | MDX import consumers and fenced-code exclusion        |

Current Node evidence checked for this review:

```text
node tests/test-public-surface.mjs          # 26 passed, 0 failed
node tests/test-public-deep-import-risk.mjs # 29 passed, 0 failed
node tests/test-workspace-no-exports.mjs    # 24 passed, 0 failed
node tests/test-mdx-consumers.mjs           # 15 passed, 0 failed
```

Goal lane: Lane D, resolver/surface. This review covers public/workspace
consumer surfaces only. It does not cover unsupported resolver-family
diagnostics, generated blind zones, deadness/ranking, performance/incremental,
or pre-write cue policy suites.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share a package/workspace consumer-surface evidence contract. The mirror
must keep every Node entrypoint runnable and must not turn public surface,
deep-import, workspace, or MDX review evidence into automated `SAFE_FIX` proof.

## Protected Invariants

The future Vitest batch must preserve these public/workspace contracts:

- Package `exports`, `main`, `module`, `types`, and `bin` fields contribute
  package public-surface files with package-relative bare targets handled
  correctly.
- Conditional `exports` entries record the condition path that produced each
  surface candidate.
- Dist output targets prefer authored source files when supported source
  mappings exist, while keeping the original package target in evidence.
- Wildcard `exports` entries expand only matching public surface files and keep
  wildcard evidence.
- Package script entrypoints for supported tools remain visible, including
  explicit and dynamic Rollup input cases.
- HTML module script entrypoints are collected with HTML-file evidence, while
  non-module script tags are ignored.
- Private packages and packages without names do not create public deep-import
  contracts.
- Publishable packages without `exports` fail closed unless package `files`
  policy or npm always-included files prove the target is excluded.
- Root-only `exports`, null export leaves, explicit file exports, wildcard
  exports, array fallbacks, and package `files` entries keep their current
  risk/reason detail.
- Unsupported package `files` entries, drive-letter paths, backslashes, and
  parent traversal fail closed instead of silently clearing risk.
- Workspace packages without `exports` register precise legacy subpath aliases
  and do not let workspace imports leak to external.
- Legacy workspace aliases remain additive: imported exports become live while
  genuinely unconsumed sibling exports remain dead candidates.
- Explicit `exports` restrictions still win; legacy fallback must not reopen a
  package surface that explicit `exports` deliberately closes.
- Dist and declarationDir output targets map back to package-root or source
  files only under supported output-to-source layouts.
- Missing generated typings entries stay unresolved rather than fake-resolved.
- MDX import parsing records named, default, and namespace consumers, including
  default-plus-namespace forms.
- MDX fenced-code imports do not contribute fan-in evidence.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- A script or HTML string mention that is not a supported entrypoint must not
  become public surface evidence.
- A package `files` entry with unsupported or unsafe path shape must not clear
  public deep-import risk.
- `main`, default `index.js`, `bin`, `directories.bin`, and README variants
  remain npm always-included risks even when `files` excludes them.
- Workspace deep subpath imports must not be classified external simply because
  the workspace package omits `exports`.
- Workspace alias fallback must not blanket-mark every package export as live.
- Dotted extensionless stems such as `location.input` must still resolve to
  authored source when supported.
- Types-only source entries must resolve both type and value imports when the
  fixture proves that package shape.
- Missing generated declaration output must remain unresolved, not guessed.
- MDX imports inside fenced examples must not protect otherwise unused exports.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- Temporary package fixtures may be shared for setup, file writes, JSON reads,
  command execution, and cleanup only.
- Shared helpers must not decide package public-surface meaning, deep-import
  risk, workspace alias eligibility, output-to-source mapping, MDX import
  semantics, fan-in evidence, deadness ranking, or action-safety promotion.
- The mirror must not absorb unsupported-family resolver suites, generated
  blind-zone suites, deadness/ranking/action-safety suites,
  performance/incremental suites, pre-write cue policy suites, or full
  audit-repo orchestration suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/public-surface.test.mjs`,
2. `tests/public-deep-import-risk.test.mjs`,
3. `tests/workspace-no-exports.test.mjs`,
4. `tests/mdx-consumers.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving the four suites from `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups represented
as named Vitest `it(...)` blocks. It may share setup-only temporary repo and
artifact-read helpers inside test files, but no shared helper should decide
package public-surface rules, public deep-import policy, workspace fallback
resolution, MDX import semantics, fan-in meaning, deadness ranking, or
action-safety meaning.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest`, doc-script checks, and formatting checks
so the reviewed runner discovery boundary and wiki references stay current.
