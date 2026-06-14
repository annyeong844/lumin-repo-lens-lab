# Vitest Symlink Aliasing Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-symlink-aliasing.mjs`

---

## Purpose

This review decides whether `tests/test-symlink-aliasing.mjs` may move to a
focused Lane D Vitest mirror. It does not add the Vitest suite.

The suite protects resolver canonicalization for source files reached through
file and directory symlinks. It is acceptable as a single-suite batch because
the fixture boundary is narrow: create a temporary package, create symlinks
when the OS permits them, build the alias map, call `makeResolver(...)`, and
assert that graph-facing paths use the canonical realpath.

This review must stay separate from broader resolver expansion,
unsupported-family diagnostics, deadness/ranking, generated surfaces, public
package policy, and performance/incremental cache behavior.

## Reviewed Evidence

| Suite                             | Preserved Node Command                 | Proposed Focused Vitest Command        | Surface Under Review       |
| --------------------------------- | -------------------------------------- | -------------------------------------- | -------------------------- |
| `tests/test-symlink-aliasing.mjs` | `node tests/test-symlink-aliasing.mjs` | `npm run test:vitest:symlink-aliasing` | resolver realpath aliasing |

Current Node evidence checked for this review on the local Windows shell:

```text
node tests/test-symlink-aliasing.mjs # 0 passed, 0 failed, 6 skipped
```

The skip is expected when the shell cannot create file symlinks without
elevated privilege. Linux CI or Windows Developer Mode provides the positive
realpath coverage.

Goal lane: Lane D, resolver/surface. This review covers only symlink
canonicalization in resolver output.

## Result

This suite is acceptable as one focused Vitest mirror.

The future implementation PR may add one mirror file and one focused script,
provided it keeps the Node entrypoint runnable, preserves the clean platform
skip path, and keeps resolver realpath evidence local to this suite.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- on platforms that can create symlinks, a file symlink import resolves to the
  target realpath, not the in-tree symlink path;
- an extensionless symlink spec resolves to the same realpath as the explicit
  file import;
- a directory symlink with `/index.ts` lookup resolves to the target realpath;
- resolver sentinel returns such as `null` and `EXTERNAL` pass through
  unchanged;
- ordinary non-symlink relative imports still resolve normally;
- on platforms that cannot create test symlinks, the suite reports a clean skip
  instead of a failure.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- returning `src/lib.ts` for a symlinked file instead of the vendored realpath
  must fail;
- resolving extensionless symlink imports differently from explicit symlink
  imports must fail;
- resolving a directory symlink to the symlink path rather than the target
  `/index.ts` realpath must fail;
- canonicalizing `null` or `EXTERNAL` sentinels into filesystem paths must fail;
- breaking ordinary relative imports while fixing symlink imports must fail;
- hard-failing on Windows shells without symlink creation privilege must fail.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The Node suite remains runnable and authoritative until a later cleanup spec
  retires it.
- The fixture boundary is temporary package creation, file and directory
  symlink creation, `detectRepoMode(...)`, `buildAliasMap(...)`,
  `makeResolver(...)`, and direct resolver assertions.
- Shared setup may probe symlink support, create temporary files, create
  symlinks when allowed, build resolver inputs, and clean up temporary
  directories.
- Shared helpers must not decide package exports behavior, unsupported-family
  diagnostics, blind-zone scoping, public API policy, deadness/ranking,
  generated-surface behavior, or performance cache identity.
- The mirror must not change resolver behavior, alias-map behavior, symlink
  policy, dead-export classification, or action-safety proof.
- The mirror must not absorb other resolver unsupported-family suites, output
  source layout diagnostics, import-meta-glob diagnostics, deadness/ranking
  suites, generated/resource suites, or performance/incremental suites.

## Implementation Notes

- Prefer one Vitest file: `tests/symlink-aliasing.test.mjs`.
- Add one focused script: `test:vitest:symlink-aliasing`.
- Preserve the platform probe as an explicit skip path with the same reason:
  local symlink fixture creation may require elevated privilege on Windows.
- Keep the six Node assertions as six Vitest cases when symlink creation is
  available.
- Keep resolver sentinels in the same fixture to guard canonicalization from
  touching non-path returns.

## Validation Commands

The implementation PR must run:

```text
node tests/test-symlink-aliasing.mjs
npm run test:vitest:symlink-aliasing
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-symlink-aliasing.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/test-migration-candidate-board.md
git diff --check
```

Before merge, the implementation should also keep the broader runner lane
green:

```text
npm run check
npm run lint
npm run test:vitest
npm test
```

## Non-Goals

- Do not expand resolver support.
- Do not change symlink resolver behavior.
- Do not add unsupported-family diagnostics.
- Do not change public API, generated-surface, or deadness/ranking behavior.
- Do not treat symlink realpath evidence as automatic deletion proof.
