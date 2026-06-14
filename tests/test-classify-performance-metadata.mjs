// classify-dead-exports performance metadata and per-file AST batching.

import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
let passed = 0, failed = 0;

function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

function write(root, rel, content) {
  const target = path.join(root, rel);
  mkdirSync(path.dirname(target), { recursive: true });
  writeFileSync(target, content);
}

function run(script, args) {
  execFileSync(process.execPath, [path.join(DIR, script), ...args], {
    cwd: DIR,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

const fx = mkdtempSync(path.join(os.tmpdir(), 'classify-perf-fx-'));
const fxTextZero = mkdtempSync(path.join(os.tmpdir(), 'classify-perf-text-zero-fx-'));
const out = mkdtempSync(path.join(os.tmpdir(), 'classify-perf-out-'));
const outLimited = mkdtempSync(path.join(os.tmpdir(), 'classify-perf-limited-out-'));
const outBudget = mkdtempSync(path.join(os.tmpdir(), 'classify-perf-budget-out-'));
const outSized = mkdtempSync(path.join(os.tmpdir(), 'classify-perf-sized-out-'));
const outTextZero = mkdtempSync(path.join(os.tmpdir(), 'classify-perf-text-zero-out-'));

try {
  write(fx, 'package.json', JSON.stringify({ name: 'classify-perf', type: 'module', private: true }));
  write(fx, 'src/dead.ts',
    `export const Alpha = 1;\n` +
    `export const Beta = 2;\n` +
    `const gamma = Alpha + Beta;\n`);
  write(fx, 'src/text-zero.ts',
    `export const Gamma = 3;\n` +
    `export const Delta = 4;\n`);
  write(fxTextZero, 'package.json',
    JSON.stringify({ name: 'classify-perf-text-zero', type: 'module', private: true }));
  write(fxTextZero, 'src/text-zero.ts',
    `export const Gamma = 3;\n` +
    `export const Delta = 4;\n`);

  run('build-symbol-graph.mjs', ['--root', fx, '--output', out]);
  run('classify-dead-exports.mjs', ['--root', fx, '--output', out]);

  const artifact = JSON.parse(readFileSync(path.join(out, 'dead-classify.json'), 'utf8'));
  const perf = artifact.summary?.performance;

  assert('T1. classify summary carries performance metadata',
    perf && typeof perf === 'object',
    `summary=${JSON.stringify(artifact.summary)}`);
  assert('T2. performance metadata records processed dead candidates',
    perf?.deadCandidatesProcessed === 4,
    `performance=${JSON.stringify(perf)}`);
  assert('T3. same-file candidates are AST-counted through one file batch',
    perf?.astFilesParsed === 1,
    `performance=${JSON.stringify(perf)}`);
  assert('T4. no candidate cap was applied',
    perf?.candidateLimitApplied === false,
    `performance=${JSON.stringify(perf)}`);
  assert('T4b. file-size degrade is opt-in, not a default accuracy policy',
    perf?.maxFileBytes === 0 &&
      perf?.astFilesSkippedBySize === 0,
    `performance=${JSON.stringify(perf)}`);
  assert('T4c. text-zero candidates skip AST without degrading accuracy',
    perf?.textZeroCandidates === 2 &&
      perf?.textZeroFiles === 1,
    `performance=${JSON.stringify(perf)}`);
  assert('T4d. provenance work is cached per file, not repeated per symbol',
    perf?.provenanceCacheEntries === 2,
    `performance=${JSON.stringify(perf)}`);

  run('build-symbol-graph.mjs', ['--root', fxTextZero, '--output', outTextZero]);
  run('classify-dead-exports.mjs', [
    '--root', fxTextZero,
    '--output', outTextZero,
  ]);
  const textZero = JSON.parse(readFileSync(path.join(outTextZero, 'dead-classify.json'), 'utf8'));
  assert('T4e. all-text-zero batches can finish without parsing candidate files',
    textZero.summary?.performance?.textZeroCandidates === 2 &&
      textZero.summary?.performance?.astFilesParsed === 0,
    `performance=${JSON.stringify(textZero.summary?.performance)}`);

  run('build-symbol-graph.mjs', ['--root', fx, '--output', outLimited]);
  run('classify-dead-exports.mjs', [
    '--root', fx,
    '--output', outLimited,
    '--classify-candidate-limit', '1',
  ]);
  const limited = JSON.parse(readFileSync(path.join(outLimited, 'dead-classify.json'), 'utf8'));
  assert('T5. candidate cap marks classify artifact incomplete',
    limited.summary?.incomplete === true &&
      limited.summary?.performance?.candidateLimitApplied === true,
    `summary=${JSON.stringify(limited.summary)}`);
  assert('T6. candidate cap records total vs processed counts',
    limited.summary?.performance?.deadCandidatesTotal === 4 &&
      limited.summary?.performance?.deadCandidatesProcessed === 1,
    `performance=${JSON.stringify(limited.summary?.performance)}`);

  run('build-symbol-graph.mjs', ['--root', fx, '--output', outBudget]);
  run('classify-dead-exports.mjs', [
    '--root', fx,
    '--output', outBudget,
    '--classify-time-budget-ms', '1',
  ]);
  const budgeted = JSON.parse(readFileSync(path.join(outBudget, 'dead-classify.json'), 'utf8'));
  assert('T7. time budget marks classify artifact incomplete',
    budgeted.summary?.incomplete === true &&
      budgeted.summary?.performance?.timeBudgetExceeded === true,
    `summary=${JSON.stringify(budgeted.summary)}`);
  assert('T8. time-budgeted candidates are materialized as degraded proposals',
    Array.isArray(budgeted.proposal_DEGRADED_unprocessed) &&
      budgeted.proposal_DEGRADED_unprocessed.length > 0,
    `unprocessed=${JSON.stringify(budgeted.proposal_DEGRADED_unprocessed)}`);

  run('build-symbol-graph.mjs', ['--root', fx, '--output', outSized]);
  run('classify-dead-exports.mjs', [
    '--root', fx,
    '--output', outSized,
    '--classify-max-file-bytes', '10',
  ]);
  const sized = JSON.parse(readFileSync(path.join(outSized, 'dead-classify.json'), 'utf8'));
  assert('T9. oversized candidate files are degraded instead of AST-counted',
    sized.summary?.performance?.astFilesSkippedBySize === 2 &&
      sized.proposal_DEGRADED_unprocessed?.length === 4,
    `summary=${JSON.stringify(sized.summary)} unprocessed=${JSON.stringify(sized.proposal_DEGRADED_unprocessed)}`);
} finally {
  rmSync(fx, { recursive: true, force: true });
  rmSync(fxTextZero, { recursive: true, force: true });
  rmSync(out, { recursive: true, force: true });
  rmSync(outLimited, { recursive: true, force: true });
  rmSync(outBudget, { recursive: true, force: true });
  rmSync(outSized, { recursive: true, force: true });
  rmSync(outTextZero, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
