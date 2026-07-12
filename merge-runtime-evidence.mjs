// merge-runtime-evidence.mjs — runtime coverage evidence wrapper.
//
// JS owns coverage-file discovery and artifact I/O. lumin-audit-core owns
// deterministic runtime-evidence projection from symbols.json + Istanbul JSON.

import { existsSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import { runAuditCoreJsonResultFile } from './_lib/audit-core.mjs';
import { parseCliArgs } from './_lib/cli.mjs';

const cli = parseCliArgs({
  coverage: { type: 'string' },
});
const { root: ROOT, output: OUTPUT, verbose } = cli;
const coverageArg = cli.raw.coverage;

function locateCoverage() {
  if (coverageArg) {
    const p = path.resolve(coverageArg);
    if (!existsSync(p)) throw new Error(`--coverage not found: ${p}`);
    return p;
  }
  const candidates = [
    path.join(ROOT, 'coverage', 'coverage-final.json'),
    path.join(ROOT, '.nyc_output', 'coverage-final.json'),
  ];
  for (const candidate of candidates) {
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

const coveragePath = locateCoverage();
if (!coveragePath) {
  console.error('[merge-rt] no coverage-final.json found.');
  console.error('[merge-rt] run tests with coverage first, e.g.:');
  console.error('[merge-rt]   npx c8 --reporter=json npm test');
  console.error('[merge-rt]   npx nyc --reporter=json npm test');
  console.error('[merge-rt] or pass --coverage <path> explicitly.');
  process.exit(2);
}
if (verbose) console.error(`[merge-rt] coverage: ${coveragePath}`);

const symbolsPath = path.join(OUTPUT, 'symbols.json');
if (!existsSync(symbolsPath)) {
  console.error(`[merge-rt] missing ${symbolsPath} — run build-symbol-graph.mjs first.`);
  process.exit(2);
}

const symbolsData = JSON.parse(readFileSync(symbolsPath, 'utf8'));
const coverageRaw = JSON.parse(readFileSync(coveragePath, 'utf8'));
const coverageStat = statSync(coveragePath);

const artifact = runAuditCoreJsonResultFile(
  ['runtime-evidence-artifact', '--input', '-'],
  'runtime-evidence-artifact',
  {
    input: JSON.stringify({
      schemaVersion: 'lumin-runtime-evidence-producer-request.v1',
      root: ROOT,
      generated: new Date().toISOString(),
      symbolsSource: symbolsPath,
      coverageSource: coveragePath,
      coverageMtime: coverageStat.mtime.toISOString(),
      symbols: symbolsData,
      coverage: coverageRaw,
    }),
  },
);

console.log(`[merge-rt] ${artifact.summary.total} dead candidates, ${artifact.summary.coverageFileCount} files in coverage`);
console.log('\n══════ runtime-fused grounding ══════');
console.log(`  grounded  (static-dead + runtime zero)     : ${artifact.summary.grounded_dead}`);
console.log(`  degraded/FP suspect (runtime hit > 0)      : ${artifact.summary.degraded_fp_suspect}`);
console.log(`  degraded/file-untested (0% coverage)       : ${artifact.summary.degraded_file_untested}`);
console.log(`  degraded/uncovered (file absent)           : ${artifact.summary.degraded_uncovered}`);
console.log(`  degraded/type-only (runtime n/a)           : ${artifact.summary.degraded_type_only}`);
console.log(`  ─────────────────────────────────────────── `);
console.log(`  total                                      : ${artifact.summary.total}`);
console.log(`\n  grounded share: ${artifact.summary.groundedSharePct}% of all dead candidates`);

if (artifact.summary.degraded_fp_suspect > 0) {
  console.log(`\n⚠ ${artifact.summary.degraded_fp_suspect} candidates have runtime hits — these are probable FPs.`);
  const sample = artifact.merged
    .filter((finding) => finding.runtimeStatus === 'executed')
    .slice(0, 10);
  for (const finding of sample) {
    console.log(`    ${finding.file}:${finding.line}  ${finding.symbol}  (${finding.hitsInSymbol}× hits)`);
  }
}

const outPath = path.join(OUTPUT, 'runtime-evidence.json');
writeFileSync(outPath, JSON.stringify(artifact, null, 2));
console.log(`\n[merge-rt] saved → ${outPath}`);
