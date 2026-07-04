#!/usr/bin/env node
// measure-staleness.mjs — git temporal evidence wrapper.
//
// JS owns CLI compatibility, symbols.json loading, artifact I/O, and console
// summary. lumin-audit-core owns git evidence collection, incremental cache
// behavior, tier classification, and staleness.json construction.

import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import { runAuditCoreJsonResultFile } from '../lib/audit-core.mjs';
import { parseCliArgs } from '../lib/cli.mjs';

const STALENESS_REQUEST_SCHEMA_VERSION = 'lumin-staleness-producer-request.v1';

const cli = parseCliArgs({
  'max-age-days': { type: 'string', default: '365' },
  'stale-age-days': { type: 'string', default: '90' },
  since: { type: 'string', default: '5 years ago' },
  'skip-pickaxe': { type: 'boolean', default: false },
  'no-incremental': { type: 'boolean', default: false },
  'cache-root': { type: 'string' },
  'clear-incremental-cache': { type: 'boolean', default: false },
});

const { root: ROOT, output: OUTPUT } = cli;
const maxAgeDays = Number(cli.raw['max-age-days'] ?? 365);
const staleAgeDays = Number(cli.raw['stale-age-days'] ?? 90);
const since = cli.raw.since ?? '5 years ago';
const skipPickaxe = cli.raw['skip-pickaxe'] === true;
const incrementalEnabled = cli.raw['no-incremental'] !== true;

const symbolsPath = path.join(OUTPUT, 'symbols.json');
if (!existsSync(symbolsPath)) {
  console.error(`[staleness] missing ${symbolsPath} — run build-symbol-graph.mjs first.`);
  process.exit(2);
}

const symbols = JSON.parse(readFileSync(symbolsPath, 'utf8'));
const deadList = symbols.deadProdList ?? [];
console.log(`[staleness] ${deadList.length} dead candidates — measuring git history ...`);

const artifact = runAuditCoreJsonResultFile(
  ['staleness-artifact', '--input', '-'],
  'staleness-artifact',
  {
    input: JSON.stringify({
      schemaVersion: STALENESS_REQUEST_SCHEMA_VERSION,
      root: ROOT,
      generated: new Date().toISOString(),
      symbolsSource: symbolsPath,
      symbols,
      maxAgeDays,
      staleAgeDays,
      since,
      skipPickaxe,
      incrementalEnabled,
      cacheRoot: cli.raw['cache-root'] ? path.resolve(cli.raw['cache-root']) : undefined,
      clearIncrementalCache: cli.raw['clear-incremental-cache'] === true,
    }),
  },
);

const byTier = artifact.summary?.byTier ?? {};
const byGrounding = artifact.summary?.byGrounding ?? {};
console.log('\n══════ staleness distribution ══════');
console.log(`  fossil (>=${maxAgeDays}d untouched)  : ${byTier.fossil ?? 0}`);
console.log(`  stale  (>=${staleAgeDays}d untouched)   : ${byTier.stale ?? 0}`);
console.log(`  recent (<${staleAgeDays}d - active)    : ${byTier.recent ?? 0}`);
console.log(`  unknown (untracked / no history)    : ${byTier.unknown ?? 0}`);
console.log('');
console.log(`  grounded : ${byGrounding.grounded ?? 0}`);
console.log(`  degraded : ${byGrounding.degraded ?? 0}`);
console.log(`  blind    : ${byGrounding.blind ?? 0}`);

const topFossils = (artifact.enriched ?? [])
  .filter((entry) => entry.stalenessTier === 'fossil')
  .sort((a, b) => (b.lineLastTouchedDaysAgo ?? 0) - (a.lineLastTouchedDaysAgo ?? 0))
  .slice(0, 10);
if (topFossils.length) {
  console.log('\n── oldest fossils (top 10) ──');
  for (const entry of topFossils) {
    const age = entry.lineLastTouchedDaysAgo ?? entry.fileLastTouchedDaysAgo ?? '?';
    console.log(`  ${entry.file}:${entry.line}  ${entry.symbol}  (last touched ${age}d ago)`);
  }
}

const recentRisky = (artifact.enriched ?? []).filter((entry) => entry.stalenessTier === 'recent');
if (recentRisky.length) {
  console.log(`\n! ${recentRisky.length} dead candidates touched within ${staleAgeDays}d — verify before removal.`);
  for (const entry of recentRisky.slice(0, 5)) {
    const age = entry.lineLastTouchedDaysAgo ?? entry.fileLastTouchedDaysAgo ?? '?';
    console.log(`    ${entry.file}:${entry.line}  ${entry.symbol}  (${age}d)`);
  }
}

const outPath = path.join(OUTPUT, 'staleness.json');
writeFileSync(outPath, JSON.stringify(artifact, null, 2));
console.log(`\n[staleness] saved → ${outPath}`);
