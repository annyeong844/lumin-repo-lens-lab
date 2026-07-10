// Shared strict incremental cache store.
//
// Cache files are untrusted source-derived artifacts. A cache entry is a
// strict hit only when the current snapshot entry and producer context match.

import { createHash } from 'node:crypto';
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readdirSync,
  readFileSync,
  readSync,
  rmSync,
  statSync,
} from 'node:fs';
import path from 'node:path';

import { atomicCopy, atomicWrite } from './atomic-write.mjs';
import { repoFingerprintForRoot } from './incremental-snapshot.mjs';

export const CACHE_STORE_SCHEMA_VERSION = 1;
export const ARTIFACT_CACHE_SCHEMA_VERSION = 1;
const FILE_HASH_BUFFER_BYTES = 1024 * 1024;

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

function artifactCacheKey(requestIdentity) {
  const match = /^sha256:([a-f0-9]{64})$/.exec(requestIdentity);
  if (!match) throw new Error('artifact cache request identity must be sha256');
  return match[1];
}

function producerArtifactCachePaths(store, producerId, requestIdentity) {
  const dir = path.join(store.repoCacheDir, `${producerId}.artifact-cache`);
  const key = artifactCacheKey(requestIdentity);
  return {
    dir,
    key,
    artifact: path.join(dir, `${key}.artifact.json`),
    manifest: path.join(dir, `${key}.manifest.json`),
  };
}

function cleanupOldProducerArtifactCacheEntries(paths) {
  let failures = 0;
  let names;
  try {
    names = readdirSync(paths.dir);
  } catch {
    return 1;
  }
  const keep = new Set([
    `${paths.key}.artifact.json`,
    `${paths.key}.manifest.json`,
  ]);
  for (const name of names) {
    if (keep.has(name)) continue;
    if (
      name !== 'artifact.json' &&
      name !== 'manifest.json' &&
      !/^[a-f0-9]{64}\.(?:artifact|manifest)\.json$/.test(name)
    ) {
      continue;
    }
    try {
      rmSync(path.join(paths.dir, name), { force: true });
    } catch {
      failures++;
    }
  }
  return failures;
}

function hashFile(pathName) {
  const hash = createHash('sha256');
  const buffer = Buffer.allocUnsafe(FILE_HASH_BUFFER_BYTES);
  const fd = openSync(pathName, 'r');
  try {
    for (;;) {
      const bytesRead = readSync(fd, buffer, 0, buffer.length, null);
      if (bytesRead === 0) break;
      hash.update(buffer.subarray(0, bytesRead));
    }
  } finally {
    closeSync(fd);
  }
  return `sha256:${hash.digest('hex')}`;
}

export function loadProducerCache(store, producerId) {
  const file = producerCachePath(store, producerId);
  if (!existsSync(file)) return emptyCache();

  try {
    const parsed = JSON.parse(readFileSync(file, 'utf8'));
    if (
      parsed?.schemaVersion !== CACHE_STORE_SCHEMA_VERSION ||
      !parsed.entries ||
      typeof parsed.entries !== 'object'
    ) {
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

export function loadProducerArtifactCache(store, producerId, requestIdentity) {
  const paths = producerArtifactCachePaths(store, producerId, requestIdentity);
  if (!existsSync(paths.manifest)) {
    return { status: 'miss', reason: 'missing-manifest' };
  }

  let manifest;
  try {
    manifest = JSON.parse(readFileSync(paths.manifest, 'utf8'));
  } catch {
    return { status: 'miss', reason: 'malformed-manifest' };
  }
  if (
    manifest?.schemaVersion !== ARTIFACT_CACHE_SCHEMA_VERSION ||
    manifest?.producerId !== producerId ||
    typeof manifest?.requestIdentity !== 'string' ||
    typeof manifest?.artifactSha256 !== 'string' ||
    !Number.isSafeInteger(manifest?.artifactBytes) ||
    manifest.artifactBytes < 0
  ) {
    return { status: 'miss', reason: 'incompatible-manifest' };
  }
  if (manifest.requestIdentity !== requestIdentity) {
    return { status: 'miss', reason: 'identity-mismatch' };
  }
  if (!existsSync(paths.artifact)) {
    return { status: 'miss', reason: 'missing-artifact' };
  }

  try {
    const artifactBytes = statSync(paths.artifact).size;
    if (artifactBytes !== manifest.artifactBytes) {
      return { status: 'miss', reason: 'size-mismatch' };
    }
    if (hashFile(paths.artifact) !== manifest.artifactSha256) {
      return { status: 'miss', reason: 'hash-mismatch' };
    }
  } catch {
    return { status: 'miss', reason: 'artifact-read-failed' };
  }

  return {
    status: 'hit',
    artifactPath: paths.artifact,
    artifactBytes: manifest.artifactBytes,
    artifactSha256: manifest.artifactSha256,
  };
}

export function saveProducerArtifactCache(
  store,
  producerId,
  { requestIdentity, artifactPath },
) {
  const paths = producerArtifactCachePaths(store, producerId, requestIdentity);
  mkdirSync(paths.dir, { recursive: true });
  atomicCopy(artifactPath, paths.artifact);
  const artifactBytes = statSync(paths.artifact).size;
  const artifactSha256 = hashFile(paths.artifact);
  atomicWrite(paths.manifest, JSON.stringify({
    schemaVersion: ARTIFACT_CACHE_SCHEMA_VERSION,
    producerId,
    requestIdentity,
    artifactBytes,
    artifactSha256,
  }, null, 2) + '\n');
  const cleanupFailures = cleanupOldProducerArtifactCacheEntries(paths);
  return { artifactBytes, artifactSha256, cleanupFailures };
}

export function restoreProducerArtifactCache(cacheHit, targetPath) {
  if (cacheHit?.status !== 'hit' || !cacheHit.artifactPath) {
    throw new Error('artifact cache restore requires a verified cache hit');
  }
  atomicCopy(cacheHit.artifactPath, targetPath);
}

export function clearIncrementalCache(store) {
  rmSync(store.repoCacheDir, { recursive: true, force: true });
}

export function strictCacheKeyForEntry(snapshotEntry) {
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
    a.configFingerprint === b.configFingerprint &&
    a.sourceSetFingerprint === b.sourceSetFingerprint
  );
}

export function putFact(cache, { snapshotEntry, producerMeta, payload }) {
  const key = strictCacheKeyForEntry(snapshotEntry);
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

  const prior = cache.entries?.[strictCacheKeyForEntry(snapshotEntry)];
  if (!prior) return { status: 'miss', reason: 'missing-entry' };

  if (!sameProducerMeta(prior.producerMeta, producerMeta)) {
    return { status: 'miss', reason: 'producer-or-context-mismatch' };
  }

  const identity = prior.identity ?? {};
  if (identity.contextFingerprint !== snapshotEntry.contextFingerprint) {
    return { status: 'miss', reason: 'context-fingerprint-mismatch' };
  }
  if (identity.contentHash !== snapshotEntry.contentHash) {
    return { status: 'miss', reason: 'content-hash-mismatch' };
  }

  return { status: 'hit', payload: prior.payload };
}
