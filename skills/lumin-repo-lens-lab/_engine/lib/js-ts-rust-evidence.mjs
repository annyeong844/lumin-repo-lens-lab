import path from 'node:path';

import {
  runAuditCoreJsonResultFile,
  runWindowsHostAuditCoreJsonResultFile,
  windowsHostPathToWsl,
  wslPathToWindowsHost,
} from './audit-core.mjs';
import { packageRoot } from './pre-write-lookup-dep.mjs';

const REQUEST_SCHEMA_VERSION = 'lumin-js-ts-pre-write-evidence-request.v1';
const RESPONSE_SCHEMA_VERSION = 'lumin-js-ts-pre-write-evidence-response.v1';

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function validateResponse(response, expectedArtifact, expectedInventory) {
  if (response?.schemaVersion !== RESPONSE_SCHEMA_VERSION) {
    throw new Error('Rust JS/TS evidence returned an unsupported schemaVersion');
  }
  if (!isObject(response.symbols) ||
      !isObject(response.topology) ||
      !isObject(response.shapeIndex) ||
      !isObject(response.anyInventory)) {
    throw new Error('Rust JS/TS evidence omitted symbols, topology, shapeIndex, or anyInventory');
  }
  if (!Array.isArray(response.shapeIntentNormalizations) ||
      response.shapeIntentNormalizations.some((entry) =>
        !isObject(entry) || typeof entry.typeLiteral !== 'string' || typeof entry.ok !== 'boolean')) {
    throw new Error('Rust JS/TS evidence returned invalid shape intent normalizations');
  }
  if (response.symbols.meta?.evidenceArtifact !== expectedArtifact ||
      response.topology.meta?.evidenceArtifact !== expectedArtifact) {
    throw new Error('Rust JS/TS evidence returned the wrong artifact identity');
  }
  if (!Array.isArray(response.files) || response.files.some((file) =>
    typeof file !== 'string' || file.length === 0 || path.isAbsolute(file) ||
    file.split('/').includes('..'))) {
    throw new Error('Rust JS/TS evidence returned an invalid file inventory');
  }
  const sortedFiles = [...new Set(response.files)].sort();
  if (JSON.stringify(sortedFiles) !== JSON.stringify(response.files)) {
    throw new Error('Rust JS/TS evidence returned an unsorted or duplicate file inventory');
  }
  const expectedFiles = response.files.length;
  if (response.summary?.fileCount !== expectedFiles) {
    throw new Error(
      `Rust JS/TS evidence returned ${response.summary?.fileCount ?? 'unknown'} files; expected ${expectedFiles}`,
    );
  }
  if (response.anyInventory.meta?.artifact !== expectedInventory ||
      response.anyInventory.meta?.fileCount !== expectedFiles ||
      response.anyInventory.meta?.supports?.typeEscapes !== true ||
      !Array.isArray(response.anyInventory.meta?.supports?.escapeKinds) ||
      !Array.isArray(response.anyInventory.typeEscapes)) {
    throw new Error('Rust JS/TS evidence returned an invalid type-escape inventory');
  }
  if (response.symbols.meta?.supports?.identityFanIn !== true ||
      response.symbols.meta?.supports?.dependencyImportConsumers !== true ||
      response.shapeIndex.schemaVersion !== 'shape-index.v1' ||
      response.shapeIndex.meta?.supports?.normalizedVersion !== 'shape-hash.normalized.v1' ||
      !Array.isArray(response.shapeIndex.facts) ||
      !isObject(response.shapeIndex.groupsByHash) ||
      !Array.isArray(response.shapeIndex.diagnostics) ||
      typeof response.topology.meta?.complete !== 'boolean') {
    throw new Error('Rust JS/TS evidence omitted required capability guarantees');
  }
  return response;
}

export function collectRustJsTsEvidence({
  root,
  evidenceArtifact,
  anyInventoryArtifact,
  includeTests,
  exclude = [],
  dependencySpecifiers = [],
  shapeTypeLiterals = [],
  noIncremental = false,
  cacheRoot = null,
  clearIncrementalCache = false,
  label = 'Rust JS/TS evidence',
}) {
  const dependencyRoots = [...new Set(
    dependencySpecifiers.map(packageRoot).filter(Boolean),
  )].sort();
  const request = {
    schemaVersion: REQUEST_SCHEMA_VERSION,
    root,
    evidenceArtifact,
    anyInventoryArtifact,
    generated: new Date().toISOString(),
    includeTests: includeTests === true,
    excludes: [...exclude],
    dependencyRoots,
    shapeTypeLiterals: [...new Set(shapeTypeLiterals)].sort(),
    discoverFiles: true,
    files: [],
    incremental: {
      enabled: noIncremental !== true,
      cacheRoot: path.resolve(cacheRoot ?? path.join(root, '.audit', '.cache')),
      clear: clearIncrementalCache === true,
    },
  };
  const args = ['js-ts-pre-write-evidence', '--input', '-'];
  const windowsRoot = wslPathToWindowsHost(request.root);
  const windowsCacheRoot = request.incremental.enabled
    ? wslPathToWindowsHost(request.incremental.cacheRoot)
    : null;
  let response;
  if (windowsRoot && (!request.incremental.enabled || windowsCacheRoot)) {
    response = runWindowsHostAuditCoreJsonResultFile(
      args,
      label,
      {
        input: JSON.stringify({
          ...request,
          root: windowsRoot,
          incremental: {
            ...request.incremental,
            ...(request.incremental.enabled ? { cacheRoot: windowsCacheRoot } : {}),
          },
        }),
        ...(request.incremental.enabled
          ? { resultTempRoot: request.incremental.cacheRoot }
          : {}),
      },
    );
    if (isObject(response)) {
      response.root = request.root;
      if (isObject(response.anyInventory?.meta)) {
        response.anyInventory.meta.root = request.root;
      }
      const incremental = response.anyInventory?.meta?.incremental;
      if (isObject(incremental)) {
        incremental.cacheRoot = request.incremental.cacheRoot;
        if (typeof incremental.cacheFile === 'string') {
          const cacheFile = windowsHostPathToWsl(incremental.cacheFile);
          if (!cacheFile) {
            throw new Error('Rust JS/TS evidence returned an untranslatable host cache path');
          }
          incremental.cacheFile = cacheFile;
        }
      }
    }
  }
  if (response === undefined) {
    response = runAuditCoreJsonResultFile(
      args,
      label,
      { input: JSON.stringify(request) },
    );
  }
  return validateResponse(response, evidenceArtifact, anyInventoryArtifact);
}
