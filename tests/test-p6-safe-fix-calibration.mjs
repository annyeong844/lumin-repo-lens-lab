// P6 SAFE_FIX calibration corpus.
//
// This suite builds a tiny real git repo, runs the production artifact
// pipeline, injects a minimal Istanbul coverage file, and asserts that
// multi-source evidence can actually produce SAFE_FIX. Large real-world
// corpora may legitimately have SAFE_FIX=0; this fixture proves the
// zero is corpus evidence, not a broken ranking path.

import { execFileSync } from 'node:child_process';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function runScript(script, args, options = {}) {
  execFileSync(NODE, [path.join(DIR, script), ...args], {
    cwd: DIR,
    stdio: ['ignore', 'pipe', 'pipe'],
    ...options,
  });
}

function git(root, args, options = {}) {
  return execFileSync('git', args, {
    cwd: root,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    ...options,
  }).trim();
}

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, 'utf8'));
}

function coverageEntry(absFile, symbol, line, hits) {
  return {
    path: absFile,
    statementMap: {
      0: {
        start: { line, column: 0 },
        end: { line: line + 2, column: 1 },
      },
    },
    s: { 0: hits },
    fnMap: {
      0: {
        name: symbol,
        decl: {
          start: { line, column: 16 },
          end: { line, column: 16 + symbol.length },
        },
        loc: {
          start: { line, column: 0 },
          end: { line: line + 2, column: 1 },
        },
        line,
      },
    },
    f: { 0: hits },
    branchMap: {},
    b: {},
  };
}

function bySymbol(list, symbol) {
  return list.find((entry) => entry.symbol === symbol || entry.finding?.symbol === symbol);
}

const root = mkdtempSync(path.join(tmpdir(), 'p6-safe-calibration-'));
const out = mkdtempSync(path.join(tmpdir(), 'p6-safe-calibration-out-'));
const adjudicationPath = path.join(out, 'adjudication.json');
const coveragePath = path.join(out, 'coverage-final.json');

try {
  write(root, 'package.json', JSON.stringify({
    name: 'p6-safe-fix-calibration',
    private: true,
    type: 'module',
  }, null, 2));
  write(root, 'src/dead-runtime.ts',
    `export function staleRuntimeDead() {\n` +
    `  return 'remove-me';\n` +
    `}\n`);
  write(root, 'src/executed-runtime.ts',
    `export function runtimeSeen() {\n` +
    `  return 'dynamic-use';\n` +
    `}\n`);
  write(root, 'src/uncovered-runtime.ts',
    `export function uncoveredRuntime() {\n` +
    `  return 'missing-test-range';\n` +
    `}\n`);
  write(root, 'src/type-only.ts',
    `export interface ColdTypeOnly {\n` +
    `  value: string;\n` +
    `}\n`);

  git(root, ['init', '-q']);
  git(root, ['config', 'user.email', 'p6-safe-fix@example.test']);
  git(root, ['config', 'user.name', 'P6 SAFE_FIX Fixture']);
  git(root, ['add', '.']);
  git(root, ['commit', '-m', 'seed safe-fix calibration corpus'], {
    env: {
      ...process.env,
      GIT_AUTHOR_DATE: '2024-01-01T00:00:00 +0000',
      GIT_COMMITTER_DATE: '2024-01-01T00:00:00 +0000',
    },
  });

  runScript('build-symbol-graph.mjs', ['--root', root, '--output', out]);
  runScript('classify-dead-exports.mjs', ['--root', root, '--output', out]);
  runScript('measure-staleness.mjs', ['--root', root, '--output', out]);

  const symbols = readJson(path.join(out, 'symbols.json'));
  const deadList = symbols.deadProdList ?? [];
  const stale = bySymbol(deadList, 'staleRuntimeDead');
  const seen = bySymbol(deadList, 'runtimeSeen');
  const uncovered = bySymbol(deadList, 'uncoveredRuntime');
  const typeOnly = bySymbol(deadList, 'ColdTypeOnly');

  assert('P6S-1a. calibration corpus emits the runtime-dead candidate',
    !!stale, `deadList=${JSON.stringify(deadList)}`);
  assert('P6S-1b. calibration corpus emits the runtime-hit candidate',
    !!seen, `deadList=${JSON.stringify(deadList)}`);
  assert('P6S-1c. calibration corpus emits the uncovered candidate',
    !!uncovered, `deadList=${JSON.stringify(deadList)}`);
  assert('P6S-1d. calibration corpus emits the type-only candidate',
    !!typeOnly && typeOnly.kind === 'TSInterfaceDeclaration',
    `typeOnly=${JSON.stringify(typeOnly)}`);

  const coverage = {};
  if (stale) {
    const abs = path.join(root, stale.file);
    coverage[abs] = coverageEntry(abs, stale.symbol, stale.line, 0);
  }
  if (seen) {
    const abs = path.join(root, seen.file);
    coverage[abs] = coverageEntry(abs, seen.symbol, seen.line, 3);
  }
  writeFileSync(coveragePath, JSON.stringify(coverage, null, 2));

  runScript('merge-runtime-evidence.mjs', [
    '--root', root,
    '--output', out,
    '--coverage', coveragePath,
  ]);
  runScript('export-action-safety.mjs', ['--root', root, '--output', out]);
  runScript('rank-fixes.mjs', ['--root', root, '--output', out]);

  const runtime = readJson(path.join(out, 'runtime-evidence.json'));
  const merged = runtime.merged ?? [];
  const rtStale = bySymbol(merged, 'staleRuntimeDead');
  const rtSeen = bySymbol(merged, 'runtimeSeen');
  const rtUncovered = bySymbol(merged, 'uncoveredRuntime');
  const rtTypeOnly = bySymbol(merged, 'ColdTypeOnly');

  assert('P6S-2a. covered zero-hit runtime symbol is dead-confirmed',
    rtStale?.runtimeStatus === 'dead-confirmed' && rtStale.grounding === 'grounded',
    JSON.stringify(rtStale));
  assert('P6S-2b. runtime-hit static-dead symbol is marked executed',
    rtSeen?.runtimeStatus === 'executed' && rtSeen.hitsInSymbol === 3,
    JSON.stringify(rtSeen));
  assert('P6S-2c. file absent from coverage is uncovered, not dead-confirmed',
    rtUncovered?.runtimeStatus === 'uncovered',
    JSON.stringify(rtUncovered));
  assert('P6S-2d. erased interface is type-only runtime evidence',
    rtTypeOnly?.runtimeStatus === 'type-only',
    JSON.stringify(rtTypeOnly));

  const plan = readJson(path.join(out, 'fix-plan.json'));
  const safe = plan.safeFixes.find((entry) => entry.finding.symbol === 'staleRuntimeDead');
  const degraded = plan.degraded.find((entry) => entry.finding.symbol === 'runtimeSeen');
  const reviewUncovered = plan.reviewFixes.find((entry) => entry.finding.symbol === 'uncoveredRuntime');
  const reviewType = plan.reviewFixes.find((entry) => entry.finding.symbol === 'ColdTypeOnly');

  assert('P6S-3a. SAFE_FIX path is reachable with AST + runtime + stale evidence',
    plan.summary.SAFE_FIX === 1 && safe?.reason.includes('runtime-dead-confirmed'),
    `summary=${JSON.stringify(plan.summary)} safe=${JSON.stringify(safe)}`);
  assert('P6S-3b. runtime-hit contradiction is DEGRADED, never SAFE_FIX',
    plan.summary.DEGRADED === 1 && degraded?.reason.includes('runtime-executed'),
    `summary=${JSON.stringify(plan.summary)} degraded=${JSON.stringify(plan.degraded)}`);
  assert('P6S-3c. uncovered runtime range stays REVIEW_FIX',
    !!reviewUncovered && reviewUncovered.reason.includes('runtime=uncovered'),
    JSON.stringify(reviewUncovered));
  assert('P6S-3d. type-only export stays REVIEW_FIX',
    !!reviewType && reviewType.reason.includes('runtime=type-only'),
    JSON.stringify(reviewType));

  writeFileSync(path.join(out, 'canon-drift.json'), JSON.stringify({
    summary: { driftCount: 0 },
    perSource: { naming: { status: 'clean', driftCount: 0 } },
  }, null, 2));
  writeFileSync(adjudicationPath, JSON.stringify({
    entries: [
      {
        corpusName: 'safe-fix-calibration',
        tier: 'SAFE_FIX',
        verdict: 'true_dead',
        file: safe?.finding.file,
        symbol: safe?.finding.symbol,
      },
    ],
  }, null, 2));

  const commit = git(root, ['rev-parse', 'HEAD']);
  runScript('p6-measurement.mjs', [
    '--root', root,
    '--output', out,
    '--corpus-name', 'safe-fix-calibration',
    '--repo', 'local-fixture',
    '--commit', commit,
    '--worktree-dirty', 'false',
    '--loc-bucket', 'other',
    '--adjudication', adjudicationPath,
  ]);

  const measurement = readJson(path.join(out, 'p6-measurement.json'));
  const reasonCodes = new Set((measurement.readiness.reasons ?? []).map((r) => r.code));
  assert('P6S-4a. P6 measurement sees the non-empty SAFE_FIX population',
    measurement.candidateCounts.safeFix === 1 &&
    measurement.candidateCounts.reviewVisibleCleanup === 3,
    JSON.stringify(measurement.candidateCounts));
  assert('P6S-4b. SAFE_FIX adjudication denominator is known and zero-FP',
    measurement.readiness.safeFix.fpRate === 0 &&
    !reasonCodes.has('fp-rate-unknown') &&
    !reasonCodes.has('safe-fix-population-empty'),
    JSON.stringify(measurement.readiness));
  assert('P6S-4c. one tiny local corpus remains Yellow, not Green',
    measurement.readiness.gate === 'Yellow' &&
    reasonCodes.has('benchmark-incomplete'),
    JSON.stringify(measurement.readiness));
} finally {
  rmSync(root, { recursive: true, force: true });
  rmSync(out, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
