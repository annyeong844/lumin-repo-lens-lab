# Function Clone Incremental Cache Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add strict incremental caching to `build-function-clone-index.mjs` without changing clone cue semantics.

**Architecture:** Reuse the existing strict incremental snapshot/cache store. Cache only per-file function clone payloads, then rebuild exact groups, structure groups, and near-function candidates from the current aggregate every run. Keep the implementation producer-local for now; do not introduce a generic incremental producer runner in this slice.

**Tech Stack:** Node.js ESM scripts, OXC AST parser through existing helpers, strict incremental cache helpers in `_lib/incremental-snapshot.mjs` and `_lib/incremental-cache-store.mjs`, custom `node tests/*.mjs` suites.

---

## File Structure

Create:

- `tests/test-function-clone-incremental.mjs`  
  Direct producer TDD regression suite for cold/warm equivalence, changed/deleted files, cache clearing, path identity, current-run `observedAt`, and mixed fresh/reused clone grouping.
- `tests/test-function-clone-audit-forwarding.mjs`
  Audit orchestrator regression suite for forwarding `--no-incremental`, `--cache-root`, and the cache-clear behavior to supported incremental producers.

Modify:

- `_lib/function-clone-artifact.mjs`  
  Split cold artifact construction into per-file extraction and global assembly.
- `build-function-clone-index.mjs`  
  Add strict incremental cache flow and CLI flags.
- `audit-repo.mjs`  
  Clear shared incremental cache once per audit invocation and add `build-function-clone-index.mjs` to the existing incremental producer forwarding set.
- `scripts/update-test-doc.mjs`  
  Add the new test suite description.
- `tests/README.md`  
  Regenerate through `npm run update-test-doc`.
- `skills/lumin-repo-lens-lab/_engine/lib/function-clone-artifact.mjs`  
  Skill mirror from build step.
- `skills/lumin-repo-lens-lab/_engine/producers/build-function-clone-index.mjs`  
  Skill mirror from build step.
- `skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`  
  Skill mirror from build step.

Do not modify:

- `_lib/incremental-cache-store.mjs`
- `_lib/incremental-snapshot.mjs`
- `build-shape-index.mjs`
- clone thresholds, scoring, or review cue wording

---

### Task 1: Write The Failing Function Clone Incremental Test

**Files:**

- Create: `tests/test-function-clone-incremental.mjs`

- [ ] **Step 1: Create the RED test file**

Add this test file. It intentionally expects APIs and CLI behavior that do not exist yet.

```js
import { execFileSync } from 'node:child_process';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const NODE = process.execPath;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const CLI = path.join(ROOT, 'build-function-clone-index.mjs');

let passed = 0;
let failed = 0;

function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

function fresh() {
  return mkdtempSync(path.join(tmpdir(), 'lumin-fn-clone-inc-'));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(root, output, args = []) {
  return execFileSync(NODE, [CLI, '--root', root, '--output', output, ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readIndex(output) {
  return JSON.parse(readFileSync(path.join(output, 'function-clones.json'), 'utf8'));
}

function stripRunMetadata(value) {
  if (Array.isArray(value)) return value.map(stripRunMetadata);
  if (value && typeof value === 'object') {
    const out = {};
    for (const [key, child] of Object.entries(value)) {
      if (key === 'generated' || key === 'observedAt' || key === 'incremental') continue;
      out[key] = stripRunMetadata(child);
    }
    return out;
  }
  return value;
}

function stableIndex(index) {
  return stripRunMetadata(index);
}

function setupRepo(repo) {
  write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
  write(repo, 'src/money-a.ts',
    `export function formatCurrencyCents(cents: number, currency = 'USD') {\n` +
    `  const dollars = cents / 100;\n` +
    `  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);\n` +
    `}\n`);
  write(repo, 'src/money-b.ts',
    `export function renderPaymentTotal(value: number, unit = 'USD') {\n` +
    `  const amount = value / 100;\n` +
    `  return new Intl.NumberFormat('en-US', { style: 'currency', currency: unit }).format(amount);\n` +
    `}\n`);
  write(repo, 'src/exact-a.ts',
    `export const parseOne = (raw: string) => {\n` +
    `  const value = raw.trim();\n` +
    `  return value.toUpperCase();\n` +
    `};\n`);
  write(repo, 'src/exact-b.ts',
    `const local = (raw: string) => {\n` +
    `  const value = raw.trim();\n` +
    `  return value.toUpperCase();\n` +
    `};\n` +
    `export { local as parseTwo };\n`);
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);

    run(repo, output, ['--no-incremental']);
    const cold = readIndex(output);
    run(repo, output);
    const firstIncremental = readIndex(output);
    run(repo, output);
    const warm = readIndex(output);

    assert('function-clones incremental equals cold public artifact',
      JSON.stringify(stableIndex(firstIncremental)) === JSON.stringify(stableIndex(cold)));
    assert('warm function-clones equals cold public artifact',
      JSON.stringify(stableIndex(warm)) === JSON.stringify(stableIndex(cold)));
    assert('warm function-clones reports strict incremental enabled',
      warm.meta.incremental?.enabled === true &&
        warm.meta.incremental?.identityMode === 'strict-content-hash',
      JSON.stringify(warm.meta.incremental));
    assert('warm function-clones reused unchanged file payloads',
      warm.meta.incremental?.reusedFiles >= 4,
      JSON.stringify(warm.meta.incremental));
    assert('warm reused facts are stamped with current artifact observedAt',
      warm.facts.every((fact) => fact.observedAt === warm.meta.observedAt),
      JSON.stringify(warm.facts.map((fact) => ({
        identity: fact.identity,
        factObservedAt: fact.observedAt,
        metaObservedAt: warm.meta.observedAt,
      }))));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);

    write(repo, 'src/money-b.ts',
      `export function renderPaymentTotal(value: number, unit = 'USD') {\n` +
      `  const amount = value / 100;\n` +
      `  return new Intl.NumberFormat('en-GB', { style: 'currency', currency: unit }).format(amount);\n` +
      `}\n`);
    run(repo, output);
    const index = readIndex(output);
    const changed = index.facts.find((f) => f.identity === 'src/money-b.ts::renderPaymentTotal');
    const unchanged = index.facts.find((f) => f.identity === 'src/money-a.ts::formatCurrencyCents');

    assert('changed file refreshes function clone fact',
      changed?.exactBodyHash && unchanged?.exactBodyHash &&
        changed.exactBodyHash !== unchanged.exactBodyHash,
      JSON.stringify(index.facts));
    assert('changed run reuses unchanged function clone files',
      index.meta.incremental?.changedFiles >= 1 &&
        index.meta.incremental?.reusedFiles >= 1,
      JSON.stringify(index.meta.incremental));
    assert('changed file does not count as dropped',
      index.meta.incremental?.droppedFiles === 0,
      JSON.stringify(index.meta.incremental));

    const coldAfterChangeOutput = path.join(repo, '.audit-cold-after-change');
    run(repo, coldAfterChangeOutput, ['--no-incremental']);
    const coldAfterChange = readIndex(coldAfterChangeOutput);
    assert('changed incremental artifact equals cold artifact after same change',
      JSON.stringify(stableIndex(index)) === JSON.stringify(stableIndex(coldAfterChange)));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);

    write(repo, 'src/new-exact-c.ts',
      `export const parseThree = (raw: string) => {\n` +
      `  const value = raw.trim();\n` +
      `  return value.toUpperCase();\n` +
      `};\n`);
    run(repo, output);
    const index = readIndex(output);
    const matchingGroup = (index.exactBodyGroups ?? []).find((group) =>
      (group.members ?? []).some((m) => m.identity === 'src/exact-a.ts::parseOne') &&
      (group.members ?? []).some((m) => m.identity === 'src/new-exact-c.ts::parseThree'));

    assert('global clone groups rebuild from mixed fresh and reused facts',
      !!matchingGroup,
      JSON.stringify(index.exactBodyGroups));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);

    rmSync(path.join(repo, 'src/exact-b.ts'), { force: true });
    run(repo, output);
    const index = readIndex(output);

    assert('deleted file function clone facts disappear',
      !index.facts.some((f) => f.ownerFile === 'src/exact-b.ts'),
      JSON.stringify(index.facts));
    assert('deleted file contributes function clone dropped count',
      index.meta.incremental?.droppedFiles >= 1,
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);

    rmSync(path.join(repo, 'src/money-a.ts'), { force: true });
    write(repo, 'src/moved-money-a.ts',
      `export function formatCurrencyCents(cents: number, currency = 'USD') {\n` +
      `  const dollars = cents / 100;\n` +
      `  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);\n` +
      `}\n`);
    run(repo, output);
    const index = readIndex(output);

    assert('moved file with same content is treated as changed under relPath identity',
      index.meta.incremental?.changedFiles >= 1 &&
        index.meta.incremental?.droppedFiles >= 1 &&
        index.facts.some((f) => f.identity === 'src/moved-money-a.ts::formatCurrencyCents'),
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);
    run(repo, output, ['--clear-incremental-cache']);
    const index = readIndex(output);
    assert('--clear-incremental-cache clears function clone cache before run',
      index.meta.incremental?.enabled === true &&
        index.meta.incremental?.reusedFiles === 0 &&
        index.meta.incremental?.changedFiles >= 4,
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output, ['--no-incremental']);
    const index = readIndex(output);
    assert('--no-incremental reports disabled function clone cache',
      index.meta.incremental?.enabled === false &&
        index.meta.incremental?.reason === 'disabled-by-flag',
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
```

- [ ] **Step 2: Run the RED test and verify it fails**

Run:

```bash
node tests/test-function-clone-incremental.mjs
```

Expected: at least the incremental assertions fail because `function-clones.json` has no `meta.incremental` yet.

- [ ] **Step 3: Commit only the RED test**

```bash
git add tests/test-function-clone-incremental.mjs
git commit -m "test: cover function clone incremental cache"
```

---

### Task 2: Split Function Clone Artifact Into Per-File Payload And Global Assembly

**Files:**

- Modify: `_lib/function-clone-artifact.mjs`
- Test: `tests/test-build-function-clone-index.mjs`
- Test: `tests/test-function-clone-incremental.mjs`

- [ ] **Step 1: Export a read-error payload helper**

In `_lib/function-clone-artifact.mjs`, add this exported helper near the existing diagnostic helpers:

```js
export function functionCloneReadErrorPayload(relFile, message) {
  return {
    facts: [],
    diagnostics: [readErrorDiagnostic(relFile, `read failed: ${message}`)],
    filesWithParseErrors: [],
    filesWithReadErrors: [{ file: relFile, message }],
  };
}
```

- [ ] **Step 2: Export per-file extraction without run metadata**

Add this function after `functionCloneReadErrorPayload`. Do not accept `observedAt`.

```js
export function extractFunctionCloneFilePayload({ src, relFile, scope }) {
  let parsed;
  try {
    parsed = parseOxcOrThrow(relFile, src);
  } catch (e) {
    return {
      facts: [],
      diagnostics: [parseErrorDiagnostic(relFile, e.message)],
      filesWithParseErrors: [{ file: relFile, message: e.message }],
      filesWithReadErrors: [],
    };
  }

  const lineStarts = computeLineStarts(src);
  const facts = [];
  for (const entry of topLevelExportedFunctions(parsed.program)) {
    const fact = buildFunctionFact({
      entry,
      src,
      ownerFile: relFile,
      lineStarts,
      scope,
    });
    if (fact) facts.push(fact);
  }

  return {
    facts,
    diagnostics: [],
    filesWithParseErrors: [],
    filesWithReadErrors: [],
  };
}
```

- [ ] **Step 3: Make `buildFunctionFact` stop storing `observedAt`**

Change the function signature and return object:

```js
function buildFunctionFact({ entry, src, ownerFile, lineStarts, scope }) {
  // keep existing body
  return {
    // keep existing fields
    source: 'fresh-ast-pass',
    scope,
    confidence: 'high',
    ...(generatedFile ? { generatedFile } : {}),
  };
}
```

Remove the `observedAt` property from this per-file fact. It will be added during assembly.

- [ ] **Step 4: Add global assembly that stamps current-run metadata**

Add this exported function before `buildFunctionCloneArtifact(...)`:

```js
export function assembleFunctionCloneArtifact({
  metaBase,
  includeTests,
  exclude,
  scope,
  observedAt,
  fileCount,
  facts,
  diagnostics,
  filesWithParseErrors,
  filesWithReadErrors,
  incremental = null,
}) {
  const stampedFacts = (facts ?? []).map((fact) => ({
    ...fact,
    observedAt,
  }));

  stampedFacts.sort((a, b) => {
    if (a.ownerFile !== b.ownerFile) return a.ownerFile < b.ownerFile ? -1 : 1;
    if ((a.line ?? 0) !== (b.line ?? 0)) return (a.line ?? 0) - (b.line ?? 0);
    return a.exportedName.localeCompare(b.exportedName);
  });
  const sortedDiagnostics = (diagnostics ?? []).slice().sort((a, b) =>
    (a.file ?? '').localeCompare(b.file ?? '') ||
    (a.code ?? '').localeCompare(b.code ?? ''));

  const exactBodyGroups = groupFacts(stampedFacts, 'normalizedExactHash');
  const structureGroups = groupFacts(stampedFacts, 'normalizedStructureHash');
  const nearFunctionCandidates = buildNearFunctionCandidates(
    stampedFacts,
    exactBodyGroups,
    structureGroups
  );
  const generatedFileFactCount = stampedFacts.filter((fact) => fact.generatedFile).length;

  return {
    schemaVersion: FUNCTION_CLONE_SCHEMA_VERSION,
    meta: {
      ...metaBase,
      source: 'fresh-ast-pass',
      scope,
      observedAt,
      complete: filesWithReadErrors.length === 0 && filesWithParseErrors.length === 0,
      includeTests: includeTests === true,
      exclude: exclude ?? [],
      fileCount,
      factCount: stampedFacts.length,
      generatedFileFactCount,
      exactBodyGroupCount: exactBodyGroups.filter((g) => !g.generatedOnly).length,
      structureGroupCount: structureGroups.filter((g) => !g.generatedOnly).length,
      nearFunctionCandidateCount: nearFunctionCandidates.filter((g) => !g.generatedOnly).length,
      diagnosticCount: sortedDiagnostics.length,
      filesWithParseErrors,
      filesWithReadErrors,
      ...(incremental ? { incremental } : {}),
      supports: {
        exportedTopLevelFunctions: true,
        exportedConstArrowFunctions: true,
        defaultFunctionExports: true,
        exactBodyHash: true,
        normalizedExactHash: true,
        normalizedStructureHash: true,
        normalizedVersion: FUNCTION_CLONE_NORMALIZED_VERSION,
        nearFunctionCandidates: true,
        generatedFileEvidence: true,
        semanticEquivalence: false,
      },
      caveat: 'Function clone groups and near candidates are deterministic review cues. They do not prove semantic equivalence or justify automatic merging.',
    },
    facts: stampedFacts,
    exactBodyGroups,
    structureGroups,
    nearFunctionCandidates,
    diagnostics: sortedDiagnostics,
  };
}
```

- [ ] **Step 5: Rewrite cold wrapper to use extraction + assembly**

Replace the body of `buildFunctionCloneArtifact(...)` with the wrapper pattern:

```js
export function buildFunctionCloneArtifact({
  root,
  files,
  readFile,
  metaBase,
  includeTests,
  exclude,
  scope,
  observedAt,
}) {
  const aggregate = {
    facts: [],
    diagnostics: [],
    filesWithParseErrors: [],
    filesWithReadErrors: [],
  };

  function appendPayload(payload) {
    aggregate.facts.push(...(payload.facts ?? []));
    aggregate.diagnostics.push(...(payload.diagnostics ?? []));
    aggregate.filesWithParseErrors.push(...(payload.filesWithParseErrors ?? []));
    aggregate.filesWithReadErrors.push(...(payload.filesWithReadErrors ?? []));
  }

  for (const abs of files) {
    const relFile = toRel(root, abs);
    let src;
    try {
      src = readFile(abs, 'utf8');
    } catch (e) {
      appendPayload(functionCloneReadErrorPayload(relFile, e.message));
      continue;
    }

    appendPayload(extractFunctionCloneFilePayload({
      src,
      relFile,
      scope,
    }));
  }

  return assembleFunctionCloneArtifact({
    metaBase,
    includeTests,
    exclude,
    scope,
    observedAt,
    fileCount: files.length,
    ...aggregate,
  });
}
```

- [ ] **Step 6: Run existing clone tests**

Run:

```bash
node tests/test-build-function-clone-index.mjs
```

Expected: PASS. If this fails, the split changed existing artifact semantics.

- [ ] **Step 7: Run RED incremental test again**

Run:

```bash
node tests/test-function-clone-incremental.mjs
```

Expected: still FAIL on incremental metadata/flag behavior. It may have fewer failures than Task 1.

- [ ] **Step 8: Commit the artifact split**

```bash
git add _lib/function-clone-artifact.mjs
git commit -m "refactor: split function clone file facts from grouping"
```

---

### Task 3: Add Strict Incremental Cache To build-function-clone-index.mjs

**Files:**

- Modify: `build-function-clone-index.mjs`
- Test: `tests/test-function-clone-incremental.mjs`
- Test: `tests/test-build-function-clone-index.mjs`

- [ ] **Step 1: Replace producer imports**

In `build-function-clone-index.mjs`, replace the old cold builder import with split artifact functions and incremental helpers:

```js
import {
  assembleFunctionCloneArtifact,
  extractFunctionCloneFilePayload,
  functionCloneReadErrorPayload,
} from './_lib/function-clone-artifact.mjs';
import {
  buildContextFingerprint,
  buildRepoSnapshot,
  STRICT_IDENTITY_MODE,
} from './_lib/incremental-snapshot.mjs';
import {
  clearIncrementalCache,
  getReusableFact,
  loadProducerCache,
  openIncrementalCacheStore,
  putFact,
  saveProducerCache,
} from './_lib/incremental-cache-store.mjs';
```

- [ ] **Step 2: Extend CLI flags and add producer identity constants**

Extend the existing `parseCliArgs(...)` option object. Do not remove existing `root`, `output`, `includeTests`, `exclude`, or production/test behavior.

```js
const cli = parseCliArgs({
  'no-incremental': { type: 'boolean', default: false },
  'cache-root': { type: 'string' },
  'clear-incremental-cache': { type: 'boolean', default: false },
});
const ROOT = cli.root;
const OUTPUT = cli.output;

const PRODUCER_ID = 'function-clones';
const PRODUCER_VERSION = 1;
const FACT_SCHEMA_VERSION = 1;
const PARSER_IDENTITY = 'function-clones:oxc-parser+normalizer+scoring-v1';
```

Keep the exact string stable. It deliberately covers parser support plus clone normalization/scoring semantics. If clone normalization/scoring changes later, bump one of these identity values instead of reusing old cache entries.

- [ ] **Step 3: Replace `collectFiles(...)` with a strict repo snapshot**

Use the same scan options and JS language set:

```js
const contextFingerprint = buildContextFingerprint({
  includeTests: cli.includeTests,
  exclude: cli.exclude ?? [],
  languages: JS_FAMILY_LANGS,
  producerContext: {
    producer: PRODUCER_ID,
    producerVersion: PRODUCER_VERSION,
    factSchemaVersion: FACT_SCHEMA_VERSION,
    parserIdentity: PARSER_IDENTITY,
  },
});

const snapshot = buildRepoSnapshot({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  languages: JS_FAMILY_LANGS,
  contextFingerprint,
});
const snapshotEntries = Object.values(snapshot.files);
```

- [ ] **Step 4: Add cache open/load/clear setup**

Place this after `scope` is computed:

```js
const incrementalEnabled = cli.raw?.['no-incremental'] !== true;
const cacheStore = openIncrementalCacheStore({
  root: ROOT,
  cacheRoot: cli.raw?.['cache-root'],
});
if (cli.raw?.['clear-incremental-cache'] === true) {
  clearIncrementalCache(cacheStore);
}

const producerCacheMeta = {
  producerId: PRODUCER_ID,
  producerVersion: PRODUCER_VERSION,
  factSchemaVersion: FACT_SCHEMA_VERSION,
  parserIdentity: PARSER_IDENTITY,
  scanFingerprint: contextFingerprint,
  configFingerprint: contextFingerprint,
};
const priorCache = incrementalEnabled
  ? loadProducerCache(cacheStore, PRODUCER_ID)
  : { entries: {}, meta: { loadStatus: 'disabled' } };
const nextCache = { entries: {}, meta: { loadStatus: 'new' } };
const currentRelPaths = new Set();
```

- [ ] **Step 5: Replace cold artifact build with incremental extraction loop**

Replace the `files = collectFiles(...)` plus `buildFunctionCloneArtifact(...)` call with:

```js
const aggregate = {
  facts: [],
  diagnostics: [],
  filesWithParseErrors: [],
  filesWithReadErrors: [],
};
let changedFiles = 0;
let reusedFiles = 0;
let invalidatedFiles = 0;

function appendPayload(payload) {
  aggregate.facts.push(...(payload.facts ?? []));
  aggregate.diagnostics.push(...(payload.diagnostics ?? []));
  aggregate.filesWithParseErrors.push(...(payload.filesWithParseErrors ?? []));
  aggregate.filesWithReadErrors.push(...(payload.filesWithReadErrors ?? []));
}

for (const entry of snapshotEntries) {
  currentRelPaths.add(entry.relPath);

  if (!entry.readable) {
    changedFiles++;
    appendPayload(functionCloneReadErrorPayload(
      entry.relPath,
      entry.readError?.message ?? entry.readError?.kind ?? 'unknown'
    ));
    continue;
  }

  const reuse = incrementalEnabled
    ? getReusableFact(priorCache, { snapshotEntry: entry, producerMeta: producerCacheMeta })
    : { status: 'miss', reason: 'disabled-by-flag' };

  if (reuse.status === 'hit') {
    reusedFiles++;
    appendPayload(reuse.payload);
    putFact(nextCache, {
      snapshotEntry: entry,
      producerMeta: producerCacheMeta,
      payload: reuse.payload,
    });
    continue;
  }

  if (reuse.reason !== 'missing-entry' && reuse.reason !== 'disabled-by-flag') {
    invalidatedFiles++;
  }
  changedFiles++;

  let src;
  try {
    src = readFileSync(entry.absPath, 'utf8');
  } catch (e) {
    appendPayload(functionCloneReadErrorPayload(entry.relPath, e.message));
    continue;
  }

  const payload = extractFunctionCloneFilePayload({
    src,
    relFile: entry.relPath,
    scope,
  });
  appendPayload(payload);
  if (incrementalEnabled) {
    putFact(nextCache, {
      snapshotEntry: entry,
      producerMeta: producerCacheMeta,
      payload,
    });
  }
}

const droppedFiles = Object.keys(priorCache.entries ?? {})
  .map((key) => priorCache.entries[key]?.identity?.relPath)
  .filter((relPath) => relPath && !currentRelPaths.has(relPath)).length;
if (incrementalEnabled) {
  saveProducerCache(cacheStore, PRODUCER_ID, nextCache);
}

const artifact = assembleFunctionCloneArtifact({
  metaBase,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  scope,
  observedAt: metaBase.generated,
  fileCount: snapshotEntries.length,
  ...aggregate,
  incremental: {
    enabled: incrementalEnabled,
    identityMode: incrementalEnabled ? STRICT_IDENTITY_MODE : null,
    cacheVersion: 1,
    cacheRoot: incrementalEnabled ? cacheStore.cacheRoot : null,
    changedFiles,
    reusedFiles,
    droppedFiles,
    invalidatedFiles,
    reason: incrementalEnabled ? null : 'disabled-by-flag',
  },
});
```

`droppedFiles` must be computed by prior normalized path membership, not by strict cache key membership. A changed file has a different strict identity but is still present in the current scan; it should count as changed or invalidated, not dropped.

- [ ] **Step 6: Update console output**

Keep existing summary and add the incremental line:

```js
if (incrementalEnabled) {
  console.log(
    `[function-clones] incremental: ${changedFiles} changed, ${reusedFiles} reused, ` +
    `${droppedFiles} dropped, ${invalidatedFiles} invalidated`
  );
}
console.log(`[function-clones] saved -> ${outPath}`);
```

- [ ] **Step 7: Remove unused imports**

`build-function-clone-index.mjs` should no longer import `collectFiles` or `buildFunctionCloneArtifact`.

- [ ] **Step 8: Run focused tests**

Run:

```bash
node tests/test-function-clone-incremental.mjs
node tests/test-build-function-clone-index.mjs
```

Expected: both PASS.

- [ ] **Step 9: Commit the producer implementation**

```bash
git add build-function-clone-index.mjs _lib/function-clone-artifact.mjs tests/test-function-clone-incremental.mjs
git commit -m "feat: cache function clone file facts strictly"
```

---

### Task 4: Forward Incremental Flags From audit-repo.mjs

**Files:**

- Modify: `audit-repo.mjs`
- Create: `tests/test-function-clone-audit-forwarding.mjs`
- Test: `tests/test-audit-repo.mjs`

- [ ] **Step 1: Add the audit forwarding regression**

Create `tests/test-function-clone-audit-forwarding.mjs`. Keep this separate from the direct producer test so Task 3 can go GREEN before the orchestrator is changed.

```js
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const NODE = process.execPath;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const AUDIT = path.join(ROOT, 'audit-repo.mjs');

let passed = 0;
let failed = 0;

function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

function fresh() {
  return mkdtempSync(path.join(tmpdir(), 'lumin-fn-clone-audit-forward-'));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function setupRepo(repo) {
  write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
  write(repo, 'src/a.ts',
    `export const runA = () => {\n` +
    `  const value = 'a'.trim();\n` +
    `  return value.toUpperCase();\n` +
    `};\n`);
  write(repo, 'src/b.ts',
    `export const runB = () => {\n` +
    `  const value = 'b'.trim();\n` +
    `  return value.toUpperCase();\n` +
    `};\n`);
}

function runAudit(root, output, args = []) {
  return execFileSync(NODE, [AUDIT, '--root', root, '--output', output, ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readIndex(output) {
  return JSON.parse(readFileSync(path.join(output, 'function-clones.json'), 'utf8'));
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    runAudit(repo, output, ['--profile', 'full', '--no-incremental']);
    const index = readIndex(output);
    assert('audit-repo forwards --no-incremental to function clone producer',
      index.meta.incremental?.enabled === false &&
        index.meta.incremental?.reason === 'disabled-by-flag',
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    const cacheRoot = path.join(repo, 'cache root with spaces');
    runAudit(repo, output, ['--profile', 'full', '--cache-root', cacheRoot]);
    runAudit(repo, output, ['--profile', 'full', '--cache-root', cacheRoot]);
    const warmIndex = readIndex(output);
    assert('audit-repo forwards --cache-root to function clone producer',
      warmIndex.meta.incremental?.enabled === true &&
        path.resolve(warmIndex.meta.incremental.cacheRoot) === path.resolve(cacheRoot) &&
        warmIndex.meta.incremental.reusedFiles >= 2,
      JSON.stringify(warmIndex.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    const cacheRoot = path.join(repo, 'cache root with spaces');
    runAudit(repo, output, ['--profile', 'full', '--cache-root', cacheRoot]);
    runAudit(repo, output, ['--profile', 'full', '--cache-root', cacheRoot]);
    runAudit(repo, output, ['--profile', 'full', '--cache-root', cacheRoot, '--clear-incremental-cache']);
    const clearedIndex = readIndex(output);
    assert('audit-repo clears shared incremental cache once before supported producers run',
      clearedIndex.meta.incremental?.enabled === true &&
        clearedIndex.meta.incremental.reusedFiles === 0 &&
        clearedIndex.meta.incremental.changedFiles >= 2,
      JSON.stringify(clearedIndex.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
```

- [ ] **Step 2: Clear shared cache once in audit-repo**

When `audit-repo.mjs` receives `--clear-incremental-cache`, it must clear the shared repo cache root at most once before supported producers run. Do not forward the whole-store clear flag to every incremental producer; that can delete cache files a previous producer wrote earlier in the same audit.

Add `openIncrementalCacheStore` and `clearIncrementalCache` imports from `_lib/incremental-cache-store.mjs`, then clear once after `ROOT` is known and before the base pipeline starts:

```js
if (values['clear-incremental-cache'] === true) {
  const cacheStore = openIncrementalCacheStore({
    root: ROOT,
    cacheRoot: values['cache-root'],
  });
  clearIncrementalCache(cacheStore);
}
```

Then update `forwardedIncrementalArgs()` so it forwards `--no-incremental` and `--cache-root`, but not `--clear-incremental-cache`.

- [ ] **Step 3: Add function clone producer to the forwarding set**

In `audit-repo.mjs`, change:

```js
const INCREMENTAL_PRODUCER_STEPS = new Set([
  'build-symbol-graph.mjs',
  'build-shape-index.mjs',
]);
```

to:

```js
const INCREMENTAL_PRODUCER_STEPS = new Set([
  'build-symbol-graph.mjs',
  'build-shape-index.mjs',
  'build-function-clone-index.mjs',
]);
```

- [ ] **Step 4: Run audit forwarding regression**

Run:

```bash
node tests/test-function-clone-audit-forwarding.mjs
node tests/test-audit-repo.mjs
```

Expected: both PASS.

- [ ] **Step 5: Commit audit forwarding**

```bash
git add audit-repo.mjs tests/test-function-clone-audit-forwarding.mjs
git commit -m "feat: forward incremental flags to function clone producer"
```

---

### Task 5: Update Test Documentation

**Files:**

- Modify: `scripts/update-test-doc.mjs`
- Modify: `tests/README.md`

- [ ] **Step 1: Add suite description**

In `scripts/update-test-doc.mjs`, add this entry next to the other incremental suite descriptions:

```js
'test-function-clone-incremental.mjs': 'strict incremental build-function-clone-index cold/warm equivalence + changed/deleted file behavior',
'test-function-clone-audit-forwarding.mjs': 'audit-repo incremental flag forwarding for function clone producer',
```

- [ ] **Step 2: Regenerate tests README**

Run:

```bash
npm run update-test-doc
```

Expected: `tests/README.md` updates with one new suite line.

- [ ] **Step 3: Verify generated docs are current**

Run:

```bash
npm run check:test-doc
```

Expected: PASS.

- [ ] **Step 4: Commit docs**

```bash
git add scripts/update-test-doc.mjs tests/README.md
git commit -m "docs: list function clone incremental test"
```

---

### Task 6: Rebuild The Shipping Skill Mirror

**Files:**

- Modify: `skills/lumin-repo-lens-lab/_engine/lib/function-clone-artifact.mjs`
- Modify: `skills/lumin-repo-lens-lab/_engine/producers/build-function-clone-index.mjs`
- Modify: `skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`

- [ ] **Step 1: Build skill package mirror**

Run:

```bash
npm run build:skill
```

Expected: skill mirror files update. No source files outside the expected mirror should be unexpectedly rewritten.

- [ ] **Step 2: Check mirror diff**

Run:

```bash
git diff --stat -- skills/lumin-repo-lens-lab/_engine/lib/function-clone-artifact.mjs skills/lumin-repo-lens-lab/_engine/producers/build-function-clone-index.mjs skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs
```

Expected: only these three mirror files show the producer/lib changes from earlier tasks.

- [ ] **Step 3: Run focused mirror-sensitive tests**

Run:

```bash
node tests/test-skill-package.mjs
node tests/test-plugin-package.mjs
```

Expected: both PASS.

- [ ] **Step 4: Commit mirror update**

```bash
git add skills/lumin-repo-lens-lab/_engine/lib/function-clone-artifact.mjs skills/lumin-repo-lens-lab/_engine/producers/build-function-clone-index.mjs skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs
git commit -m "build: mirror function clone incremental cache"
```

---

### Task 7: Final Verification And PR Preparation

**Files:**

- No planned source modifications.

- [ ] **Step 1: Run focused regression suite**

Run:

```bash
node tests/test-function-clone-incremental.mjs
node tests/test-function-clone-audit-forwarding.mjs
node tests/test-build-function-clone-index.mjs
node tests/test-shape-index-incremental.mjs
node tests/test-symbol-graph-incremental.mjs
node tests/test-audit-repo.mjs
npm run check:test-doc
```

Expected: all PASS.

- [ ] **Step 2: Run full CI**

Run:

```bash
npm run ci
```

Expected: PASS. Existing Windows line-ending warnings are acceptable only if the command exits 0.

- [ ] **Step 3: Run diff hygiene**

Run:

```bash
git diff --check
```

Expected: exit 0. Existing line-ending warnings are acceptable only if the command exits 0.

- [ ] **Step 4: Confirm final status**

Run:

```bash
git status --short --branch
git log --oneline -5
```

Expected: current branch contains only intentional commits and no unstaged changes.

- [ ] **Step 5: Open a draft PR**

Use the established repository flow:

```bash
git push -u origin "$(git branch --show-current)"
```

Open a draft PR against `annyeong844/lumin_lab:main` with this validation list:

```md
## Validation

- `node tests/test-function-clone-incremental.mjs`
- `node tests/test-function-clone-audit-forwarding.mjs`
- `node tests/test-build-function-clone-index.mjs`
- `node tests/test-shape-index-incremental.mjs`
- `node tests/test-symbol-graph-incremental.mjs`
- `node tests/test-audit-repo.mjs`
- `npm run check:test-doc`
- `npm run ci`
- `git diff --check`
```

Expected: GitHub CI starts for Node 20 and Node 22.

---

## Self-Review Checklist

- The plan implements the approved design in `docs/superpowers/specs/2026-05-04-function-clone-incremental-cache-design.md`.
- Per-file cached payloads do not contain `observedAt` or other run-scoped metadata.
- Global clone groups and near candidates are rebuilt every run.
- No generic incremental runner is introduced.
- Function clone semantics and review wording are unchanged.
- New test starts RED before implementation.
- Focused tests and full CI are specified before PR.
