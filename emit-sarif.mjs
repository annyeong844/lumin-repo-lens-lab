#!/usr/bin/env node
// emit-sarif.mjs — compatibility wrapper for Rust-owned SARIF projection.

import { writeFileSync } from 'node:fs';
import path from 'node:path';

import { runAuditCoreJsonResultFile } from './_lib/audit-core.mjs';
import { loadIfExists as loadArtifact } from './_lib/artifacts.mjs';
import { parseCliArgs } from './_lib/cli.mjs';

const SARIF_REQUEST_SCHEMA_VERSION = 'lumin-sarif-producer-request.v1';

const cli = parseCliArgs({
  'out-sarif': { type: 'string' },
});
const { root: ROOT, output: OUTPUT } = cli;
const outPath = cli.raw['out-sarif'] ?? path.join(OUTPUT, 'lumin-repo-lens-lab.sarif');

const read = (name) => loadArtifact(OUTPUT, name);
const sarif = runAuditCoreJsonResultFile(
  ['sarif-artifact', '--input', '-'],
  'sarif-artifact',
  {
    input: JSON.stringify({
      schemaVersion: SARIF_REQUEST_SCHEMA_VERSION,
      root: ROOT,
      generated: new Date().toISOString(),
      fixPlan: read('fix-plan.json'),
      runtimeEvidence: read('runtime-evidence.json'),
      staleness: read('staleness.json'),
      deadClassify: read('dead-classify.json'),
      symbols: read('symbols.json'),
      topology: read('topology.json'),
      discipline: read('discipline.json'),
      barrels: read('barrels.json'),
    }),
  },
);

writeFileSync(outPath, JSON.stringify(sarif, null, 2));

const run = sarif.runs?.[0] ?? {};
const results = run.results ?? [];
const artifactsUsed = run.properties?.artifactsUsed ?? [];
const byRule = new Map();
const byLevel = { error: 0, warning: 0, note: 0 };
for (const result of results) {
  byRule.set(result.ruleId, (byRule.get(result.ruleId) ?? 0) + 1);
  byLevel[result.level] = (byLevel[result.level] ?? 0) + 1;
}

console.log(`[sarif] ${results.length} findings from ${artifactsUsed.length} artifacts`);
const rules = run.tool?.driver?.rules ?? [];
for (const rule of rules) {
  const count = byRule.get(rule.id);
  if (count) {
    console.log(`  ${rule.id} ${rule.name.padEnd(24)} ${count}`);
  }
}
console.log(`  by level: warning=${byLevel.warning}, note=${byLevel.note}, error=${byLevel.error}`);
console.log(`[sarif] artifacts used: ${artifactsUsed.join(', ') || '(none — nothing to report)'}`);
console.log(`[sarif] saved → ${outPath}`);
