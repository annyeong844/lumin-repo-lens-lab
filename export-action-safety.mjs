#!/usr/bin/env node
// Select proof-carrying safe edit actions for dead-export findings.
//
// Input: dead-classify.json
// Output: export-action-safety.json

import path from 'node:path';

import { loadIfExists } from './_lib/artifacts.mjs';
import { parseCliArgs } from './_lib/cli.mjs';
import { atomicWrite } from './_lib/atomic-write.mjs';
import { projectExportActionSafetyArtifact } from './_lib/export-action-safety-artifact.mjs';
import { collectExportActionSafetyFacts } from './_lib/export-action-safety.mjs';

const { root, output } = parseCliArgs();
const ROOT = path.resolve(root);
const OUT = path.resolve(output);

const deadClassify = loadIfExists(OUT, 'dead-classify.json', { tag: 'export-action-safety' });
if (!deadClassify) {
  console.error('[export-action-safety] dead-classify.json is required. Run classify-dead-exports.mjs first.');
  process.exit(1);
}

const facts = collectExportActionSafetyFacts({ root: ROOT, deadClassify });
const artifact = projectExportActionSafetyArtifact(facts);
const outPath = path.join(OUT, 'export-action-safety.json');
atomicWrite(outPath, JSON.stringify(artifact, null, 2));

console.log('\n══════ export action safety ══════');
console.log(`  findings      : ${artifact.meta.total}`);
console.log(`  warnings      : ${artifact.meta.warnings.length}`);
console.log(`  wrote         : ${outPath}`);
