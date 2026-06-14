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
