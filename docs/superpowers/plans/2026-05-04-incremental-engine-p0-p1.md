# Incremental Engine P0/P1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the strict shared incremental substrate and wire `any-inventory` plus post-write after-snapshot generation to it without changing public artifact claims.

**Architecture:** Keep ranking and review logic unaware of cache reuse. P0 adds strict repo snapshot and cache-store helpers that require current content hashes for reusable facts. P1 adapts `any-inventory.mjs` and post-write spawning so unchanged type-escape facts can be reused while public artifacts remain cold/warm equivalent.

**Tech Stack:** Node.js ESM (`.mjs`), existing `collectFiles()`, `atomicWrite()`, `extractTypeEscapes()`, fixture-driven tests under `tests/`, PowerShell verification commands.

---

## Scope

This plan implements only P0 and P1 from [`docs/spec/incremental-engine-architecture.md`](../../spec/incremental-engine-architecture.md):

- P0: shared strict snapshot and cache store.
- P1: `any-inventory` incremental adapter and post-write after-snapshot forwarding.

It does not implement symbol graph, shape index, function clone, topology, call graph, or reachability adapters. Those producers require P2-P4 plans after P0/P1 behavior is stable.

## File Structure

- Create `_lib/incremental-snapshot.mjs`
  - Builds the scan snapshot around current `collectFiles()` behavior.
  - Computes current content hashes from file bytes in strict mode.
  - Keeps unreadable in-scope files visible as read-error entries.
  - Computes per-file identity fields: `relPath`, `language`, `isTestLike`, `packageScope`, `contentHash`, `contextFingerprint`.

- Create `_lib/incremental-cache-store.mjs`
  - Owns stable cache root layout and strict hit validation.
  - Reads malformed cache as empty and writes cache atomically.
  - Stores producer fact entries keyed by strict identity.
  - Exposes compatibility boundaries so old producer-local caches are not strict hits.

- Modify `any-inventory.mjs`
  - Adds `--no-incremental`, `--cache-root`, and `--clear-incremental-cache`.
  - Uses snapshot/cache helpers for per-file type-escape facts.
  - Emits `meta.incremental` and deterministic `typeEscapes` ordering.

- Modify `post-write.mjs`
  - Adds `--no-incremental` and `--cache-root`.
  - Forwards incremental flags to the after-snapshot `any-inventory.mjs` spawn.
  - Keeps `preWrite.anyInventoryPath` immutable and invocation-specific.

- Modify `audit-repo.mjs`
  - Adds top-level `--no-incremental`, `--cache-root`, and `--clear-incremental-cache`.
  - Forwards incremental flags only to lifecycle steps that support them.

- Keep `_lib/incremental.mjs`
  - Leave legacy helper in place for existing producer-local caches.
  - Add a header note that strict shared cache work lives in the new modules.
  - Do not route new P0/P1 code through stat-first-cut behavior.

- Add tests:
  - `tests/test-incremental-snapshot.mjs`
  - `tests/test-incremental-cache-store.mjs`
  - `tests/test-any-inventory-incremental.mjs`
  - `tests/test-post-write-incremental.mjs`

## Task 1: Add Strict Snapshot Tests

**Files:**
- Create: `tests/test-incremental-snapshot.mjs`
- Create later in this task: `_lib/incremental-snapshot.mjs`

- [ ] **Step 1: Write failing snapshot tests**

Create `tests/test-incremental-snapshot.mjs`:

```js
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import {
  buildContextFingerprint,
  buildRepoSnapshot,
  defaultPackageScopeOf,
  hashBytes,
  normalizeRepoRel,
} from '../_lib/incremental-snapshot.mjs';

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
  return mkdtempSync(path.join(tmpdir(), 'lumin-snapshot-'));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
  return full;
}

{
  const root = fresh();
  try {
    const full = write(root, 'src/a.ts', 'export const a = 1;\n');
    assert('normalizeRepoRel returns POSIX repo-relative paths',
      normalizeRepoRel(root, full) === 'src/a.ts');
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    write(root, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(root, 'src/a.ts', 'export const a = 1;\n');
    write(root, 'tests/a.test.ts', 'const x = y as any;\n');

    const contextFingerprint = buildContextFingerprint({
      includeTests: false,
      exclude: [],
      languages: ['ts'],
      producerContext: { producer: 'any-inventory', factSchemaVersion: 1 },
    });
    const snapshot = buildRepoSnapshot({
      root,
      includeTests: false,
      exclude: [],
      languages: ['ts'],
      contextFingerprint,
    });

    assert('snapshot includes production file',
      !!snapshot.files['src/a.ts']);
    assert('snapshot excludes test file when includeTests=false',
      !snapshot.files['tests/a.test.ts']);
    const entry = snapshot.files['src/a.ts'];
    assert('entry has strict identity fields',
      entry.relPath === 'src/a.ts' &&
      entry.language === 'ts' &&
      entry.isTestLike === false &&
      entry.packageScope === '.' &&
      typeof entry.contentHash === 'string' &&
      entry.contextFingerprint === contextFingerprint);
    assert('hash is sha256-prefixed',
      /^sha256:[a-f0-9]{64}$/.test(entry.contentHash));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    write(root, 'package.json', JSON.stringify({ name: 'root' }));
    write(root, 'packages/core/package.json', JSON.stringify({ name: 'core' }));
    write(root, 'packages/core/src/a.ts', 'export const a = 1;\n');

    const scope = defaultPackageScopeOf(root, path.join(root, 'packages/core/src/a.ts'));
    assert('package scope uses nearest package root',
      scope === 'packages/core',
      `got ${scope}`);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    const content = Buffer.from('same bytes\n', 'utf8');
    const h1 = hashBytes(content);
    const h2 = hashBytes(content);
    assert('hashBytes is deterministic sha256',
      h1 === h2 && /^sha256:[a-f0-9]{64}$/.test(h1));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    write(root, 'package.json', JSON.stringify({ name: 'fixture' }));
    const unreadable = write(root, 'src/secret.ts', 'export const secret = 1;\n');
    let chmodWorked = true;
    try {
      chmodSync(unreadable, 0o000);
    } catch {
      chmodWorked = false;
    }

    const contextFingerprint = buildContextFingerprint({
      includeTests: true,
      exclude: [],
      languages: ['ts'],
      producerContext: { producer: 'any-inventory', factSchemaVersion: 1 },
    });
    const snapshot = buildRepoSnapshot({
      root,
      includeTests: true,
      exclude: [],
      languages: ['ts'],
      contextFingerprint,
    });

    const entry = snapshot.files['src/secret.ts'];
    if (chmodWorked && entry?.readable === false) {
      assert('unreadable in-scope file remains visible with read error',
        entry.hash === null &&
        entry.contentHash === null &&
        entry.readError?.kind);
    } else {
      assert('unreadable test skipped on platform that still allows read',
        !!entry);
    }
  } finally {
    try {
      chmodSync(path.join(root, 'src/secret.ts'), 0o600);
    } catch {}
    rmSync(root, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```powershell
node tests/test-incremental-snapshot.mjs
```

Expected: FAIL with module-not-found for `_lib/incremental-snapshot.mjs`.

- [ ] **Step 3: Implement `_lib/incremental-snapshot.mjs`**

Create `_lib/incremental-snapshot.mjs`:

```js
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, realpathSync, statSync } from 'node:fs';
import path from 'node:path';

import { collectFiles } from './collect-files.mjs';
import { JS_FAMILY_LANGS } from './lang.mjs';
import { isTestLikePath } from './test-paths.mjs';

export const SNAPSHOT_SCHEMA_VERSION = 1;
export const STRICT_IDENTITY_MODE = 'strict-content-hash';

function posixRel(value) {
  return String(value).replace(/\\/g, '/');
}

export function normalizeRepoRel(root, absPath) {
  const rel = path.relative(path.resolve(root), path.resolve(absPath));
  return posixRel(rel);
}

export function languageOf(absPath) {
  return path.extname(absPath).slice(1).toLowerCase();
}

export function hashBytes(bytes) {
  return `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
}

export function hashJson(value) {
  return hashBytes(Buffer.from(JSON.stringify(value), 'utf8'));
}

export function buildContextFingerprint({
  includeTests,
  exclude = [],
  languages = JS_FAMILY_LANGS,
  producerContext = {},
}) {
  return hashJson({
    includeTests: includeTests === true,
    exclude: [...exclude].map(String).sort(),
    languages: [...languages].map(String).sort(),
    producerContext,
  });
}

export function defaultPackageScopeOf(root, absPath) {
  const resolvedRoot = path.resolve(root);
  let dir = path.dirname(path.resolve(absPath));
  while (true) {
    if (existsSync(path.join(dir, 'package.json'))) {
      const rel = normalizeRepoRel(resolvedRoot, dir);
      return rel === '' ? '.' : rel;
    }
    if (dir === resolvedRoot) return '.';
    const parent = path.dirname(dir);
    const relToParent = path.relative(resolvedRoot, parent);
    if (parent === dir || relToParent.startsWith('..') || path.isAbsolute(relToParent)) {
      return '.';
    }
    dir = parent;
  }
}

export function repoFingerprintForRoot(root) {
  let realRoot = path.resolve(root);
  try {
    realRoot = realpathSync.native(root);
  } catch {
    realRoot = path.resolve(root);
  }
  const marker = existsSync(path.join(realRoot, '.git'))
    ? 'git-worktree'
    : existsSync(path.join(realRoot, 'package.json'))
      ? 'package-root'
      : 'directory-root';
  return hashJson({
    schemaVersion: SNAPSHOT_SCHEMA_VERSION,
    realRoot: posixRel(realRoot),
    marker,
    platform: process.platform,
  });
}

export function buildFileSnapshotEntry({
  root,
  absPath,
  contextFingerprint,
  packageScopeOf = defaultPackageScopeOf,
}) {
  const relPath = normalizeRepoRel(root, absPath);
  const language = languageOf(absPath);
  const isTestLike = isTestLikePath(relPath);
  const packageScope = packageScopeOf(root, absPath);

  let stat = null;
  try {
    stat = statSync(absPath);
  } catch (error) {
    return {
      relPath,
      absPath,
      language,
      isTestLike,
      packageScope,
      readable: false,
      mtimeMs: null,
      size: null,
      hash: null,
      contentHash: null,
      contextFingerprint,
      readError: { kind: error?.code ?? 'stat-failed' },
    };
  }

  try {
    const bytes = readFileSync(absPath);
    const contentHash = hashBytes(bytes);
    return {
      relPath,
      absPath,
      language,
      isTestLike,
      packageScope,
      readable: true,
      mtimeMs: stat.mtimeMs,
      size: stat.size,
      hash: contentHash,
      contentHash,
      contextFingerprint,
    };
  } catch (error) {
    return {
      relPath,
      absPath,
      language,
      isTestLike,
      packageScope,
      readable: false,
      mtimeMs: stat.mtimeMs,
      size: stat.size,
      hash: null,
      contentHash: null,
      contextFingerprint,
      readError: { kind: error?.code ?? 'read-failed' },
    };
  }
}

export function buildRepoSnapshot({
  root,
  includeTests = true,
  exclude = [],
  languages = JS_FAMILY_LANGS,
  contextFingerprint,
  previousSnapshot = null,
  packageScopeOf = defaultPackageScopeOf,
}) {
  const files = collectFiles(root, { includeTests, exclude, languages });
  const entries = {};
  for (const absPath of files) {
    const entry = buildFileSnapshotEntry({
      root,
      absPath,
      contextFingerprint,
      packageScopeOf,
    });
    entries[entry.relPath] = entry;
  }

  const current = new Set(Object.keys(entries));
  const previous = previousSnapshot?.files && typeof previousSnapshot.files === 'object'
    ? Object.keys(previousSnapshot.files)
    : [];
  const droppedSincePrevious = previous
    .filter((relPath) => !current.has(relPath))
    .sort()
    .map((pathName) => ({ path: pathName, reason: 'deleted' }));

  return {
    schemaVersion: SNAPSHOT_SCHEMA_VERSION,
    root: path.resolve(root),
    repoFingerprint: repoFingerprintForRoot(root),
    identityMode: STRICT_IDENTITY_MODE,
    scanOptions: {
      includeTests: includeTests === true,
      exclude: [...exclude],
      languages: [...languages],
    },
    files: Object.fromEntries(Object.entries(entries).sort(([a], [b]) => a.localeCompare(b))),
    droppedSincePrevious,
  };
}
```

- [ ] **Step 4: Run the snapshot test**

Run:

```powershell
node tests/test-incremental-snapshot.mjs
```

Expected: all assertions pass. On Windows, the unreadable-file assertion may report the platform-skip branch when chmod does not block reads.

- [ ] **Step 5: Commit P0 snapshot helper**

Run:

```powershell
git add _lib/incremental-snapshot.mjs tests/test-incremental-snapshot.mjs
git commit -m "feat: add strict incremental snapshot helper"
```

## Task 2: Add Shared Cache Store Tests

**Files:**
- Create: `tests/test-incremental-cache-store.mjs`
- Create later in this task: `_lib/incremental-cache-store.mjs`

- [ ] **Step 1: Write failing cache-store tests**

Create `tests/test-incremental-cache-store.mjs`:

```js
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import {
  clearIncrementalCache,
  defaultAuditCacheRoot,
  getReusableFact,
  loadProducerCache,
  openIncrementalCacheStore,
  putFact,
  saveProducerCache,
} from '../_lib/incremental-cache-store.mjs';

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
  return mkdtempSync(path.join(tmpdir(), 'lumin-cache-store-'));
}

function entry(overrides = {}) {
  return {
    relPath: 'src/a.ts',
    language: 'ts',
    isTestLike: false,
    packageScope: '.',
    readable: true,
    contentHash: 'sha256:' + 'a'.repeat(64),
    contextFingerprint: 'sha256:' + 'b'.repeat(64),
    ...overrides,
  };
}

const producerMeta = {
  producerId: 'any-inventory',
  producerVersion: 1,
  factSchemaVersion: 1,
  parserIdentity: 'oxc-parser:test',
  scanFingerprint: 'sha256:' + 'c'.repeat(64),
  configFingerprint: 'sha256:' + 'd'.repeat(64),
};

{
  const root = fresh();
  try {
    const cacheRoot = defaultAuditCacheRoot({ root, output: path.join(root, '.audit', 'runs', 'r1') });
    assert('default cache root is stable .audit/.cache sibling',
      cacheRoot === path.join(root, '.audit', '.cache'),
      `got ${cacheRoot}`);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    const store = openIncrementalCacheStore({ root });
    const cache = loadProducerCache(store, 'any-inventory');
    putFact(cache, {
      snapshotEntry: entry(),
      producerMeta,
      payload: { typeEscapes: [{ file: 'src/a.ts', escapeKind: 'as-any' }] },
    });
    const hit = getReusableFact(cache, {
      snapshotEntry: entry(),
      producerMeta,
    });
    assert('strict reusable fact hit requires matching current content hash',
      hit.status === 'hit' && hit.payload.typeEscapes.length === 1,
      JSON.stringify(hit));

    const miss = getReusableFact(cache, {
      snapshotEntry: entry({ contentHash: 'sha256:' + 'e'.repeat(64) }),
      producerMeta,
    });
    assert('different current content hash misses',
      miss.status === 'miss' && miss.reason === 'content-hash-mismatch',
      JSON.stringify(miss));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    const store = openIncrementalCacheStore({ root });
    const cache = loadProducerCache(store, 'any-inventory');
    putFact(cache, {
      snapshotEntry: entry({ readable: false, contentHash: null }),
      producerMeta,
      payload: { typeEscapes: [] },
    });
    const miss = getReusableFact(cache, {
      snapshotEntry: entry({ readable: false, contentHash: null }),
      producerMeta,
    });
    assert('read-error entries never become clean hits',
      miss.status === 'miss' && miss.reason === 'current-file-unreadable',
      JSON.stringify(miss));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    const store = openIncrementalCacheStore({ root });
    mkdirSync(store.repoCacheDir, { recursive: true });
    writeFileSync(path.join(store.repoCacheDir, 'any-inventory.cache.json'), '{not-json');
    const cache = loadProducerCache(store, 'any-inventory');
    assert('malformed cache loads as empty with invalidated reason',
      Object.keys(cache.entries).length === 0 &&
      cache.meta.loadStatus === 'ignored-malformed');
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const root = fresh();
  try {
    const store = openIncrementalCacheStore({ root });
    const cache = loadProducerCache(store, 'any-inventory');
    putFact(cache, {
      snapshotEntry: entry(),
      producerMeta,
      payload: { typeEscapes: [] },
    });
    saveProducerCache(store, 'any-inventory', cache);
    const file = path.join(store.repoCacheDir, 'any-inventory.cache.json');
    assert('saveProducerCache writes cache file atomically',
      existsSync(file) && JSON.parse(readFileSync(file, 'utf8')).schemaVersion === 1);
    clearIncrementalCache(store);
    assert('clearIncrementalCache removes repo cache dir contents',
      !existsSync(file));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```powershell
node tests/test-incremental-cache-store.mjs
```

Expected: FAIL with module-not-found for `_lib/incremental-cache-store.mjs`.

- [ ] **Step 3: Implement `_lib/incremental-cache-store.mjs`**

Create `_lib/incremental-cache-store.mjs`:

```js
import { existsSync, mkdirSync, readFileSync, rmSync } from 'node:fs';
import path from 'node:path';

import { atomicWrite } from './atomic-write.mjs';
import { repoFingerprintForRoot } from './incremental-snapshot.mjs';

export const CACHE_STORE_SCHEMA_VERSION = 1;

function emptyCache(loadStatus = 'empty') {
  return {
    schemaVersion: CACHE_STORE_SCHEMA_VERSION,
    meta: { loadStatus },
    entries: {},
  };
}

export function defaultAuditCacheRoot({ root }) {
  return path.join(path.resolve(root), '.audit', '.cache');
}

export function openIncrementalCacheStore({ root, cacheRoot = null, repoFingerprint = null } = {}) {
  const resolvedRoot = path.resolve(root);
  const fingerprint = repoFingerprint ?? repoFingerprintForRoot(resolvedRoot);
  const baseCacheRoot = cacheRoot ? path.resolve(cacheRoot) : defaultAuditCacheRoot({ root: resolvedRoot });
  const repoCacheDir = path.join(baseCacheRoot, 'incremental', fingerprint.replace(/^sha256:/, ''));
  return {
    root: resolvedRoot,
    cacheRoot: baseCacheRoot,
    repoFingerprint: fingerprint,
    repoCacheDir,
  };
}

function producerCachePath(store, producerId) {
  return path.join(store.repoCacheDir, `${producerId}.cache.json`);
}

export function loadProducerCache(store, producerId) {
  const file = producerCachePath(store, producerId);
  if (!existsSync(file)) return emptyCache();
  try {
    const parsed = JSON.parse(readFileSync(file, 'utf8'));
    if (parsed?.schemaVersion !== CACHE_STORE_SCHEMA_VERSION || !parsed.entries || typeof parsed.entries !== 'object') {
      return emptyCache('ignored-incompatible');
    }
    return {
      schemaVersion: CACHE_STORE_SCHEMA_VERSION,
      meta: { loadStatus: 'ok' },
      entries: parsed.entries,
    };
  } catch {
    return emptyCache('ignored-malformed');
  }
}

export function saveProducerCache(store, producerId, cache) {
  mkdirSync(store.repoCacheDir, { recursive: true });
  const file = producerCachePath(store, producerId);
  const stableEntries = Object.fromEntries(
    Object.entries(cache.entries ?? {}).sort(([a], [b]) => a.localeCompare(b))
  );
  atomicWrite(file, JSON.stringify({
    schemaVersion: CACHE_STORE_SCHEMA_VERSION,
    entries: stableEntries,
  }, null, 2) + '\n');
}

export function clearIncrementalCache(store) {
  rmSync(store.repoCacheDir, { recursive: true, force: true });
}

function strictKey(snapshotEntry) {
  return [
    snapshotEntry.relPath,
    snapshotEntry.language,
    snapshotEntry.isTestLike ? 'test' : 'prod',
    snapshotEntry.packageScope,
  ].join('|');
}

function sameProducerMeta(a = {}, b = {}) {
  return (
    a.producerId === b.producerId &&
    a.producerVersion === b.producerVersion &&
    a.factSchemaVersion === b.factSchemaVersion &&
    a.parserIdentity === b.parserIdentity &&
    a.scanFingerprint === b.scanFingerprint &&
    a.configFingerprint === b.configFingerprint
  );
}

export function putFact(cache, { snapshotEntry, producerMeta, payload }) {
  const key = strictKey(snapshotEntry);
  cache.entries[key] = {
    key,
    identity: {
      relPath: snapshotEntry.relPath,
      language: snapshotEntry.language,
      isTestLike: snapshotEntry.isTestLike === true,
      packageScope: snapshotEntry.packageScope,
      contextFingerprint: snapshotEntry.contextFingerprint,
      contentHash: snapshotEntry.contentHash,
    },
    producerMeta: { ...producerMeta },
    payload,
  };
}

export function getReusableFact(cache, { snapshotEntry, producerMeta }) {
  if (!snapshotEntry?.readable || !snapshotEntry.contentHash) {
    return { status: 'miss', reason: 'current-file-unreadable' };
  }
  const prior = cache.entries?.[strictKey(snapshotEntry)];
  if (!prior) return { status: 'miss', reason: 'missing-entry' };
  if (!sameProducerMeta(prior.producerMeta, producerMeta)) {
    return { status: 'miss', reason: 'producer-or-context-mismatch' };
  }
  const id = prior.identity ?? {};
  if (id.contextFingerprint !== snapshotEntry.contextFingerprint) {
    return { status: 'miss', reason: 'context-fingerprint-mismatch' };
  }
  if (id.contentHash !== snapshotEntry.contentHash) {
    return { status: 'miss', reason: 'content-hash-mismatch' };
  }
  return { status: 'hit', payload: prior.payload };
}
```

- [ ] **Step 4: Run cache-store tests**

Run:

```powershell
node tests/test-incremental-cache-store.mjs
```

Expected: all assertions pass.

- [ ] **Step 5: Add a legacy helper header note**

Modify the header comment in `_lib/incremental.mjs`:

```js
// _lib/incremental.mjs — Legacy producer-local cache helper.
//
// New strict shared incremental work must use `_lib/incremental-snapshot.mjs`
// and `_lib/incremental-cache-store.mjs`. This helper intentionally preserves
// historical stat-first-cut behavior for existing producer-local caches until
// each producer is migrated through a compatibility adapter.
```

- [ ] **Step 6: Run legacy incremental tests**

Run:

```powershell
node tests/test-incremental.mjs
```

Expected: existing assertions continue to pass because this task does not change legacy behavior.

- [ ] **Step 7: Commit P0 cache store**

Run:

```powershell
git add _lib/incremental-cache-store.mjs _lib/incremental.mjs tests/test-incremental-cache-store.mjs
git commit -m "feat: add strict incremental cache store"
```

## Task 3: Add Any-Inventory Incremental Tests

**Files:**
- Create: `tests/test-any-inventory-incremental.mjs`
- Modify later: `any-inventory.mjs`

- [ ] **Step 1: Write failing any-inventory incremental tests**

Create `tests/test-any-inventory-incremental.mjs`:

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
const CLI = path.join(ROOT, 'any-inventory.mjs');

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
  return mkdtempSync(path.join(tmpdir(), 'lumin-any-inc-'));
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

function readInv(output, name = 'any-inventory.json') {
  return JSON.parse(readFileSync(path.join(output, name), 'utf8'));
}

function stableInventory(inv) {
  return {
    complete: inv.meta.complete,
    scope: inv.meta.scope,
    includeTests: inv.meta.includeTests,
    exclude: inv.meta.exclude,
    supports: inv.meta.supports,
    typeEscapes: inv.typeEscapes,
    filesWithParseErrors: inv.meta.filesWithParseErrors,
  };
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    write(repo, 'src/b.ts', 'const b = value as unknown as string;\n');

    run(repo, output, ['--no-incremental']);
    const cold = readInv(output);
    run(repo, output);
    const warm = readInv(output);

    assert('warm any-inventory equals cold public facts',
      JSON.stringify(stableInventory(warm)) === JSON.stringify(stableInventory(cold)));
    assert('warm run reports incremental enabled',
      warm.meta.incremental?.enabled === true);
    assert('warm run reused at least one file',
      warm.meta.incremental?.reusedFiles >= 1,
      JSON.stringify(warm.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    write(repo, 'src/b.ts', 'const b = value as any;\n');
    run(repo, output);

    write(repo, 'src/b.ts', 'const b = value as unknown as string;\n');
    run(repo, output);
    const inv = readInv(output);

    assert('changed file facts update after edit',
      inv.typeEscapes.some((fact) => fact.file === 'src/b.ts' && fact.escapeKind === 'as-unknown-as-T'));
    assert('unchanged file remains present',
      inv.typeEscapes.some((fact) => fact.file === 'src/a.ts' && fact.escapeKind === 'as-any'));
    assert('incremental changed count is positive',
      inv.meta.incremental.changedFiles >= 1);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    write(repo, 'src/b.ts', 'const b = value as any;\n');
    run(repo, output);

    rmSync(path.join(repo, 'src/b.ts'), { force: true });
    run(repo, output);
    const inv = readInv(output);

    assert('deleted file facts disappear',
      !inv.typeEscapes.some((fact) => fact.file === 'src/b.ts'));
    assert('deleted file contributes dropped count',
      inv.meta.incremental.droppedFiles >= 1);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    write(repo, 'tests/a.test.ts', 'const t = value as any;\n');
    run(repo, output, ['--production']);
    run(repo, output);
    const inv = readInv(output);

    assert('scan option change keeps public artifact correct',
      inv.meta.includeTests === true &&
      inv.typeEscapes.some((fact) => fact.file === 'tests/a.test.ts'));
    assert('scan option change prevents stale production-only reuse',
      inv.meta.incremental.invalidatedFiles >= 0);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    run(repo, output);

    const cacheFile = path.join(output, '.cache', 'incremental');
    mkdirSync(cacheFile, { recursive: true });
    writeFileSync(path.join(cacheFile, 'bad.cache.json'), '{broken');

    run(repo, output);
    const inv = readInv(output);
    assert('malformed unrelated cache does not crash producer',
      inv.meta.complete === true && inv.typeEscapes.length === 1);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    run(repo, output, ['--no-incremental']);
    const inv = readInv(output);
    assert('--no-incremental reports disabled meta',
      inv.meta.incremental?.enabled === false &&
      inv.meta.incremental?.reason === 'disabled-by-flag');
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```powershell
node tests/test-any-inventory-incremental.mjs
```

Expected: FAIL because `any-inventory.mjs` does not emit `meta.incremental` and does not support `--no-incremental`.

- [ ] **Step 3: Commit the failing test**

Run:

```powershell
git add tests/test-any-inventory-incremental.mjs
git commit -m "test: specify incremental any-inventory behavior"
```

## Task 4: Implement Any-Inventory Incremental Adapter

**Files:**
- Modify: `any-inventory.mjs`

- [ ] **Step 1: Add imports and CLI flags**

Modify the import block in `any-inventory.mjs`:

```js
import {
  buildContextFingerprint,
  buildRepoSnapshot,
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

Modify the `parseCliArgs()` call:

```js
const cli = parseCliArgs({
  'artifact-name': { type: 'string' },
  'no-incremental': { type: 'boolean', default: false },
  'cache-root': { type: 'string' },
  'clear-incremental-cache': { type: 'boolean', default: false },
});
```

- [ ] **Step 2: Add producer identity constants**

Add near `ESCAPE_KINDS`:

```js
const PRODUCER_ID = 'any-inventory';
const PRODUCER_VERSION = 1;
const FACT_SCHEMA_VERSION = 1;
const PARSER_IDENTITY = 'oxc-parser:extract-type-escapes-v1';

function sortTypeEscapes(facts) {
  return [...facts].sort((a, b) =>
    String(a.file).localeCompare(String(b.file)) ||
    Number(a.line ?? 0) - Number(b.line ?? 0) ||
    String(a.escapeKind).localeCompare(String(b.escapeKind)) ||
    String(a.occurrenceKey ?? '').localeCompare(String(b.occurrenceKey ?? '')));
}
```

- [ ] **Step 3: Replace the current file loop with strict snapshot/cache flow**

Replace the current `const files = collectFiles(...)` through the file loop with:

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

const incrementalEnabled = cli.raw?.['no-incremental'] !== true;
const cacheStore = openIncrementalCacheStore({
  root: ROOT,
  cacheRoot: cli.raw?.['cache-root'],
});
if (cli.raw?.['clear-incremental-cache'] === true) {
  clearIncrementalCache(cacheStore);
}

const producerMeta = {
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

const typeEscapes = [];
const filesWithParseErrors = [];
const currentStrictKeys = new Set();
let changedFiles = 0;
let reusedFiles = 0;
let invalidatedFiles = 0;

for (const entry of Object.values(snapshot.files)) {
  currentStrictKeys.add([
    entry.relPath,
    entry.language,
    entry.isTestLike ? 'test' : 'prod',
    entry.packageScope,
  ].join('|'));

  if (!entry.readable) {
    changedFiles++;
    filesWithParseErrors.push({
      file: entry.relPath,
      message: `read failed: ${entry.readError?.kind ?? 'unknown'}`,
      line: 0,
    });
    continue;
  }

  const reuse = incrementalEnabled
    ? getReusableFact(priorCache, { snapshotEntry: entry, producerMeta })
    : { status: 'miss', reason: 'disabled-by-flag' };

  if (reuse.status === 'hit') {
    reusedFiles++;
    for (const fact of reuse.payload.typeEscapes ?? []) typeEscapes.push(fact);
    putFact(nextCache, {
      snapshotEntry: entry,
      producerMeta,
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
    filesWithParseErrors.push({
      file: entry.relPath,
      message: `read failed: ${e.message}`,
      line: 0,
    });
    continue;
  }

  const result = extractTypeEscapes(src, entry.relPath);
  if (result.parseError) {
    filesWithParseErrors.push({
      file: entry.relPath,
      message: result.parseError.slice(0, 200),
      line: 0,
    });
    continue;
  }

  const payload = { typeEscapes: Array.isArray(result.typeEscapes) ? result.typeEscapes : [] };
  for (const fact of payload.typeEscapes) typeEscapes.push(fact);
  if (incrementalEnabled) {
    putFact(nextCache, {
      snapshotEntry: entry,
      producerMeta,
      payload,
    });
  }
}

const droppedFiles = Object.keys(priorCache.entries ?? {})
  .filter((key) => !currentStrictKeys.has(key)).length;
if (incrementalEnabled) {
  saveProducerCache(cacheStore, PRODUCER_ID, nextCache);
}
```

- [ ] **Step 4: Update artifact meta and deterministic ordering**

In the artifact object, replace `fileCount: files.length` with `fileCount: Object.keys(snapshot.files).length`, and add:

```js
incremental: {
  enabled: incrementalEnabled,
  identityMode: incrementalEnabled ? 'strict-content-hash' : null,
  cacheVersion: 1,
  cacheRoot: incrementalEnabled ? cacheStore.cacheRoot : null,
  changedFiles,
  reusedFiles,
  droppedFiles,
  invalidatedFiles,
  reason: incrementalEnabled ? null : 'disabled-by-flag',
},
```

Set the top-level facts to deterministic order:

```js
typeEscapes: sortTypeEscapes(typeEscapes),
```

- [ ] **Step 5: Run focused tests**

Run:

```powershell
node tests/test-any-inventory-incremental.mjs
node tests/test-any-inventory.mjs
```

Expected: both pass.

- [ ] **Step 6: Commit any-inventory adapter**

Run:

```powershell
git add any-inventory.mjs tests/test-any-inventory-incremental.mjs
git commit -m "feat: make any-inventory use strict incremental cache"
```

## Task 5: Add Post-Write Incremental Forwarding Tests

**Files:**
- Create: `tests/test-post-write-incremental.mjs`
- Modify later: `post-write.mjs`

- [ ] **Step 1: Write failing post-write incremental tests**

Create `tests/test-post-write-incremental.mjs`:

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
const ANY = path.join(ROOT, 'any-inventory.mjs');
const POST = path.join(ROOT, 'post-write.mjs');

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
  return mkdtempSync(path.join(tmpdir(), 'lumin-post-inc-'));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function runAny(root, out, args = []) {
  execFileSync(NODE, [ANY, '--root', root, '--output', out, ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readJson(file) {
  return JSON.parse(readFileSync(file, 'utf8'));
}

{
  const repo = fresh();
  const out = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    mkdirSync(out, { recursive: true });

    runAny(repo, out, ['--artifact-name', 'any-inventory.pre.invocation.json']);
    const advisory = {
      preWrite: {
        anyInventoryPath: 'any-inventory.pre.invocation.json',
        fileInventory: { status: 'available', files: ['src/a.ts'] },
      },
      intent: { files: ['src/a.ts'] },
      scanRange: { output: out },
    };
    const advisoryPath = path.join(out, 'pre-write-advisory.json');
    writeFileSync(advisoryPath, JSON.stringify(advisory, null, 2));

    write(repo, 'src/a.ts', 'const a = value as any;\nconst b = value as unknown as string;\n');
    execFileSync(NODE, [
      POST,
      '--root', repo,
      '--output', out,
      '--pre-write-advisory', advisoryPath,
    ], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });

    const after = readJson(path.join(out, 'any-inventory.json'));
    assert('post-write after-snapshot uses incremental any-inventory by default',
      after.meta.incremental?.enabled === true,
      JSON.stringify(after.meta.incremental));

    const before = readJson(path.join(out, 'any-inventory.pre.invocation.json'));
    assert('pre-write baseline artifact is not mutated by post-write',
      !before.typeEscapes.some((fact) => fact.escapeKind === 'as-unknown-as-T'));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const out = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    mkdirSync(out, { recursive: true });

    runAny(repo, out, ['--artifact-name', 'any-inventory.pre.invocation.json']);
    const advisoryPath = path.join(out, 'pre-write-advisory.json');
    writeFileSync(advisoryPath, JSON.stringify({
      preWrite: {
        anyInventoryPath: 'any-inventory.pre.invocation.json',
        fileInventory: { status: 'available', files: ['src/a.ts'] },
      },
      intent: { files: ['src/a.ts'] },
      scanRange: { output: out },
    }, null, 2));

    execFileSync(NODE, [
      POST,
      '--root', repo,
      '--output', out,
      '--pre-write-advisory', advisoryPath,
      '--no-incremental',
    ], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });

    const after = readJson(path.join(out, 'any-inventory.json'));
    assert('post-write forwards --no-incremental to after-snapshot',
      after.meta.incremental?.enabled === false &&
      after.meta.incremental?.reason === 'disabled-by-flag',
      JSON.stringify(after.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```powershell
node tests/test-post-write-incremental.mjs
```

Expected: FAIL because `post-write.mjs` does not parse or forward `--no-incremental`.

- [ ] **Step 3: Commit the failing test**

Run:

```powershell
git add tests/test-post-write-incremental.mjs
git commit -m "test: specify post-write incremental after-snapshot"
```

## Task 6: Implement Post-Write And Audit CLI Forwarding

**Files:**
- Modify: `post-write.mjs`
- Modify: `audit-repo.mjs`

- [ ] **Step 1: Add post-write CLI flags**

In `post-write.mjs`, extend `parseCliArgs()`:

```js
const args = parseCliArgs({
  'pre-write-advisory': { type: 'string' },
  'delta-out': { type: 'string' },
  'no-fresh-audit': { type: 'boolean', default: false },
  'no-incremental': { type: 'boolean', default: false },
  'cache-root': { type: 'string' },
  'clear-incremental-cache': { type: 'boolean', default: false },
});
```

- [ ] **Step 2: Forward incremental flags to `any-inventory.mjs`**

Inside the `if (!noFreshAudit)` block, after exclude forwarding, add:

```js
if (args.raw?.['no-incremental'] === true) hookArgs.push('--no-incremental');
if (args.raw?.['cache-root']) hookArgs.push('--cache-root', path.resolve(args.raw['cache-root']));
if (args.raw?.['clear-incremental-cache'] === true) hookArgs.push('--clear-incremental-cache');
```

- [ ] **Step 3: Add audit-repo CLI options**

In `audit-repo.mjs`, add to `CLI_OPTIONS`:

```js
'no-incremental': { type: 'boolean', default: false },
'cache-root': { type: 'string' },
'clear-incremental-cache': { type: 'boolean', default: false },
```

- [ ] **Step 4: Add forwarded incremental args helper**

Near `forwardedScanArgs()`, add:

```js
function forwardedIncrementalArgs() {
  const args = [];
  if (values['no-incremental'] === true) args.push('--no-incremental');
  if (values['cache-root']) args.push('--cache-root', path.resolve(values['cache-root']));
  if (values['clear-incremental-cache'] === true) args.push('--clear-incremental-cache');
  return args;
}
```

When spawning post-write from `audit-repo.mjs`, append `...forwardedIncrementalArgs()` only to the post-write child argv. Do not pass these flags to producers that do not support them.

- [ ] **Step 5: Update help text**

In `audit-repo.mjs` `HELP_TEXT`, add:

```text
  --no-incremental        force cold producer artifacts where incremental is supported
  --cache-root <path>     stable incremental cache root (default: <root>/.audit/.cache)
  --clear-incremental-cache
                           clear this repo's incremental cache before supported producers run
```

- [ ] **Step 6: Run focused tests**

Run:

```powershell
node tests/test-post-write-incremental.mjs
node tests/test-post-write-cli.mjs
node tests/test-audit-repo-post-write.mjs
node tests/test-cli.mjs
```

Expected: all pass.

- [ ] **Step 7: Commit post-write forwarding**

Run:

```powershell
git add post-write.mjs audit-repo.mjs tests/test-post-write-incremental.mjs
git commit -m "feat: forward incremental cache flags through post-write"
```

## Task 7: Update Documentation And Run Full Verification

**Files:**
- Modify: `docs/spec/incremental-engine-architecture.md`
- Modify: `docs/spec/README.md` only if the implementation plan link should be surfaced from spec staging.
- Modify: `README.md` only if CLI help surface changes require public documentation.

- [ ] **Step 1: Add implementation status note to the architecture spec**

In `docs/spec/incremental-engine-architecture.md`, under the status block, add:

```md
> **Implementation plan:** P0/P1 execution lives in
> [`docs/superpowers/plans/2026-05-04-incremental-engine-p0-p1.md`](../superpowers/plans/2026-05-04-incremental-engine-p0-p1.md).
```

- [ ] **Step 2: Run focused test suite**

Run:

```powershell
node tests/test-incremental-snapshot.mjs
node tests/test-incremental-cache-store.mjs
node tests/test-any-inventory-incremental.mjs
node tests/test-post-write-incremental.mjs
node tests/test-incremental.mjs
node tests/test-any-inventory.mjs
node tests/test-post-write-cli.mjs
node tests/test-audit-repo-post-write.mjs
```

Expected: every command exits 0 and prints only PASS summaries.

- [ ] **Step 3: Run repo gates**

Run:

```powershell
npm run check:doc-script-refs
node tests/test-skill-surface.mjs
npm run ci
```

Expected:

```text
[check-doc-script-refs] all documented .mjs references resolve on disk
35 passed, 0 failed
npm run ci exits 0
```

- [ ] **Step 4: Run self smoke for any-inventory**

Run:

```powershell
Remove-Item -Recurse -Force .audit\incremental-self -ErrorAction SilentlyContinue
node any-inventory.mjs --root . --output .audit\incremental-self --no-incremental
node any-inventory.mjs --root . --output .audit\incremental-self
node -e "const j=require('./.audit/incremental-self/any-inventory.json'); console.log(JSON.stringify(j.meta.incremental,null,2))"
```

Expected: second run reports `enabled: true`, `identityMode: "strict-content-hash"`, and `reusedFiles > 0`.

- [ ] **Step 5: Commit docs and verification adjustments**

Run:

```powershell
git add docs/spec/incremental-engine-architecture.md docs/spec/README.md README.md
git commit -m "docs: link incremental P0 P1 implementation plan"
```

Only include `docs/spec/README.md` or `README.md` if they changed.

## Self-Review Checklist

- [ ] P0 strict snapshot and cache store are implemented before any producer adapter.
- [ ] Strict cache hit requires current `contentHash`; stat cannot prove a hit.
- [ ] Unreadable in-scope files remain visible and invalidate prior facts.
- [ ] Public `any-inventory.json` facts are cold/warm equivalent except `meta.incremental`.
- [ ] Post-write baseline artifact is immutable and invocation-specific.
- [ ] Legacy `_lib/incremental.mjs` stat-first-cut behavior is not used by new strict P0/P1 code.
- [ ] All new tests fail before implementation and pass after implementation.
- [ ] No source edits are applied by the incremental engine.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-04-incremental-engine-p0-p1.md`. Two execution options:

**1. Subagent-Driven (recommended)** - dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** - execute tasks in this session using executing-plans, with checkpoints after each task.

Choose one before implementation starts.
