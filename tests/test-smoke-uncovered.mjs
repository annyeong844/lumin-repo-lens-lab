// Smoke tests for the 5 scripts that had no coverage prior to 1.5.0:
//   - build-call-graph.mjs
//   - check-barrel-discipline.mjs
//   - measure-discipline.mjs
//   - emit-sarif.mjs
//   - merge-runtime-evidence.mjs
//
// These are deliberately shallow: each test builds a minimal fixture,
// runs the script end-to-end, and asserts the expected artifact is
// produced, parses correctly, and has the expected shape. They're
// intended to catch total breakage — wrong imports, crashed parsers,
// missing writes, schema regressions — not to validate every edge
// case. Deeper assertions belong in dedicated suites.

import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, rmSync, existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const FX = '/tmp/fx-smoke-uncovered';
const OUT = '/tmp/out-smoke-uncovered';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function run(cmd, cwd = DIR) {
  try {
    return {
      ok: true,
      out: execSync(cmd, { cwd, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }),
      err: '',
    };
  } catch (e) {
    return { ok: false, out: e.stdout || '', err: e.stderr || e.message };
  }
}

function loadJson(artifactName) {
  const p = path.join(OUT, artifactName);
  if (!existsSync(p)) return null;
  try { return JSON.parse(readFileSync(p, 'utf8')); }
  catch { return null; }
}

// ── Build a minimal but semantically interesting fixture ─
// Two files with:
//   - a function call (exercises build-call-graph)
//   - a re-export barrel (exercises check-barrel-discipline)
//   - some exported + unused symbols (exercises measure-discipline)
rmSync(FX, { recursive: true, force: true });
rmSync(OUT, { recursive: true, force: true });
mkdirSync(path.join(FX, 'src'), { recursive: true });

writeFileSync(path.join(FX, 'package.json'),
  '{"name":"fx-smoke","type":"module"}');

writeFileSync(path.join(FX, 'src/lib.ts'),
  'export function helper(x: number): number {\n' +
  '  return x + 1;\n' +
  '}\n' +
  'export function unused(): void {}\n'
);

writeFileSync(path.join(FX, 'src/app.ts'),
  "import { helper } from './lib';\n" +
  'export function main(): number {\n' +
  '  return helper(1);\n' +
  '}\n'
);

// A barrel file — aggregate re-exports
writeFileSync(path.join(FX, 'src/index.ts'),
  "export { helper } from './lib';\n" +
  "export { main } from './app';\n"
);

// ─────────────────────────────────────────────────────────
// A. build-call-graph.mjs
// ─────────────────────────────────────────────────────────
{
  const r = run(`node build-call-graph.mjs --root ${FX} --output ${OUT}`);
  assert('A1. build-call-graph completes without error',
    r.ok, r.err.slice(0, 400));

  const art = loadJson('call-graph.json');
  assert('A2. call-graph.json exists and parses as JSON',
    art !== null,
    `artifact missing or unparseable at ${OUT}/call-graph.json`);

  // Schema sanity — any one of these top-level keys should exist for
  // the artifact to be useful downstream. Not asserting specific counts
  // because the file is small and parsers vary.
  const hasExpectedShape = art && typeof art === 'object' && (
    'edges' in art || 'summary' in art || 'meta' in art || 'nodes' in art
  );
  assert('A3. call-graph.json has recognizable top-level shape',
    hasExpectedShape,
    `keys: ${art ? Object.keys(art).slice(0, 8).join(', ') : '(null)'}`);
}

// ─────────────────────────────────────────────────────────
// B. check-barrel-discipline.mjs
// ─────────────────────────────────────────────────────────
{
  const r = run(`node check-barrel-discipline.mjs --root ${FX} --output ${OUT}`);
  assert('B1. check-barrel-discipline completes without error',
    r.ok, r.err.slice(0, 400));

  const art = loadJson('barrels.json');
  assert('B2. barrels.json exists and parses as JSON',
    art !== null,
    `artifact missing or unparseable at ${OUT}/barrels.json`);

  // The fixture has one barrel (src/index.ts) with 2 re-exports
  const hasShape = art && typeof art === 'object' && (
    'barrels' in art || 'summary' in art || 'meta' in art
  );
  assert('B3. barrels.json has recognizable top-level shape',
    hasShape,
    `keys: ${art ? Object.keys(art).slice(0, 8).join(', ') : '(null)'}`);
}

// ─────────────────────────────────────────────────────────
// C. measure-discipline.mjs
// ─────────────────────────────────────────────────────────
{
  const r = run(`node measure-discipline.mjs --root ${FX} --output ${OUT}`);
  assert('C1. measure-discipline completes without error',
    r.ok, r.err.slice(0, 400));

  const art = loadJson('discipline.json');
  assert('C2. discipline.json exists and parses as JSON',
    art !== null,
    `artifact missing or unparseable at ${OUT}/discipline.json`);

  const hasShape = art && typeof art === 'object' && (
    'summary' in art || 'files' in art || 'metrics' in art || 'meta' in art
  );
  assert('C3. discipline.json has recognizable top-level shape',
    hasShape,
    `keys: ${art ? Object.keys(art).slice(0, 8).join(', ') : '(null)'}`);
}

// ─────────────────────────────────────────────────────────
// D. emit-sarif.mjs — runs with zero upstream artifacts (all optional)
// ─────────────────────────────────────────────────────────
{
  // Fresh output dir so previous tests' artifacts don't leak in and
  // change the SARIF output. emit-sarif calls `loadIfExists` for every
  // upstream artifact so an empty dir is a valid, minimal input.
  const OUT2 = '/tmp/out-smoke-sarif';
  rmSync(OUT2, { recursive: true, force: true });
  mkdirSync(OUT2, { recursive: true });

  const r = run(`node emit-sarif.mjs --root ${FX} --output ${OUT2}`);
  assert('D1. emit-sarif completes without error (zero upstream artifacts)',
    r.ok, r.err.slice(0, 400));

  const p = path.join(OUT2, 'lumin-repo-lens-lab.sarif');
  const exists = existsSync(p);
  assert('D2. lumin-repo-lens-lab.sarif exists',
    exists,
    `artifact missing at ${p}`);

  let sarif = null;
  try { sarif = JSON.parse(readFileSync(p, 'utf8')); }
  catch { /* D3 asserts sarif !== null; the empty catch just converts a read/parse failure into a test failure rather than an uncaught exception. */ }
  assert('D3. SARIF file parses as JSON', sarif !== null, '');

  // SARIF 2.1.0 requires `version` and `runs`; the tool entry must
  // carry our TOOL_VERSION (which drift guard keeps in sync).
  assert('D4. SARIF has version=2.1.0',
    sarif?.version === '2.1.0',
    `got version=${sarif?.version}`);
  assert('D5. SARIF has at least one run',
    Array.isArray(sarif?.runs) && sarif.runs.length > 0,
    `runs=${JSON.stringify(sarif?.runs?.length)}`);

  // Drift guard verifies TOOL_VERSION matches package.json. Here we
  // verify SARIF *uses* it — closes the loop.
  const toolName = sarif?.runs?.[0]?.tool?.driver?.name;
  const toolVer  = sarif?.runs?.[0]?.tool?.driver?.version;
  assert('D6. SARIF tool.driver has name + version',
    Boolean(toolName && toolVer),
    `name=${toolName} version=${toolVer}`);
}

// ─────────────────────────────────────────────────────────
// D'. emit-sarif respects classifier policy filters (v1.8.1 regression)
// ─────────────────────────────────────────────────────────
{
  // Build a fixture with ONE config file (eslint.config.mjs) that the
  // classifier would exclude via FP-22, plus one real dead export.
  // Pre-1.8.1 SARIF showed both (read raw symbols.json); post-1.8.1 it
  // prefers dead-classify.json and shows only the real one.
  const FX2 = '/tmp/fx-sarif-policy';
  const OUT4 = '/tmp/out-sarif-policy';
  rmSync(FX2, { recursive: true, force: true });
  rmSync(OUT4, { recursive: true, force: true });
  mkdirSync(path.join(FX2, 'src'), { recursive: true });
  writeFileSync(path.join(FX2, 'package.json'),
    '{"name":"sarif-policy","type":"module"}');
  writeFileSync(path.join(FX2, 'eslint.config.mjs'),
    'export default [{ rules: {} }];\n');
  writeFileSync(path.join(FX2, 'src/realDead.ts'),
    'export const genuinelyUnused = 42;\n');
  writeFileSync(path.join(FX2, 'src/consumer.ts'),
    'export const c = 1;\n');

  run(`node build-symbol-graph.mjs --root ${FX2} --output ${OUT4}`);
  run(`node classify-dead-exports.mjs --root ${FX2} --output ${OUT4}`);
  const r = run(`node emit-sarif.mjs --root ${FX2} --output ${OUT4}`);
  assert("D'1. full pipeline exits 0", r.ok, r.err.slice(0, 400));

  const sarif = JSON.parse(readFileSync(path.join(OUT4, 'lumin-repo-lens-lab.sarif'), 'utf8'));
  const ga001 = sarif.runs[0].results.filter((r) => r.ruleId === 'GA001');
  const symbols = ga001.map((r) => r.properties?.symbol).filter(Boolean);

  assert("D'2. SARIF emits the real dead export",
    symbols.includes('genuinelyUnused'),
    `expected 'genuinelyUnused' in ${JSON.stringify(symbols)}`);

  assert("D'3. SARIF does NOT emit the eslint.config default (FP-22 excluded)",
    !symbols.includes('default'),
    `SARIF leaked config-file default: ${JSON.stringify(symbols)}`);

  assert("D'4. SARIF sources list includes dead-classify.json",
    JSON.stringify(sarif).includes('dead-classify.json') ||
    // Either direct mention or via the tool.driver.run property
    ga001.some((r) => r.properties?.proposalBucket),
    `no evidence of classifier integration in SARIF`);
}

// ─────────────────────────────────────────────────────────
// D''. Explicit failure recording (v1.8.2)
// ─────────────────────────────────────────────────────────
{
  // Build a fixture where build-symbol-graph hits a parse error. The
  // artifact should carry a structured warning; downstream SARIF should
  // surface it under upstreamWarnings.
  const FX3 = '/tmp/fx-warnings';
  const OUT5 = '/tmp/out-warnings';
  rmSync(FX3, { recursive: true, force: true });
  rmSync(OUT5, { recursive: true, force: true });
  mkdirSync(path.join(FX3, 'src'), { recursive: true });
  writeFileSync(path.join(FX3, 'package.json'),
    '{"name":"warnings-fx","type":"module"}');
  writeFileSync(path.join(FX3, 'src/ok.ts'), 'export const ok = 1;\n');
  // Genuinely malformed TS — oxc returns errors[] which we now escalate.
  writeFileSync(path.join(FX3, 'src/broken.ts'), 'export const = ;\n');

  run(`node build-symbol-graph.mjs --root ${FX3} --output ${OUT5}`);
  const syms = JSON.parse(readFileSync(path.join(OUT5, 'symbols.json'), 'utf8'));

  assert('D"1. symbols.json.meta.warnings is an array',
    Array.isArray(syms.meta?.warnings),
    `got ${typeof syms.meta?.warnings}`);

  assert('D"2. parse error surfaces as structured warning',
    syms.meta.warnings.some((w) => w.code === 'parse-errors' && w.count >= 1),
    `warnings: ${JSON.stringify(syms.meta.warnings)}`);

  // Clean repo (ok.ts only) should have empty warnings
  const FX4 = '/tmp/fx-clean';
  const OUT6 = '/tmp/out-clean';
  rmSync(FX4, { recursive: true, force: true });
  rmSync(OUT6, { recursive: true, force: true });
  mkdirSync(path.join(FX4, 'src'), { recursive: true });
  writeFileSync(path.join(FX4, 'package.json'),
    '{"name":"clean-fx","type":"module"}');
  writeFileSync(path.join(FX4, 'src/ok.ts'), 'export const ok = 1;\n');

  run(`node build-symbol-graph.mjs --root ${FX4} --output ${OUT6}`);
  const cleanSyms = JSON.parse(readFileSync(path.join(OUT6, 'symbols.json'), 'utf8'));
  assert('D"3. clean scan produces empty warnings[]',
    Array.isArray(cleanSyms.meta.warnings) && cleanSyms.meta.warnings.length === 0,
    `warnings: ${JSON.stringify(cleanSyms.meta.warnings)}`);

  // SARIF propagation
  run(`node classify-dead-exports.mjs --root ${FX3} --output ${OUT5}`);
  run(`node emit-sarif.mjs --root ${FX3} --output ${OUT5}`);
  const sarif = JSON.parse(readFileSync(path.join(OUT5, 'lumin-repo-lens-lab.sarif'), 'utf8'));
  const up = sarif.runs[0].properties?.upstreamWarnings ?? [];

  assert('D"4. SARIF surfaces upstream warnings from symbols.json',
    up.some((w) => w.source === 'symbols.json' && w.code === 'parse-errors'),
    `upstreamWarnings: ${JSON.stringify(up)}`);
}

// ─────────────────────────────────────────────────────────
// F. check-drift catches package-lock.json version mismatch (v1.8.3)
// ─────────────────────────────────────────────────────────
{
  // Real release drift caught in dogfood: package-lock.json stayed at
  // 1.4.0 while package.json moved to 1.8.2. Drift guard didn't notice
  // because it only checked package.json / emit-sarif / CHANGELOG.
  //
  // This test asserts:
  //   (1) running drift-check with a matching lockfile exits 0;
  //   (1b) prerelease versions such as 0.9.0-beta.1 are compared whole;
  //   (2) running with a deliberately corrupted lockfile exits 1 AND
  //       reports the lockfile as the offender.
  //
  // We drive scripts/check-drift.mjs against a synthetic repo whose
  // package.json / Rust SARIF owner / CHANGELOG / package-lock all match or
  // don't match as specified.
  const FXroot = '/tmp/fx-check-drift-lock';
  rmSync(FXroot, { recursive: true, force: true });
  mkdirSync(path.join(FXroot, 'scripts'), { recursive: true });
  mkdirSync(path.join(FXroot, 'experiments/rust-main/lumin-audit-core/src'), { recursive: true });
  const writeFixture = (lockVersion, version = '9.9.9') => {
    writeFileSync(path.join(FXroot, 'package.json'),
      JSON.stringify({ name: 'x', version, type: 'module' }));
    writeFileSync(path.join(FXroot, 'experiments/rust-main/lumin-audit-core/src/sarif.rs'),
      `const TOOL_VERSION: &str = "${version}";\n`);
    writeFileSync(path.join(FXroot, 'CHANGELOG.md'),
      `# Changelog\n\n## ${version} — 2026-04-19\n\nSynthetic.\n`);
    writeFileSync(path.join(FXroot, 'package-lock.json'),
      JSON.stringify({
        name: 'x',
        version: lockVersion,
        lockfileVersion: 3,
        packages: { '': { name: 'x', version: lockVersion } },
      }, null, 2));
  };

  // Copy the real scripts/check-drift.mjs into the fixture so the
  // check runs there without touching the real repo.
  const driftSrc = readFileSync(
    path.join(DIR, 'scripts/check-drift.mjs'), 'utf8');
  writeFileSync(path.join(FXroot, 'scripts/check-drift.mjs'), driftSrc);

  // (1) Matching lockfile → exit 0
  writeFixture('9.9.9');
  const rMatch = run(`node scripts/check-drift.mjs`, FXroot);
  assert('F1. check-drift exits 0 when lockfile matches package.json',
    rMatch.ok,
    `stdout: ${rMatch.out.slice(0, 300)} | stderr: ${rMatch.err.slice(0, 300)}`);

  // (1b) Matching prerelease version → exit 0, with the full prerelease
  // suffix included in the CHANGELOG comparison.
  writeFixture('0.9.0-beta.1', '0.9.0-beta.1');
  const rPrerelease = run(`node scripts/check-drift.mjs`, FXroot);
  assert('F1b. check-drift accepts matching prerelease semver',
    rPrerelease.ok,
    `stdout: ${rPrerelease.out.slice(0, 300)} | stderr: ${rPrerelease.err.slice(0, 300)}`);

  // (2) Mismatched lockfile → exit non-zero, report mentions lockfile
  writeFixture('1.0.0');
  const rMismatch = run(`node scripts/check-drift.mjs`, FXroot);
  assert('F2. check-drift exits non-zero when lockfile drifted',
    !rMismatch.ok,
    `expected failure, got exit 0`);
  assert('F3. drift report names package-lock.json',
    rMismatch.err.includes('package-lock.json'),
    `stderr: ${rMismatch.err.slice(0, 400)}`);
}

// ─────────────────────────────────────────────────────────
// E. merge-runtime-evidence.mjs — requires symbols.json + coverage-final.json
// ─────────────────────────────────────────────────────────
{
  const OUT3 = '/tmp/out-smoke-merge';
  rmSync(OUT3, { recursive: true, force: true });
  mkdirSync(OUT3, { recursive: true });

  // Minimal symbols.json matching the shape build-symbol-graph emits.
  // Just enough for the merge to have something to enrich.
  const helperFile = path.join(FX, 'src/lib.ts');
  const appFile    = path.join(FX, 'src/app.ts');
  writeFileSync(path.join(OUT3, 'symbols.json'), JSON.stringify({
    meta: { generated: new Date().toISOString(), root: FX },
    symbolsByFile: {
      [helperFile]: [
        { name: 'helper', kind: 'FunctionDeclaration', line: 1, exported: true },
        { name: 'unused', kind: 'FunctionDeclaration', line: 4, exported: true },
      ],
      [appFile]: [
        { name: 'main', kind: 'FunctionDeclaration', line: 2, exported: true },
      ],
    },
    deadProdList: [
      { file: helperFile, symbol: 'unused', kind: 'FunctionDeclaration', line: 4 },
    ],
  }, null, 2));

  // Minimal Istanbul-shape coverage-final.json — one file hit, one cold.
  writeFileSync(path.join(OUT3, 'coverage-final.json'), JSON.stringify({
    [helperFile]: {
      path: helperFile,
      fnMap: {
        '0': { name: 'helper', decl: { start: { line: 1 }, end: { line: 1 } },
               loc: { start: { line: 1 }, end: { line: 3 } }, line: 1 },
        '1': { name: 'unused', decl: { start: { line: 4 }, end: { line: 4 } },
               loc: { start: { line: 4 }, end: { line: 4 } }, line: 4 },
      },
      f: { '0': 1, '1': 0 },
      statementMap: {}, s: {}, branchMap: {}, b: {},
    },
  }, null, 2));

  const r = run(`node merge-runtime-evidence.mjs --root ${FX} --output ${OUT3} --coverage ${OUT3}/coverage-final.json`);
  assert('E1. merge-runtime-evidence completes without error',
    r.ok, r.err.slice(0, 600));

  const evidence = (() => {
    try { return JSON.parse(readFileSync(path.join(OUT3, 'runtime-evidence.json'), 'utf8')); }
    catch { return null; }
  })();
  assert('E2. runtime-evidence.json exists and parses',
    evidence !== null,
    `artifact missing or unparseable at ${OUT3}/runtime-evidence.json`);

  const hasShape = evidence && typeof evidence === 'object' && (
    'meta' in evidence || 'touched' in evidence || 'summary' in evidence || 'perSymbol' in evidence
  );
  assert('E3. runtime-evidence.json has recognizable shape',
    hasShape,
    `keys: ${evidence ? Object.keys(evidence).slice(0, 8).join(', ') : '(null)'}`);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
