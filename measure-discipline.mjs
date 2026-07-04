#!/usr/bin/env node
// measure-discipline.mjs — thin wrapper for Rust-owned discipline counters.

import { writeFileSync } from 'node:fs';
import path from 'node:path';

import { parseCliArgs } from './_lib/cli.mjs';
import { collectFiles } from './_lib/collect-files.mjs';
import { JS_FAMILY_LANGS } from './_lib/lang.mjs';
import { relPath } from './_lib/paths.mjs';
import { runAuditCoreJsonResultFile } from './_lib/audit-core.mjs';

const cli = parseCliArgs();
const { root, output } = cli;

// Keep scan-scope ownership in JS for this slice. Rust owns the regex
// counting, unreadable-file accounting, rates, and discipline.json projection.
const langList = [...JS_FAMILY_LANGS, 'py', 'go'];
const files = collectFiles(root, {
  languages: langList,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
}).map((file) => relPath(root, file));
const pyCount = files.filter((file) => file.endsWith('.py')).length;
const goCount = files.filter((file) => file.endsWith('.go')).length;

console.error(
  `[discipline] scanning ${files.length} files (${pyCount} .py, ${goCount} .go) ...`
);

const artifact = runAuditCoreJsonResultFile(
  ['discipline-artifact', '--input', '-'],
  'discipline-artifact',
  {
    input: JSON.stringify({
      schemaVersion: 'lumin-discipline-producer-request.v1',
      generated: new Date().toISOString(),
      root,
      files,
    }),
  }
);

const outPath = path.join(output, 'discipline.json');
writeFileSync(outPath, JSON.stringify(artifact, null, 2));

console.log(
  `[discipline] ${artifact.scannedFiles} files scanned (${pyCount} .py, ${goCount} .go)`
);
if (artifact.unreadableFiles > 0) {
  console.warn(`[discipline] WARN: ${artifact.unreadableFiles} file(s) could not be read — totals may be low. Check permissions/symlinks.`);
}
const totals = artifact.totals ?? {};
const tsSummary = `:any=${totals[':any']}, as any=${totals['as any']}, @ts-ignore=${totals['@ts-ignore']}`;
const pySummary = pyCount > 0
  ? `, # type: ignore=${totals['# type: ignore'] ?? 0}, # noqa=${totals['# noqa'] ?? 0}, eval(=${totals['eval('] ?? 0}`
  : '';
const goSummary = goCount > 0
  ? `, interface{}=${totals['interface{}'] ?? 0}, panic(=${totals['panic('] ?? 0}, unsafe.=${totals['unsafe.'] ?? 0}`
  : '';
console.log(`[discipline] ${tsSummary}${pySummary}${goSummary}, TODO=${totals.TODO}`);
console.log(`[discipline] saved → ${outPath}`);
