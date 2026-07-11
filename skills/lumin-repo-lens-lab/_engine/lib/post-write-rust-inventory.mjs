import { mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';

import { atomicWrite } from './atomic-write.mjs';
import { collectRustJsTsEvidence } from './js-ts-rust-evidence.mjs';

export function buildRustPostWriteInventory({
  root,
  output,
  deltaInvocationId,
  includeTests,
  exclude = [],
}) {
  const anyInventoryArtifact = 'any-inventory.json';
  const inventoryPath = path.join(output, anyInventoryArtifact);
  mkdirSync(output, { recursive: true });
  rmSync(inventoryPath, { force: true });

  const evidence = collectRustJsTsEvidence({
    root,
    evidenceArtifact: `post-write-evidence.${deltaInvocationId}.json`,
    anyInventoryArtifact,
    includeTests,
    exclude,
    label: 'post-write Rust after-inventory',
  });
  try {
    atomicWrite(inventoryPath, `${JSON.stringify(evidence.anyInventory, null, 2)}\n`);
  } catch (error) {
    rmSync(inventoryPath, { force: true });
    throw error;
  }
  return {
    inventory: evidence.anyInventory,
    inventoryPath,
    files: evidence.files,
  };
}
