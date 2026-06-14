// Tests for _lib/incremental.mjs — file-hash + stat-first-cut cache.
//
// Covers:
//   - Baseline behavior (no cache → all changed; cache hit → all unchanged).
//   - Dropped files detection.
//   - Cache version bump invalidates stale entries.
//   - **Stat-first-cut (E-6)**: when prior.mtimeMs + prior.size match the
//     current file's stat, the hash is NOT recomputed — structurally
//     verified by planting a fake hash in the cache and confirming it
//     survives (no re-hashing overwrites it).

import { writeFileSync, utimesSync, mkdtempSync, rmSync, statSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import {
  loadCache, saveCache, pickChangedFiles, cacheBanner,
} from '../_lib/incremental.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function fresh() {
  return mkdtempSync(path.join(tmpdir(), 'inc-'));
}

// ═══ T1. First run, no cache → all changed ═══

{
  const dir = fresh();
  try {
    const a = path.join(dir, 'a.ts');
    const b = path.join(dir, 'b.ts');
    writeFileSync(a, 'export const x = 1;\n');
    writeFileSync(b, 'export const y = 2;\n');
    const cache = loadCache(dir, 'demo');
    const { changed, unchanged, dropped, nextCache } = pickChangedFiles([a, b], cache);
    assert('T1. first run: both files in changed',
      changed.length === 2 && unchanged.length === 0);
    assert('T1b. dropped empty on first run',
      dropped.length === 0);
    assert('T1c. nextCache entries have hash + mtimeMs + size',
      Object.values(nextCache.entries).every((e) =>
        typeof e.hash === 'string' && typeof e.mtimeMs === 'number' && typeof e.size === 'number'));
    saveCache(dir, 'demo', nextCache);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ T2. Second run, no changes → all unchanged ═══

{
  const dir = fresh();
  try {
    const a = path.join(dir, 'a.ts');
    writeFileSync(a, 'export const x = 1;\n');
    const next = pickChangedFiles([a], loadCache(dir, 'demo')).nextCache;
    saveCache(dir, 'demo', next);
    const second = pickChangedFiles([a], loadCache(dir, 'demo'));
    assert('T2. second run with no change: file in unchanged',
      second.unchanged.length === 1 && second.changed.length === 0);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ T3. Stat-first-cut (E-6): matching mtime+size skips hash ═══
//
// Plant a FAKE hash in the cache. If the stat-first-cut fires, the fake
// hash is preserved (no re-hashing overwrites it). If the optimization
// is NOT in place, the real hash replaces the fake one.

{
  const dir = fresh();
  try {
    const a = path.join(dir, 'a.ts');
    writeFileSync(a, 'export const x = 1;\n');
    const st = statSync(a);
    const FAKE_HASH = 'fake-stat-cut-proof';
    const plantedCache = {
      version: loadCache(dir, 'demo').version,
      entries: {
        [a]: { hash: FAKE_HASH, mtimeMs: st.mtimeMs, size: st.size },
      },
    };
    const { changed, unchanged, nextCache } = pickChangedFiles([a], plantedCache);
    assert('T3a. stat match → file classified as unchanged (no hash computed)',
      unchanged.length === 1 && changed.length === 0);
    assert('T3b. stat-first-cut preserved the planted fake hash (proves hash skipped)',
      nextCache.entries[a].hash === FAKE_HASH);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ T4. Stat changed but content identical → hash falls back, still unchanged ═══
//
// Touch the file (bump mtime) without changing bytes. Stat-first-cut
// MISSES (mtime differs), so hash is recomputed. Since content is the
// same, the hash matches prior → file classifies as unchanged and the
// real hash is written (overwriting any prior value if it was fake).

{
  const dir = fresh();
  try {
    const a = path.join(dir, 'a.ts');
    writeFileSync(a, 'export const x = 1;\n');
    // Populate cache with real hash.
    const next = pickChangedFiles([a], loadCache(dir, 'demo')).nextCache;
    const realHash = next.entries[a].hash;
    saveCache(dir, 'demo', next);

    // Touch — bump mtime 1 second into the future.
    const past = statSync(a).mtime;
    const future = new Date(past.getTime() + 2000);
    utimesSync(a, future, future);

    const second = pickChangedFiles([a], loadCache(dir, 'demo'));
    assert('T4a. mtime changed but content same → still unchanged (hash re-matched)',
      second.unchanged.length === 1 && second.changed.length === 0);
    assert('T4b. nextCache.mtimeMs updated to the new mtime',
      second.nextCache.entries[a].mtimeMs !== next.entries[a].mtimeMs);
    assert('T4c. hash preserved identical to prior',
      second.nextCache.entries[a].hash === realHash);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ T5. Content change → classified as changed ═══

{
  const dir = fresh();
  try {
    const a = path.join(dir, 'a.ts');
    writeFileSync(a, 'export const x = 1;\n');
    const next = pickChangedFiles([a], loadCache(dir, 'demo')).nextCache;
    saveCache(dir, 'demo', next);

    writeFileSync(a, 'export const x = 99;\n');
    const second = pickChangedFiles([a], loadCache(dir, 'demo'));
    assert('T5. content edit → changed',
      second.changed.length === 1 && second.unchanged.length === 0);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ T6. Dropped files detected ═══

{
  const dir = fresh();
  try {
    const a = path.join(dir, 'a.ts');
    const b = path.join(dir, 'b.ts');
    writeFileSync(a, 'export const x = 1;\n');
    writeFileSync(b, 'export const y = 2;\n');
    saveCache(dir, 'demo', pickChangedFiles([a, b], loadCache(dir, 'demo')).nextCache);

    const second = pickChangedFiles([a], loadCache(dir, 'demo'));
    assert('T6. file no longer in list appears in dropped',
      second.dropped.length === 1 && second.dropped[0] === b);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ T7. Stale cache version → invalidated ═══

{
  const dir = fresh();
  try {
    const a = path.join(dir, 'a.ts');
    writeFileSync(a, 'export const x = 1;\n');
    // Write a cache with obsolete version.
    saveCache(dir, 'demo', { version: 0, entries: { [a]: { hash: 'old', mtimeMs: 1, size: 1 } } });
    const loaded = loadCache(dir, 'demo');
    assert('T7. obsolete version resets cache to empty',
      Object.keys(loaded.entries).length === 0);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ═══ T8. cacheBanner string format ═══

{
  const s = cacheBanner('topology', [1, 2], [3, 4, 5, 6, 7, 8], [9]);
  assert('T8. cacheBanner mentions the name + counts + percentage',
    s.includes('topology') && s.includes('2 changed') && s.includes('6 cached') && /75%/.test(s) && s.includes('1 dropped'));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
