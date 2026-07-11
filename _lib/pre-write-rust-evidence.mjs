import { mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';

import { runAuditCoreJsonResultFile } from './audit-core.mjs';
import { atomicWrite } from './atomic-write.mjs';
import { packageRoot } from './pre-write-lookup-dep.mjs';

const REQUEST_SCHEMA_VERSION = 'lumin-js-ts-pre-write-evidence-request.v1';
const RESPONSE_SCHEMA_VERSION = 'lumin-js-ts-pre-write-evidence-response.v1';

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function evidenceFileName(invocationId) {
  return `pre-write-evidence.${invocationId}.json`;
}

function validateResponse(response, expectedArtifact, expectedInventory) {
  if (response?.schemaVersion !== RESPONSE_SCHEMA_VERSION) {
    throw new Error('pre-write Rust evidence returned an unsupported schemaVersion');
  }
  if (!isObject(response.symbols) ||
      !isObject(response.topology) ||
      !isObject(response.anyInventory)) {
    throw new Error('pre-write Rust evidence omitted symbols, topology, or anyInventory');
  }
  if (response.symbols.meta?.evidenceArtifact !== expectedArtifact ||
      response.topology.meta?.evidenceArtifact !== expectedArtifact) {
    throw new Error('pre-write Rust evidence returned the wrong artifact identity');
  }
  if (!Array.isArray(response.files) || response.files.some((file) =>
    typeof file !== 'string' || file.length === 0 || path.isAbsolute(file) ||
    file.split('/').includes('..'))) {
    throw new Error('pre-write Rust evidence returned an invalid file inventory');
  }
  const sortedFiles = [...new Set(response.files)].sort();
  if (JSON.stringify(sortedFiles) !== JSON.stringify(response.files)) {
    throw new Error('pre-write Rust evidence returned an unsorted or duplicate file inventory');
  }
  const expectedFiles = response.files.length;
  if (response.summary?.fileCount !== expectedFiles) {
    throw new Error(
      `pre-write Rust evidence returned ${response.summary?.fileCount ?? 'unknown'} files; expected ${expectedFiles}`,
    );
  }
  if (response.anyInventory.meta?.artifact !== expectedInventory ||
      response.anyInventory.meta?.fileCount !== expectedFiles ||
      response.anyInventory.meta?.supports?.typeEscapes !== true ||
      !Array.isArray(response.anyInventory.meta?.supports?.escapeKinds) ||
      !Array.isArray(response.anyInventory.typeEscapes)) {
    throw new Error('pre-write Rust evidence returned an invalid type-escape baseline');
  }
  if (response.symbols.meta?.supports?.identityFanIn !== true ||
      response.symbols.meta?.supports?.dependencyImportConsumers !== true ||
      typeof response.topology.meta?.complete !== 'boolean') {
    throw new Error('pre-write Rust evidence omitted required capability guarantees');
  }
  return response;
}

export function buildRustPreWriteEvidence({
  root,
  output,
  invocationId,
  includeTests,
  exclude = [],
  dependencySpecifiers = [],
}) {
  const artifactName = evidenceFileName(invocationId);
  const anyInventoryArtifact = `any-inventory.pre.${invocationId}.json`;
  const dependencyRoots = [...new Set(
    dependencySpecifiers.map(packageRoot).filter(Boolean),
  )].sort();
  const request = {
    schemaVersion: REQUEST_SCHEMA_VERSION,
    root,
    evidenceArtifact: artifactName,
    anyInventoryArtifact,
    generated: new Date().toISOString(),
    includeTests: includeTests === true,
    excludes: [...exclude],
    dependencyRoots,
    discoverFiles: true,
    files: [],
  };
  const response = runAuditCoreJsonResultFile(
    ['js-ts-pre-write-evidence', '--input', '-'],
    'pre-write Rust evidence',
    { input: JSON.stringify(request) },
  );
  const evidence = validateResponse(
    response,
    artifactName,
    anyInventoryArtifact,
  );
  const content = `${JSON.stringify(evidence, null, 2)}\n`;
  const inventoryContent = `${JSON.stringify(evidence.anyInventory, null, 2)}\n`;
  const specificPath = path.join(output, artifactName);
  const latestPath = path.join(output, 'pre-write-evidence.latest.json');
  const anyInventoryPath = path.join(output, anyInventoryArtifact);
  mkdirSync(output, { recursive: true });
  try {
    atomicWrite(anyInventoryPath, inventoryContent);
    atomicWrite(specificPath, content);
    atomicWrite(latestPath, content);
  } catch (error) {
    rmSync(anyInventoryPath, { force: true });
    rmSync(specificPath, { force: true });
    throw error;
  }
  return {
    artifactName,
    specificPath,
    latestPath,
    anyInventoryArtifact,
    anyInventoryPath,
    evidence,
    files: evidence.files.map((file) => path.join(root, ...file.split('/'))),
  };
}
