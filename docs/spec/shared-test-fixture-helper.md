# Shared Test Fixture Helper

> **Role:** maintainer-facing implementation spec for the first narrow shared
> test fixture helper.
> **Status:** SPEC.
> **Last updated:** 2026-05-12

---

## 1. Problem

The Lumin test suite repeatedly builds small repositories under the OS temp
directory, writes `package.json`, writes source files, runs one producer or
audit command, reads JSON artifacts, and removes the temp tree. The repeated
setup is noisy enough to make tests harder to reform, but merging fixtures too
early can hide the failure mode each suite protects.

The next test-reform step is therefore not broad test movement. It is a narrow
setup helper that removes mechanical temp-repo boilerplate while keeping
resolver, deadness, pre-write, performance, and public-package assertions
suite-local.

## 2. Goals

- Provide one reusable helper for isolated temporary repo setup.
- Keep file writing, JSON writing, JSON reading, path resolution, and cleanup
  safe on Windows and POSIX.
- Make path traversal, absolute paths, drive-letter paths, and NUL paths fail
  loudly.
- Keep all analyzer interpretation outside the helper.
- Let suites keep their existing commands, assertions, fixture names, and
  failure modes.

## 3. Non-Goals

- Do not move test files in the helper PR.
- Do not add resolver, deadness, pre-write, package, or performance semantics.
- Do not add a command runner in the first helper. Suites should continue to
  invoke producers explicitly so the tested entry point stays visible.
- Do not auto-generate framework, generated-artifact, or workspace layouts.
- Do not change generated `tests/README.md` descriptions by hand.

## 4. Proposed File

Implementation adds:

```text
tests/_helpers/temp-repo-fixture.mjs
```

This file is test-only. It must not be imported by engine code, skill package
code, generated package code, or public command wrappers.

## 5. Proposed API

```js
const fx = createTempRepoFixture({
  prefix: 'lrl-resolver-output-layout-',
  packageJson: { name: 'fixture', private: true, type: 'module' },
  outputDirName: '.audit',
});

try {
  fx.write('src/index.ts', 'export const value = 1;\n');
  fx.writeJson('tsconfig.json', { compilerOptions: { baseUrl: '.' } });

  // The suite owns command execution.
  execFileSync(process.execPath, [producer, '--root', fx.root, '--output', fx.output]);

  const symbols = fx.readJson('symbols.json', { from: 'output' });
} finally {
  fx.cleanup();
}
```

Required exports:

```js
export function createTempRepoFixture(options = {}) { ... }
```

Returned object:

```js
{
  root,
  output,
  path(relPath),
  outputPath(relPath),
  mkdir(relPath),
  write(relPath, text),
  writeJson(relPath, value),
  read(relPath, options),
  readJson(relPath, options),
  cleanup()
}
```

`options`:

```js
{
  prefix,
  packageJson,
  outputDirName
}
```

Defaults:

- `prefix`: `lrl-fixture-`
- `packageJson`: `{ "name": "fixture", "private": true, "type": "module" }`
- `outputDirName`: `.audit`

## 6. Safety Contract

All relative paths accepted by the helper must be repo-relative, forward-slash
friendly paths. The helper may normalize separators internally, but it must
reject:

- empty paths,
- absolute paths,
- Windows drive-letter paths,
- parent traversal such as `../x` or `a/../../x`,
- NUL bytes,
- paths that resolve outside the fixture root or output root.

Cleanup must only remove the helper-created root directory. It must verify that
the resolved cleanup target is inside the OS temp directory and matches the
helper-created prefix before recursive removal.

## 7. Interpretation Boundary

The helper may:

- create directories,
- write text files,
- write JSON files with trailing newline,
- read text files,
- read JSON files,
- return root/output paths,
- remove the temp tree.

The helper must not:

- decide whether an import is internal or external,
- create resolver diagnostics,
- classify dead exports,
- create pre-write intents,
- build package allowlists,
- run `audit-repo.mjs`,
- assert `SAFE_FIX`, `AGENT_REVIEW_CUE`, or public install status.

Those meanings remain owned by the suites listed in
[`docs/lumin-wiki/concepts/fixture-shapes.md`](../lumin-wiki/concepts/fixture-shapes.md).

## 8. First Implementation Tests

The helper implementation PR adds one focused test file:

```text
tests/test-temp-repo-fixture-helper.mjs
```

Minimum cases:

| Case | Expected behavior |
|---|---|
| creates root, output, and default `package.json` | paths exist and package defaults are written |
| writes nested source file | parent directories are created and bytes round-trip |
| writes and reads JSON from root | parsed value matches input and file ends with newline |
| writes and reads JSON from output | parsed output artifact matches input |
| rejects absolute and drive-letter paths | throws before writing |
| rejects parent traversal | throws before writing |
| rejects NUL paths | throws before writing |
| cleanup removes only fixture root | root disappears and output disappears with it |

The helper test should fail on concrete safety behavior, not merely on the
absence of the helper file.

## 9. First Migration Candidate

After the helper itself is tested, migrate at most one low-risk suite in the
same PR or a follow-up PR. The first migration uses the saved-answer behavior
corpus verifier because its temp directory setup is mechanical and does not
encode analyzer-family semantics in its fixture builder.

Avoid first migrations in:

- resolver unsupported-family fixtures,
- SAFE_FIX calibration fixtures,
- public install verification notes,
- scanner equivalence fixtures,
- pre-write lifecycle baseline tests.

## 10. Acceptance Criteria

- The helper has safety tests for path containment and cleanup.
- Migrated suites still contain their original negative assertions.
- No analyzer behavior changes.
- No generated package surface changes.
- No broad test movement.
- `tests/README.md` remains generated by `npm run check:test-doc`.
- Wiki/tracker docs continue to identify the helper as setup-only.
