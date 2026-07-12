#!/usr/bin/env node
// Artifact-level repo comparison. Does NOT walk source trees: Rust audit-core
// reads already-produced audit artifacts and projects compare.json.

import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { runAuditCoreJsonResultFile } from './_lib/audit-core.mjs';

const { values } = parseArgs({
  options: {
    left: { type: 'string' },
    right: { type: 'string' },
    output: { type: 'string', short: 'o' },
    'left-label': { type: 'string' },
    'right-label': { type: 'string' },
  },
  strict: false,
});

if (!values.left || !values.right) {
  console.error('usage: compare-repos.mjs --left <dir> --right <dir> [--output <dir>] [--left-label NAME] [--right-label NAME]');
  process.exit(1);
}

const LEFT = path.resolve(values.left);
const RIGHT = path.resolve(values.right);
const OUT = path.resolve(values.output ?? path.join(process.cwd(), 'compare-output'));
const leftLabel = values['left-label'] ?? path.basename(LEFT);
const rightLabel = values['right-label'] ?? path.basename(RIGHT);

mkdirSync(OUT, { recursive: true });

const artifact = runAuditCoreJsonResultFile(
  ['compare-repos-artifact', '--input', '-'],
  'compare-repos',
  {
    input: JSON.stringify({
      schemaVersion: 'lumin-compare-repos-producer-request.v1',
      generated: new Date().toISOString(),
      left: LEFT,
      right: RIGHT,
      leftLabel,
      rightLabel,
    }),
  },
);

const outPath = path.join(OUT, 'compare.json');
writeFileSync(outPath, JSON.stringify(artifact, null, 2));

console.log('\n══════ audit-artifact compare ══════');
console.log(`  left:  ${artifact.left?.label ?? leftLabel}  (${artifact.left?.artifactsFound?.length ?? 0} artifacts)`);
console.log(`  right: ${artifact.right?.label ?? rightLabel}  (${artifact.right?.artifactsFound?.length ?? 0} artifacts)`);
console.log('');

const rows = [
  ['files', artifact.deltas?.files],
  ['loc', artifact.deltas?.loc],
  ['totalDefs', artifact.deltas?.totalDefs],
  ['deadInProd', artifact.deltas?.deadInProd],
  ['runtime SCCs', artifact.deltas?.runtimeSccs],
  ['SAFE_FIX', artifact.deltas?.safeFixes],
  ['REVIEW_FIX', artifact.deltas?.reviewFixes],
  ['DEGRADED', artifact.deltas?.degraded],
  ['MUTED', artifact.deltas?.muted],
];
for (const [label, value] of rows) {
  if (typeof value !== 'number') console.log(`  ${label.padEnd(14)} : (missing on one side)`);
  else console.log(`  ${label.padEnd(14)} : ${value >= 0 ? '+' : ''}${value}`);
}

const missingLeft = artifact.missingArtifacts?.left ?? [];
const missingRight = artifact.missingArtifacts?.right ?? [];
if (missingLeft.length || missingRight.length) {
  console.log('');
  if (missingLeft.length) console.log(`  missing on left (${leftLabel}):  ${missingLeft.join(', ')}`);
  if (missingRight.length) console.log(`  missing on right (${rightLabel}): ${missingRight.join(', ')}`);
}
console.log(`\n[compare] saved → ${outPath}`);
