import path from 'node:path';

import { runAuditCoreJsonResultFile } from './audit-core.mjs';
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
      !isObject(response.anyInventory)) {
    throw new Error('Rust JS/TS evidence omitted symbols, topology, or anyInventory');
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
    discoverFiles: true,
    files: [],
    incremental: {
      enabled: noIncremental !== true,
      cacheRoot: path.resolve(cacheRoot ?? path.join(root, '.audit', '.cache')),
      clear: clearIncrementalCache === true,
    },
  };
  const response = runAuditCoreJsonResultFile(
    ['js-ts-pre-write-evidence', '--input', '-'],
    label,
    { input: JSON.stringify(request) },
  );
  return validateResponse(response, evidenceArtifact, anyInventoryArtifact);
}
