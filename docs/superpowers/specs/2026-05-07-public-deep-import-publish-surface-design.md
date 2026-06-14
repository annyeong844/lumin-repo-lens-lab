# Public Deep-Import Publish Surface Design

> **Status:** design draft, approved for implementation planning after review  
> **Last updated:** 2026-05-07  
> **Owner:** maintainer-facing accuracy work for `lumin-repo-lens-lab`

## 1. Problem

PCEF correctly treats public deep-import risk as a contract blocker. If a
publishable package allows consumers to deep-import arbitrary files, demoting an
export can break users even when the repository graph has no internal
consumers.

The current rule is intentionally conservative:

```text
package has name
package is not private
package has no exports map
=> every file in that package has public deep-import risk
```

That avoids false positives, but it is too coarse for packages that use
`package.json#files` to publish only a narrow artifact surface. In those
packages, source files outside the published `files` allowlist are not public
deep-import targets through the npm package tarball, even if `exports` is
absent.

The goal is not to make public contract proof optimistic. The goal is to use
explicit package metadata to distinguish:

```text
exports absent and publish surface unknown
vs.
exports absent but package files excludes this source file
```

Unknown remains blocking. Explicit exclusion can unblock.

## 2. Goals

1. Preserve the current false-positive posture for unknown publish surfaces.
2. Use `package.json#files` as positive evidence that a file is outside the
   published package surface.
3. Keep `exports` map behavior authoritative when an `exports` field exists.
4. Avoid repository-specific rules, package allowlists, or references to a
   single stress-test checkout.
5. Emit structured detail so reviewers can see why public deep-import risk was
   blocked or cleared.
6. Keep this slice small enough to implement without building a full npm pack
   simulator.

## 3. Non-Goals

- Do not implement full `npm pack --dry-run` semantics in this slice.
- Do not parse `.npmignore`, `.gitignore`, npm default include/exclude rules, or
  registry metadata.
- Do not run package manager commands.
- Do not infer broad publish surfaces from `module`, `types`, or `browser`
  alone when `exports` is absent.
- Do not weaken public deep-import risk when publish surface evidence is
  unknown or ambiguous.
- Do not add package-name-specific exceptions.

## 4. Existing Contract To Preserve

The following behavior stays unchanged:

| Package state | Risk | Reason |
|---|---:|---|
| no nearest `package.json` | false | `package-json-absent` |
| `private: true` | false | `private-package` |
| missing package `name` | false | `package-name-absent` |
| `exports` map explicitly exposes file | true | `explicitly-exposed-file` |
| `exports` wildcard exposes file | true | `wildcard-exposes-file` |
| `exports` map exists and does not expose file | false | `exports-map-does-not-expose-file` |

When `exports` exists, `exports` remains the primary public contract surface.
The `files` allowlist is only used for the no-`exports` case in this design.

## 5. New Contract

### 5.1 No `exports`, No `files`

If a publishable package has no `exports` field and no `files` allowlist,
public deep-import risk remains blocking.

```json
{
  "risk": true,
  "reason": "exports-absent-publish-surface-unknown",
  "packageName": "example-package",
  "publishSurfaceSource": "implicit-npm-surface"
}
```

This replaces the current broad reason
`exports-absent-publishable-package` for this case.

### 5.2 No `exports`, File Is Always Included By npm

If a publishable package has no `exports` field and the candidate file is an
npm always-included package file, public deep-import risk remains blocking even
when `files` does not include it.

The supported always-included rules for this slice are:

- `package.json`
- README variants at the package root
- LICENSE / LICENCE variants at the package root
- the file in the `main` field
- the file or files in the `bin` field
- files under `directories.bin` when it is a string path

If `main` is absent, npm treats `index.js` at the package root as the default
main entry. This slice should treat root `index.js` as always included for the
no-`exports` risk check.

```json
{
  "risk": true,
  "reason": "exports-absent-file-published-always-included",
  "packageName": "example-package",
  "publishSurfaceSource": "npm-always-included",
  "matchedAlwaysIncludedRule": "main",
  "matchedPackageJsonField": "main"
}
```

This check runs before `files-excludes-file`. A `files` allowlist cannot clear
public deep-import risk for a candidate that npm includes regardless of the
allowlist.

### 5.3 No `exports`, `files` Includes The File

If a publishable package has no `exports` field and its `files` allowlist
includes the candidate file, public deep-import risk remains blocking.

```json
{
  "risk": true,
  "reason": "exports-absent-file-published",
  "packageName": "example-package",
  "publishSurfaceSource": "package-json-files",
  "matchedFilesEntry": "src"
}
```

This is still conservative. If the file can be included in the published
tarball and there is no `exports` map closing deep imports, the package may
support public deep imports to that file.

### 5.4 No `exports`, `files` Excludes The File

If a publishable package has no `exports` field, but its `files` allowlist does
not include the candidate file, public deep-import risk is cleared for that
file.

```json
{
  "risk": false,
  "reason": "files-excludes-file",
  "packageName": "example-package",
  "publishSurfaceSource": "package-json-files"
}
```

This does not prove the export is dead. It only clears the public deep-import
contract blocker. `SAFE_FIX` still requires clean deadness, clean contract proof,
and a complete safe action proof.

`files-excludes-file` only clears risk for the candidate repository path being
evaluated. It does not prove that compiled or generated artifacts corresponding
to the same symbol are absent from the published package. Source-to-output
symbol mapping is outside this slice.

## 6. `files` Matching Scope

This slice supports a deliberately small subset of npm `files` matching. The
matching is evidence for public-surface narrowing, not an exact packlist.

Supported entry forms:

| Entry form | Meaning |
|---|---|
| `"dist"` | includes the exact path `dist` and files under `dist/` |
| `"dist/"` | includes the exact path `dist` and files under `dist/` |
| `"src/index.ts"` | includes that exact normalized relative path |
| `"./src/index.ts"` | same as `src/index.ts` |
| `"src/*.ts"` | simple `*` wildcard within a normalized path pattern |
| `"src/**/*.ts"` | recursive `**` wildcard within a normalized path pattern |

Normalization requirements:

- Use package-root-relative paths with `/` separators.
- Strip leading `./` from `files` entries.
- Treat empty string entries, non-string entries, absolute paths, drive-letter
  paths, backslash-separated entries, and parent traversal as unsupported for
  this slice.
- Treat an empty `files: []` array as a supported empty allowlist, subject to
  npm always-included package files.
- Do not treat a malformed `files` entry as proof that a file is excluded.

Unsupported or ambiguous `files` data must fail closed:

```json
{
  "risk": true,
  "reason": "exports-absent-files-unsupported",
  "publishSurfaceSource": "package-json-files"
}
```

If any unsupported `files` entry exists and no supported entry proves inclusion,
the package must not use supported non-matches as exclusion proof. It should
return `exports-absent-files-unsupported`.

No negation semantics are introduced in this slice. If a future implementation
supports `.npmignore`, npm defaults, or package-manager packlists, it should use
a separate policy version and tests.

Ignoring nested `.npmignore` may over-report risk for files matched by `files`
entries, because nested ignore rules can remove files that `files` would
otherwise include. This slice must not use ignored `.npmignore` data to clear
risk.

## 7. Ranking Contract

Ranking should continue to consume a boolean `publicDeepImportRisk` plus
structured detail.

```js
if (contract.publicDeepImportRisk) {
  return REVIEW_FIX;
}
```

This design only changes how that boolean is computed for the no-`exports`
case. It does not alter PCEF ranking semantics.

Important invariants:

- `files-excludes-file` can clear only the public deep-import blocker.
- `exports-absent-publish-surface-unknown`,
  `exports-absent-file-published-always-included`,
  `exports-absent-file-published`, and
  `exports-absent-files-unsupported` block `SAFE_FIX`.
- Positive reachability or call-graph evidence must not override a blocking
  public deep-import risk detail.
- A package-level unknown must not become a repo-global blocker outside the
  candidate's nearest package scope.

## 8. Artifact Detail

The detail object should stay compatible with current consumers and add
publish-surface fields only when relevant:

```json
{
  "risk": false,
  "reason": "files-excludes-file",
  "packageName": "example-package",
  "packageRoot": "packages/example",
  "relFileFromPkgRoot": "src/internal.ts",
  "publishSurfaceSource": "package-json-files",
  "filesEntriesChecked": ["dist", "README.md"]
}
```

For included files:

```json
{
  "risk": true,
  "reason": "exports-absent-file-published",
  "packageName": "example-package",
  "packageRoot": "packages/example",
  "relFileFromPkgRoot": "src/public.ts",
  "publishSurfaceSource": "package-json-files",
  "matchedFilesEntry": "src"
}
```

For npm always-included files:

```json
{
  "risk": true,
  "reason": "exports-absent-file-published-always-included",
  "packageName": "example-package",
  "packageRoot": "packages/example",
  "relFileFromPkgRoot": "src/index.js",
  "publishSurfaceSource": "npm-always-included",
  "matchedAlwaysIncludedRule": "main",
  "matchedPackageJsonField": "main"
}
```

`filesEntriesChecked` should be capped if needed, but the cap must be
deterministic. The cap is display/debug detail and must not affect the risk
decision.

## 9. Design Options Considered

### Option A: Keep Current Blanket Risk

This is safest but too conservative. It keeps every publishable package without
`exports` in `REVIEW_FIX`, even when `files` explicitly publishes only build
artifacts.

### Option B: Use `files` As A Narrowing Signal

This is the recommended slice. It is conservative because unknown or unsupported
metadata still blocks, while explicit exclusion can clear only the public
deep-import blocker.

### Option C: Implement Full Packlist Resolution

This may be useful later, but it is unnecessary for the next slice. Full
packlist resolution involves npm defaults, `.npmignore`, `.gitignore`,
package-manager behavior, and command execution or extra dependency decisions.
It should not be introduced until the simpler `files` evidence path is
validated.

## 10. Test Plan

Focused unit tests for `_lib/package-exports.mjs`:

- `private: true` remains no risk.
- package without `name` remains no public deep-import contract.
- `exports` map explicit and wildcard behavior remains unchanged.
- no `exports`, no `files` returns risk with
  `exports-absent-publish-surface-unknown`.
- no `exports`, `files: ["dist"]`, `main: "src/index.js"`, candidate
  `src/index.js` returns risk with
  `exports-absent-file-published-always-included` and
  `matchedAlwaysIncludedRule: "main"`.
- no `exports`, `files: ["dist"]`, no `main`, candidate `index.js` returns risk
  with `matchedAlwaysIncludedRule: "default-main"`.
- no `exports`, `files: ["dist"]`, `bin: {"cli": "src/cli.js"}`, candidate
  `src/cli.js` returns risk with
  `exports-absent-file-published-always-included` and
  `matchedAlwaysIncludedRule: "bin"`.
- no `exports`, `files: ["dist"]`, `directories.bin: "bin"`, candidate
  `bin/tool.js` returns risk with
  `matchedAlwaysIncludedRule: "directories.bin"`.
- no `exports`, `files: []`, candidate `src/internal.js` returns no risk unless
  matched by an npm always-included rule.
- no `exports`, `files: ["dist"]`, candidate `src/internal.ts` returns no risk
  with `files-excludes-file`.
- no `exports`, `files: ["src"]`, candidate `src/internal.ts` returns risk with
  `exports-absent-file-published`.
- no `exports`, `files: ["dist"]`, candidate `dist` has deterministic exact
  path behavior.
- exact file entry matches only that file.
- no `exports`, `files: ["src/*"]`, candidate `src/a.ts` matches and candidate
  `src/nested/a.ts` does not.
- no `exports`, `files: ["src/**/*.ts"]`, direct-child behavior is explicitly
  pinned by the test.
- malformed `files` entries fail closed with
  `exports-absent-files-unsupported`.
- mixed supported and unsupported entries fail closed when no supported entry
  proves inclusion.
- absolute paths, drive-letter paths, backslash paths, and parent traversal
  entries fail closed when no supported entry proves inclusion.

Ranking tests:

- `files-excludes-file` allows an otherwise clean safe action to remain
  `SAFE_FIX`.
- `exports-absent-publish-surface-unknown` remains `REVIEW_FIX`.
- `exports-absent-file-published-always-included` remains `REVIEW_FIX`.
- `exports-absent-file-published` remains `REVIEW_FIX`.
- review reasons summarize the new reason codes.

Calibration:

- Run focused quick audits on one package with explicit `files`, one package
  without `exports` or `files`, and one package with an `exports` map.
- The expected result is fewer blanket public-deep-import reviews only where
  package metadata explicitly excludes the source file from the publish
  surface.

## 11. Implementation Slice

1. Add a helper for npm always-included package file checks inside
   `_lib/package-exports.mjs`.
2. Add a helper for `package.json#files` inclusion checks inside
   `_lib/package-exports.mjs`.
3. Update `getPublicDeepImportRisk()` only in the no-`exports` branch, checking
   npm always-included files before applying `files-excludes-file`.
4. Preserve existing detail object fields and add publish-surface detail fields.
5. Update focused public deep-import tests.
6. Update ranking integration tests for the new reason codes.
7. Run focused tests before any broad CI:

```bash
node tests/test-public-deep-import-risk.mjs
node tests/test-rank-fixes.mjs
npm run lint
```

Full local CI is useful if the implementation changes shared artifact
serialization or ranking summaries beyond these paths.

## 12. Open Future Work

- Full npm packlist support.
- `.npmignore` and `.gitignore` interaction.
- Optional dependency on a mature packlist library, if needed.
- Recording publish-surface policy version in artifact metadata.
- Package-manager-specific behavior for npm, pnpm, Yarn, and Bun.

These are intentionally outside this slice. The next implementation should
first prove that explicit `files` metadata solves the over-conservative case
without weakening unknown public-contract risk.

## 13. References

- npm package.json docs:
  <https://docs.npmjs.com/cli/v11/configuring-npm/package-json/>
