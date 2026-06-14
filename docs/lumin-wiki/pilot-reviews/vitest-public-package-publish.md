# Vitest Public Package Publish Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-plugin-package.mjs`
> - `tests/test-publish-public-plugin.mjs`
> - `tests/test-github-actions-ci-policy.mjs`

---

## Purpose

This review decides whether three public package publishing and CI policy
suites can move together as one Lane G Vitest mirror batch. It does not add the
Vitest suites.

The batch is acceptable because every candidate protects public package
delivery mechanics rather than analyzer meaning:

- `test-plugin-package` builds the Claude Code plugin root and checks packaged
  command, hook, metadata, and skill-surface wiring;
- `test-publish-public-plugin` exercises the local public-package publish
  workflow against temporary git repositories;
- `test-github-actions-ci-policy` checks the public CI routing policy that
  avoids draft-PR runner spend while preserving ready/manual/push validation.

This batch must stay separate from the larger public skill text suites
(`test-skill-package.mjs` and `test-skill-surface.mjs`) and from hook runtime
event-store suites. Those have different fixture sizes and should receive their
own review pages.

## Reviewed Evidence

| Suite                                     | Preserved Node Command                         | Proposed Focused Vitest Command                | Surface Under Review                         |
| ----------------------------------------- | ---------------------------------------------- | ---------------------------------------------- | -------------------------------------------- |
| `tests/test-plugin-package.mjs`           | `node tests/test-plugin-package.mjs`           | `npm run test:vitest:plugin-package`           | `scripts/build-plugin-package.mjs` output    |
| `tests/test-publish-public-plugin.mjs`    | `node tests/test-publish-public-plugin.mjs`    | `npm run test:vitest:publish-public-plugin`    | `scripts/publish-public-plugin.mjs` workflow |
| `tests/test-github-actions-ci-policy.mjs` | `node tests/test-github-actions-ci-policy.mjs` | `npm run test:vitest:github-actions-ci-policy` | `.github/workflows/ci.yml` policy            |

Current Node evidence checked for this review:

```text
node tests/test-plugin-package.mjs          # 14 passed, 0 failed
node tests/test-publish-public-plugin.mjs   # 13 passed, 0 failed
node tests/test-github-actions-ci-policy.mjs # 5 passed, 0 failed
```

Goal lane: Lane G, public package/plugin/hooks. This review covers only the
public package build, publish, and CI policy subset of that lane.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all three mirrors together because they
share a public package delivery boundary. The mirror must keep every Node
entrypoint runnable and must not replace local git/package fixtures with mocks
that would hide packaging drift.

## Protected Invariants

The future Vitest batch must preserve these public package contracts:

- `scripts/build-plugin-package.mjs` exits successfully when building the
  plugin root;
- the plugin root includes Claude Code plugin metadata, marketplace metadata,
  command files, hook manifest, hook runner scripts, and the three packaged
  skill surfaces;
- default plugin packaging excludes the Codex wrapper to avoid Claude Code
  surface collision, while `--include-codex` includes it explicitly;
- command files resolve `${CLAUDE_PLUGIN_ROOT}` references inside the packaged
  root and delegate to the expected packaged skill surface;
- plugin metadata version matches the packaged skill version and distribution
  marker;
- the package README names the plugin install root and warns against installing
  `skills/` alone;
- the plugin package smoke check runs after staging and stale legacy output is
  removed beside the current package;
- public publish dry-run stages current plugin and skill metadata without
  committing;
- public publish dry-run does not leak maintainer-only root directories;
- changelog sync preserves historical public entries while prepending current
  release notes;
- public package CI workflow and auto-hook files are synced into the generated
  public repository;
- `--push` commits and pushes to the public repository with the configured
  publish author;
- the public package exposes current plugin metadata, skill metadata, CI
  workflow, auto-hook manifest, and npm publish helper scripts;
- GitHub Actions can still be started manually;
- pull requests run CI for `opened`, `synchronize`, `reopened`, and
  `ready_for_review`;
- draft pull requests skip the test job before runner work;
- pushes to `main` or `master` still run CI.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a packaged command referencing a missing `${CLAUDE_PLUGIN_ROOT}` target must
  fail;
- a command delegating to the wrong skill surface must fail;
- a default build that accidentally ships the Codex wrapper must fail;
- an opt-in Codex build that omits the wrapper must fail;
- stale legacy plugin output left beside the current package must fail;
- dry-run publishing that mutates git history must fail;
- maintainer-only files such as `docs`, `tests`, `_lib`, or the Codex-only
  wrapper leaking into the public checkout must fail;
- stale public CI workflow content or maintainer-only CI paths must fail;
- pushed public repository metadata drifting from the current package version
  must fail;
- draft PRs allocating runner work must fail;
- manual, ready-for-review, and push-triggered CI paths disappearing must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The public publish fixture must keep using local git repositories, not GitHub
  network calls.
- The plugin package fixture may build into temporary directories, but must not
  mutate the maintainer checkout.
- The GitHub Actions policy mirror may remain source-text based because the
  current suite protects routing policy, not runner behavior.
- The mirror must not absorb `test-skill-package.mjs`,
  `test-skill-surface.mjs`, hook runtime/event-store suites, analyzer
  behavior, resolver behavior, generated/framework surfaces,
  deadness/ranking, or performance/incremental cache behavior.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/plugin-package.test.mjs`,
2. `tests/publish-public-plugin.test.mjs`,
3. `tests/github-actions-ci-policy.test.mjs`,
4. focused `npm run test:vitest:*` commands for each suite,
5. candidate-board updates moving the three suites from `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups represented
as named Vitest `it(...)` blocks. It may share local setup helpers inside a
test file, but no shared helper should decide which package files are public or
which CI events are allowed.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest` and the doc-script checks so the reviewed
runner discovery boundary and wiki references stay current.
