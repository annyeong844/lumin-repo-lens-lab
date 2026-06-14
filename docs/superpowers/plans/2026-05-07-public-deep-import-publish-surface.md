# Public Deep-Import Publish Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refine public deep-import risk so publishable packages without `exports` can clear the blocker only when `package.json#files` explicitly excludes the candidate file and npm always-included entrypoints do not include it.

**Architecture:** Keep all behavior inside `_lib/package-exports.mjs`. Add small pure helpers for npm always-included package files and supported `files` allowlist matching, then update only the no-`exports` branch of `getPublicDeepImportRisk()`. Pin behavior with focused unit tests and rank-fixes integration tests.

**Tech Stack:** Node.js ES modules, existing `node` test scripts, no new runtime dependency.

---

## File Structure

- Modify `_lib/package-exports.mjs`
  - Add strict path normalization for package publish metadata.
  - Add npm always-included file detection.
  - Add supported `package.json#files` allowlist matching.
  - Update the no-`exports` public deep-import risk branch.
- Modify `tests/test-public-deep-import-risk.mjs`
  - Update the old no-`exports` reason expectation.
  - Add unit coverage for `files`, `main`, `bin`, `directories.bin`, empty arrays, unsupported entries, and globs.
- Modify `tests/test-rank-fixes.mjs`
  - Update the old no-`exports` review reason.
  - Add integration coverage showing `files-excludes-file` permits `SAFE_FIX`.
  - Add integration coverage showing npm always-included `main` remains `REVIEW_FIX`.

No docs or skill mirror changes are required for this implementation slice because the spec already exists and this code is inside the source engine. Run `npm run build:skill` only if implementation touches mirrored skill files.

---

### Task 1: Public Deep-Import Unit Tests

**Files:**
- Modify: `tests/test-public-deep-import-risk.mjs`

- [ ] **Step 1: Update the old no-exports reason expectation**

In `tests/test-public-deep-import-risk.mjs`, update assertion `D9` so it expects the new unknown publish-surface reason:

```js
{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    main: './dist/index.js',
  }, 'src/internal.ts');
  assert('D9. deep-import risk detail explains unknown publish surface without exports',
    detail.risk === true &&
      detail.reason === 'exports-absent-publish-surface-unknown' &&
      detail.publishSurfaceSource === 'implicit-npm-surface' &&
      detail.packageName === 'pkg' &&
      detail.relFileFromPkgRoot === 'src/internal.ts',
    JSON.stringify(detail));
}
```

- [ ] **Step 2: Add package files allowlist tests**

Append these assertions before the final summary:

```js
{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['dist'],
  }, 'src/internal.ts');
  assert('D13. package files excluding source clears public deep-import risk',
    detail.risk === false &&
      detail.reason === 'files-excludes-file' &&
      detail.publishSurfaceSource === 'package-json-files',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src'],
  }, 'src/internal.ts');
  assert('D14. package files including source keeps public deep-import risk',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published' &&
      detail.publishSurfaceSource === 'package-json-files' &&
      detail.matchedFilesEntry === 'src',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/index.ts'],
  }, 'src/index.ts');
  assert('D15. exact package files entry includes exact file',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published' &&
      detail.matchedFilesEntry === 'src/index.ts',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/index.ts'],
  }, 'src/other.ts');
  assert('D16. exact package files entry does not include sibling file',
    detail.risk === false &&
      detail.reason === 'files-excludes-file',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: [],
  }, 'src/internal.js');
  assert('D17. empty files array excludes non-entry source file',
    detail.risk === false &&
      detail.reason === 'files-excludes-file',
    JSON.stringify(detail));
}
```

- [ ] **Step 3: Add npm always-included tests**

Append these assertions after the `files` allowlist tests:

```js
{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    main: 'src/index.js',
    files: ['dist'],
  }, 'src/index.js');
  assert('D18. main file remains public risk even when files excludes it',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.publishSurfaceSource === 'npm-always-included' &&
      detail.matchedAlwaysIncludedRule === 'main' &&
      detail.matchedPackageJsonField === 'main',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['dist'],
  }, 'index.js');
  assert('D19. default main index.js remains public risk',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.matchedAlwaysIncludedRule === 'default-main',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    bin: { cli: 'src/cli.js' },
    files: ['dist'],
  }, 'src/cli.js');
  assert('D20. bin file remains public risk even when files excludes it',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.matchedAlwaysIncludedRule === 'bin' &&
      detail.matchedPackageJsonField === 'bin',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    directories: { bin: 'bin' },
    files: ['dist'],
  }, 'bin/tool.js');
  assert('D21. directories.bin remains public risk even when files excludes it',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.matchedAlwaysIncludedRule === 'directories.bin' &&
      detail.matchedPackageJsonField === 'directories.bin',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: [],
  }, 'README.md');
  assert('D22. README variant remains public risk with empty files array',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.matchedAlwaysIncludedRule === 'readme',
    JSON.stringify(detail));
}
```

- [ ] **Step 4: Add glob and unsupported-entry tests**

Append these assertions after the always-included tests:

```js
{
  const direct = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/*'],
  }, 'src/a.ts');
  const nested = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/*'],
  }, 'src/nested/a.ts');
  assert('D23. single star files entry matches direct child only',
    direct.risk === true &&
      direct.reason === 'exports-absent-file-published' &&
      nested.risk === false &&
      nested.reason === 'files-excludes-file',
    JSON.stringify({ direct, nested }));
}

{
  const direct = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/**/*.ts'],
  }, 'src/a.ts');
  const nested = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/**/*.ts'],
  }, 'src/nested/a.ts');
  assert('D24. globstar files entry matches direct and nested children',
    direct.risk === true &&
      nested.risk === true &&
      direct.reason === 'exports-absent-file-published' &&
      nested.reason === 'exports-absent-file-published',
    JSON.stringify({ direct, nested }));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['dist', { bad: true }],
  }, 'src/internal.ts');
  assert('D25. unsupported files entry fails closed when no inclusion is proven',
    detail.risk === true &&
      detail.reason === 'exports-absent-files-unsupported' &&
      detail.publishSurfaceSource === 'package-json-files',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['dist', { bad: true }],
  }, 'dist/index.js');
  assert('D26. supported inclusion wins before unsupported files entry fallback',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published' &&
      detail.matchedFilesEntry === 'dist',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['C:/repo/src/internal.ts'],
  }, 'src/internal.ts');
  assert('D27. drive-letter files entry fails closed',
    detail.risk === true &&
      detail.reason === 'exports-absent-files-unsupported',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['..\\src\\internal.ts'],
  }, 'src/internal.ts');
  assert('D28. backslash and parent traversal files entry fails closed',
    detail.risk === true &&
      detail.reason === 'exports-absent-files-unsupported',
    JSON.stringify(detail));
}
```

- [ ] **Step 5: Run test to verify it fails**

Run:

```bash
node tests/test-public-deep-import-risk.mjs
```

Expected: FAIL. The first failure should mention the old `exports-absent-publishable-package` reason or missing new reason codes.

- [ ] **Step 6: Commit the failing tests**

```bash
git add tests/test-public-deep-import-risk.mjs
git commit -m "test: cover publish-surface deep import risk"
```

---

### Task 2: Package Publish Surface Helpers

**Files:**
- Modify: `_lib/package-exports.mjs`

- [ ] **Step 1: Add strict package metadata path helpers**

Add these helpers after `normalizeRel()`:

```js
function normalizePackageMetadataPath(value) {
  if (typeof value !== 'string') return null;
  const raw = value.trim();
  if (!raw) return null;
  if (raw.includes('\\')) return null;
  if (/^[A-Za-z]:[\\/]/.test(raw)) return null;
  if (raw.startsWith('/')) return null;

  const normalized = normalizeRel(raw);
  if (!normalized || normalized === '.') return null;
  if (normalized.split('/').some((part) => part === '..')) return null;
  return normalized;
}

function isRootReadme(rel) {
  return !rel.includes('/') && /^readme(?:\..+)?$/iu.test(rel);
}

function isRootLicense(rel) {
  return !rel.includes('/') && /^licen[cs]e(?:\..+)?$/iu.test(rel);
}

function pathIsExactOrUnder(base, rel) {
  return rel === base || rel.startsWith(`${base}/`);
}
```

- [ ] **Step 2: Add npm always-included matcher**

Add this helper after the strict path helpers:

```js
function getNpmAlwaysIncludedMatch(pkgJson, relFileFromPkgRoot) {
  const rel = normalizeRel(relFileFromPkgRoot);

  if (rel === 'package.json') {
    return {
      matchedAlwaysIncludedRule: 'package-json',
      matchedPackageJsonField: 'package.json',
    };
  }
  if (isRootReadme(rel)) {
    return {
      matchedAlwaysIncludedRule: 'readme',
      matchedPackageJsonField: 'README',
    };
  }
  if (isRootLicense(rel)) {
    return {
      matchedAlwaysIncludedRule: 'license',
      matchedPackageJsonField: 'LICENSE',
    };
  }

  const main = normalizePackageMetadataPath(pkgJson?.main ?? 'index.js');
  if (main && rel === main) {
    return {
      matchedAlwaysIncludedRule: pkgJson?.main ? 'main' : 'default-main',
      matchedPackageJsonField: pkgJson?.main ? 'main' : 'main-default',
    };
  }

  const bin = pkgJson?.bin;
  const binValues = typeof bin === 'string'
    ? [bin]
    : bin && typeof bin === 'object' && !Array.isArray(bin)
      ? Object.values(bin)
      : [];
  for (const value of binValues) {
    const binPath = normalizePackageMetadataPath(value);
    if (binPath && rel === binPath) {
      return {
        matchedAlwaysIncludedRule: 'bin',
        matchedPackageJsonField: 'bin',
      };
    }
  }

  const directoriesBin = normalizePackageMetadataPath(pkgJson?.directories?.bin);
  if (directoriesBin && pathIsExactOrUnder(directoriesBin, rel)) {
    return {
      matchedAlwaysIncludedRule: 'directories.bin',
      matchedPackageJsonField: 'directories.bin',
    };
  }

  return null;
}
```

- [ ] **Step 3: Add supported `files` matcher**

Add these helpers after `patternMatchesRel()`:

```js
function escapeRegExp(text) {
  return String(text).replace(/[\\^$.*+?()[\]{}|]/g, '\\$&');
}

function filesGlobToRegExp(pattern) {
  let out = '^';
  for (let i = 0; i < pattern.length;) {
    if (pattern.slice(i, i + 3) === '**/') {
      out += '(?:.*/)?';
      i += 3;
    } else if (pattern.slice(i, i + 2) === '**') {
      out += '.*';
      i += 2;
    } else if (pattern[i] === '*') {
      out += '[^/]*';
      i += 1;
    } else {
      out += escapeRegExp(pattern[i]);
      i += 1;
    }
  }
  out += '$';
  return new RegExp(out, 'u');
}

function normalizeFilesEntry(entry) {
  return normalizePackageMetadataPath(entry);
}

function filesEntryMatchesRel(entry, relFileFromPkgRoot) {
  const entryRel = normalizeFilesEntry(entry);
  if (!entryRel) return { supported: false, matched: false };

  const rel = normalizeRel(relFileFromPkgRoot);
  if (entryRel.includes('*')) {
    return {
      supported: true,
      matched: filesGlobToRegExp(entryRel).test(rel),
      normalizedEntry: entryRel,
    };
  }

  const matched = entryRel.includes('.')
    ? rel === entryRel
    : pathIsExactOrUnder(entryRel, rel);
  return { supported: true, matched, normalizedEntry: entryRel };
}

function getFilesAllowlistMatch(filesValue, relFileFromPkgRoot) {
  if (!Array.isArray(filesValue)) {
    return { hasFilesField: true, unsupported: true, matchedEntry: null, checkedEntries: [] };
  }

  let unsupported = false;
  const checkedEntries = [];
  for (const entry of filesValue) {
    const result = filesEntryMatchesRel(entry, relFileFromPkgRoot);
    if (!result.supported) {
      unsupported = true;
      continue;
    }
    checkedEntries.push(result.normalizedEntry);
    if (result.matched) {
      return {
        hasFilesField: true,
        unsupported,
        matchedEntry: result.normalizedEntry,
        checkedEntries,
      };
    }
  }

  return {
    hasFilesField: true,
    unsupported,
    matchedEntry: null,
    checkedEntries,
  };
}
```

- [ ] **Step 4: Update the no-exports branch**

Replace the current no-`exports` branch:

```js
if (!pkgJson.exports) {
  return {
    ...base,
    risk: true,
    reason: 'exports-absent-publishable-package',
    packageName,
  };
}
```

with:

```js
if (!pkgJson.exports) {
  const alwaysIncluded = getNpmAlwaysIncludedMatch(pkgJson, rel);
  if (alwaysIncluded) {
    return {
      ...base,
      risk: true,
      reason: 'exports-absent-file-published-always-included',
      packageName,
      publishSurfaceSource: 'npm-always-included',
      ...alwaysIncluded,
    };
  }

  if (Object.hasOwn(pkgJson, 'files')) {
    const filesMatch = getFilesAllowlistMatch(pkgJson.files, rel);
    if (filesMatch.matchedEntry) {
      return {
        ...base,
        risk: true,
        reason: 'exports-absent-file-published',
        packageName,
        publishSurfaceSource: 'package-json-files',
        matchedFilesEntry: filesMatch.matchedEntry,
        filesEntriesChecked: filesMatch.checkedEntries,
      };
    }
    if (filesMatch.unsupported) {
      return {
        ...base,
        risk: true,
        reason: 'exports-absent-files-unsupported',
        packageName,
        publishSurfaceSource: 'package-json-files',
        filesEntriesChecked: filesMatch.checkedEntries,
      };
    }
    return {
      ...base,
      reason: 'files-excludes-file',
      packageName,
      publishSurfaceSource: 'package-json-files',
      filesEntriesChecked: filesMatch.checkedEntries,
    };
  }

  return {
    ...base,
    risk: true,
    reason: 'exports-absent-publish-surface-unknown',
    packageName,
    publishSurfaceSource: 'implicit-npm-surface',
  };
}
```

- [ ] **Step 5: Run focused unit test**

Run:

```bash
node tests/test-public-deep-import-risk.mjs
```

Expected: PASS with all assertions passing.

- [ ] **Step 6: Commit implementation**

```bash
git add _lib/package-exports.mjs tests/test-public-deep-import-risk.mjs
git commit -m "Refine public deep import publish surface"
```

---

### Task 3: Rank-Fixes Integration

**Files:**
- Modify: `tests/test-rank-fixes.mjs`

- [ ] **Step 1: Update existing public no-exports review expectation**

In the current `I7b` assertion, replace the old reason expectations:

```js
planD.summary.reviewReasons?.publicDeepImportRisk?.['exports-absent-publishable-package'] === 1 &&
planD.reviewFixes?.[0]?.reason.includes('public-deep-import-risk: exports-absent-publishable-package') &&
planD.reviewFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail?.reason === 'exports-absent-publishable-package' &&
```

with:

```js
planD.summary.reviewReasons?.publicDeepImportRisk?.['exports-absent-publish-surface-unknown'] === 1 &&
planD.reviewFixes?.[0]?.reason.includes('public-deep-import-risk: exports-absent-publish-surface-unknown') &&
planD.reviewFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail?.reason === 'exports-absent-publish-surface-unknown' &&
planD.reviewFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail?.publishSurfaceSource === 'implicit-npm-surface' &&
```

- [ ] **Step 2: Add `files-excludes-file` SAFE_FIX integration case**

After the existing `I7b` block and before `I7c`, add:

```js
// I7b2: packages without exports can still clear public deep-import risk
// when package.json#files explicitly excludes the candidate source file.
rmSync(OUT_D, { recursive: true, force: true });
mkdirSync(path.join(OUT_D, 'src'), { recursive: true });
writeFileSync(path.join(OUT_D, 'package.json'), JSON.stringify({
  name: 'public-files-dist-only',
  version: '1.0.0',
  files: ['dist'],
}, null, 2));
writeFileSync(path.join(OUT_D, 'dead-classify.json'), JSON.stringify({
  summary: { total: 1, category_C: 1 },
  proposal_C_remove_symbol: [
    { file: 'src/internal.js', line: 1, symbol: 'internalThing', kind: 'FunctionDeclaration', action: '' },
  ],
  proposal_A_demote_to_internal: [],
  proposal_B_review: [],
  proposal_remove_export_specifier: [],
}));
writeFileSync(path.join(OUT_D, 'symbols.json'), JSON.stringify({
  totalUsesResolved: 1000, unresolvedUses: 0,
}));
writeFileSync(path.join(OUT_D, 'export-action-safety.json'), JSON.stringify({
  meta: { tool: 'export-action-safety.mjs' },
  findings: [
    {
      id: 'dead-export:src/internal.js:internalThing:1',
      file: 'src/internal.js',
      line: 1,
      symbol: 'internalThing',
      safeAction: safeAction('demote_export_declaration'),
      actionBlockers: [],
    },
  ],
}, null, 2));
execSync(`node rank-fixes.mjs --root ${OUT_D} --output ${OUT_D}`,
  { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });
const planD3 = JSON.parse(readFileSync(path.join(OUT_D, 'fix-plan.json'), 'utf8'));
assert('I7b2. package files exclusion allows SAFE_FIX when other proof is clean',
  planD3.summary.SAFE_FIX === 1 &&
    planD3.summary.REVIEW_FIX === 0 &&
    planD3.safeFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail?.reason === 'files-excludes-file' &&
    planD3.safeFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail?.publishSurfaceSource === 'package-json-files',
  JSON.stringify({
    summary: planD3.summary,
    safe: planD3.safeFixes,
    review: planD3.reviewFixes,
  }));
rmSync(OUT_D, { recursive: true, force: true });
```

- [ ] **Step 3: Add always-included REVIEW_FIX integration case**

After `I7b2`, add:

```js
// I7b3: package.json#files cannot clear public risk for npm always-included
// entrypoint files such as package main.
rmSync(OUT_D, { recursive: true, force: true });
mkdirSync(path.join(OUT_D, 'src'), { recursive: true });
writeFileSync(path.join(OUT_D, 'package.json'), JSON.stringify({
  name: 'public-main-source',
  version: '1.0.0',
  main: './src/index.js',
  files: ['dist'],
}, null, 2));
writeFileSync(path.join(OUT_D, 'dead-classify.json'), JSON.stringify({
  summary: { total: 1, category_C: 1 },
  proposal_C_remove_symbol: [
    { file: 'src/index.js', line: 1, symbol: 'mainThing', kind: 'FunctionDeclaration', action: '' },
  ],
  proposal_A_demote_to_internal: [],
  proposal_B_review: [],
  proposal_remove_export_specifier: [],
}));
writeFileSync(path.join(OUT_D, 'symbols.json'), JSON.stringify({
  totalUsesResolved: 1000, unresolvedUses: 0,
}));
writeFileSync(path.join(OUT_D, 'export-action-safety.json'), JSON.stringify({
  meta: { tool: 'export-action-safety.mjs' },
  findings: [
    {
      id: 'dead-export:src/index.js:mainThing:1',
      file: 'src/index.js',
      line: 1,
      symbol: 'mainThing',
      safeAction: safeAction('demote_export_declaration'),
      actionBlockers: [],
    },
  ],
}, null, 2));
execSync(`node rank-fixes.mjs --root ${OUT_D} --output ${OUT_D}`,
  { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });
const planD4 = JSON.parse(readFileSync(path.join(OUT_D, 'fix-plan.json'), 'utf8'));
assert('I7b3. npm always-included main file keeps REVIEW_FIX',
  planD4.summary.SAFE_FIX === 0 &&
    planD4.summary.REVIEW_FIX === 1 &&
    planD4.summary.reviewReasons?.publicDeepImportRisk?.['exports-absent-file-published-always-included'] === 1 &&
    planD4.reviewFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail?.matchedAlwaysIncludedRule === 'main',
  JSON.stringify({
    summary: planD4.summary,
    safe: planD4.safeFixes,
    review: planD4.reviewFixes,
  }));
rmSync(OUT_D, { recursive: true, force: true });
```

- [ ] **Step 4: Run focused rank test**

Run:

```bash
node tests/test-rank-fixes.mjs
```

Expected: PASS.

- [ ] **Step 5: Commit integration tests**

```bash
git add tests/test-rank-fixes.mjs
git commit -m "test: cover publish-surface rank decisions"
```

---

### Task 4: Focused Verification And Cleanup

**Files:**
- Verify: `_lib/package-exports.mjs`
- Verify: `tests/test-public-deep-import-risk.mjs`
- Verify: `tests/test-rank-fixes.mjs`

- [ ] **Step 1: Run focused tests**

Run:

```bash
node tests/test-public-deep-import-risk.mjs
node tests/test-rank-fixes.mjs
```

Expected: both PASS.

- [ ] **Step 2: Run syntax check and lint**

Run:

```bash
npm run check
npm run lint
```

Expected: both exit 0.

- [ ] **Step 3: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: exit 0 with no whitespace errors.

- [ ] **Step 4: Inspect final diff**

Run:

```bash
git diff --stat HEAD~3..HEAD
git status --short --branch
```

Expected:

- only `_lib/package-exports.mjs`, `tests/test-public-deep-import-risk.mjs`, and `tests/test-rank-fixes.mjs` changed after the plan/spec commit,
- working tree clean,
- branch still `codex/public-deep-import-publish-surface-design`.

- [ ] **Step 5: If verification required small fixes, commit them**

If any fixes were needed, commit them with:

```bash
git add _lib/package-exports.mjs tests/test-public-deep-import-risk.mjs tests/test-rank-fixes.mjs
git commit -m "Fix publish-surface verification issues"
```

If no fixes were needed, do not create an empty commit.

---

## Notes For The Implementer

- This slice must not add dependencies.
- This slice must not run `npm pack`, package manager commands, or filesystem scans to infer packlists.
- `files-excludes-file` clears only public deep-import risk. It does not create deadness proof.
- Unknown, unsupported, included, and always-included cases stay `REVIEW_FIX`.
- Keep helper functions pure and local to `_lib/package-exports.mjs` until a second producer needs them.
- Do not update mirrored skill/package files unless source files under `skills/lumin-repo-lens-lab/_engine` are changed by a separate build step.

## Self-Review Checklist

- Spec coverage:
  - npm always-included files: Task 1 Step 3, Task 2 Step 2, Task 3 Step 3.
  - `files` exclusion and inclusion: Task 1 Step 2, Task 2 Step 3, Task 3 Step 2.
  - unsupported entries fail closed: Task 1 Step 4, Task 2 Step 3.
  - ranking reason codes: Task 3.
- Scope:
  - No full packlist.
  - No `.npmignore`.
  - No package-manager commands.
  - No new dependency.
- Verification:
  - Focused unit tests.
  - Focused rank integration.
  - `npm run check`.
  - `npm run lint`.
  - `git diff --check`.
