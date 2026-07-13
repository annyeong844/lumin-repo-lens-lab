import { mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';

import { atomicWrite } from './atomic-write.mjs';
import { collectRustJsTsEvidence } from './js-ts-rust-evidence.mjs';

function evidenceFileName(invocationId) {
  return `pre-write-evidence.${invocationId}.json`;
}

export function buildRustPreWriteEvidence({
  root,
  output,
  invocationId,
  includeTests,
  exclude = [],
  dependencySpecifiers = [],
  shapeTypeLiterals = [],
  noIncremental = false,
  cacheRoot = null,
  clearIncrementalCache = false,
}) {
  const artifactName = evidenceFileName(invocationId);
  const anyInventoryArtifact = `any-inventory.pre.${invocationId}.json`;
  const evidence = collectRustJsTsEvidence({
    root,
    evidenceArtifact: artifactName,
    anyInventoryArtifact,
    includeTests,
    exclude,
    dependencySpecifiers,
    shapeTypeLiterals,
    noIncremental,
    cacheRoot,
    clearIncrementalCache,
    label: 'pre-write Rust evidence',
  });
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
