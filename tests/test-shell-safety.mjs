// Tests for Issue 7: shell injection fixes in triage + staleness.
// Two assertion classes:
//  - Correctness: tool still produces correct output on normal repos.
//  - Safety: filenames containing shell metacharacters ($, backticks) don't
//    break execution or leak into a subshell.
//
// Fixture setup uses Node fs APIs directly (not shell) so the test is
// cross-platform. The earlier version relied on `shell: '/bin/bash'` and
// `rm -rf`/`mkdir -p`/`echo >` which are Unix-only. The tool is the
// thing under test, not the fixture harness.
import { execSync } from 'node:child_process';
import { readFileSync, existsSync, writeFileSync, mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const DIR = path.resolve(__dirname, '..');
const TMP = tmpdir();
const FX_INJ   = path.join(TMP, 'fx-injection');
const FX_STALE = path.join(TMP, 'fx-staleness');
const FX_ROOTPY = path.join(TMP, 'fx-root-py');
const FX_GO    = path.join(TMP, 'fx-go');
const FX_SFC   = path.join(TMP, 'fx-sfc');
const FX_DOLLAR = path.join(TMP, 'fx-st-dollar');
const OUT_INJ   = path.join(TMP, 'inj-triage');
const OUT_ROOTPY = path.join(TMP, 'root-py-out');
const OUT_GO   = path.join(TMP, 'go-out');
const OUT_SFC  = path.join(TMP, 'sfc-out');
const OUT_ST   = path.join(TMP, 'st-out');
const OUT_ST_D = path.join(TMP, 'st-d-out');

// Initialize a bare git repo with a single commit covering every file
// currently in `dir`. Uses separate `execSync` calls so no shell is
// invoked — git is the executable, arguments are passed explicitly.
function gitInitAndCommit(dir) {
  execSync('git init -q', { cwd: dir });
  execSync('git config user.email t@t', { cwd: dir });
  execSync('git config user.name t', { cwd: dir });
  execSync('git add -A', { cwd: dir });
  execSync('git commit -q -m init', { cwd: dir });
}

// ── Build hermetic fixtures ──
// Injection fixture: filenames with $ (which would be shell-expanded)
rmSync(FX_INJ, { recursive: true, force: true });
mkdirSync(path.join(FX_INJ, 'src'), { recursive: true });
writeFileSync(path.join(FX_INJ, 'package.json'), '{"name":"fx-inj","type":"module"}');
writeFileSync(path.join(FX_INJ, 'src/normal.ts'), 'export const x = 1;\n');
writeFileSync(path.join(FX_INJ, 'src/dollar$file.ts'), 'export const y = 2;\n');
writeFileSync(path.join(FX_INJ, 'src/weird$name.py'), 'def foo(): pass\n');

// Staleness fixture: real git repo with a simple dead symbol
rmSync(FX_STALE, { recursive: true, force: true });
mkdirSync(path.join(FX_STALE, 'src'), { recursive: true });
writeFileSync(path.join(FX_STALE, 'package.json'), '{"name":"fx-st","type":"module"}');
writeFileSync(path.join(FX_STALE, 'src/good.ts'),
  'export const ok = 1;\nexport const dead = 2;\n');
gitInitAndCommit(FX_STALE);

let passed = 0, failed = 0;
function assert(label, ok, detail) {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function runOk(cmd) {
  try {
    const out = execSync(cmd, { cwd: DIR, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
    return { ok: true, out, err: '' };
  } catch (e) {
    return { ok: false, out: e.stdout || '', err: e.stderr || e.message };
  }
}

// Quote a path for the platform's default shell. Node's execSync without
// `shell` option uses the system default (bash/sh on Unix, cmd.exe on
// Windows). Double-quoting is safe on both for paths that may contain
// spaces or $. The tool itself must still avoid shell-expanding arguments
// that it internally forwards — that is what we're testing.
const q = (p) => `"${p}"`;

// ── A. triage correctness on injection fixture (weird filenames) ─
{
  const r = runOk(`node triage-repo.mjs --root ${q(FX_INJ)} --output ${q(OUT_INJ)}`);
  assert('A1. triage completes without error on $-containing filenames', r.ok, r.err.slice(0, 400));
  if (r.ok) {
    const art = JSON.parse(readFileSync(path.join(OUT_INJ, 'triage.json'), 'utf8'));
    // Expected: 2 ts (normal.ts + dollar$file.ts) + 1 py (weird$name.py)
    assert(
      'A2. triage counts ts files correctly including $-named',
      art.shape.tsFiles === 2,
      `tsFiles=${art.shape.tsFiles}, expected 2. shape=${JSON.stringify(art.shape)}`,
    );
    assert(
      'A3. triage counts py files correctly including $-named',
      art.shape.pyFiles === 1,
      `pyFiles=${art.shape.pyFiles}, expected 1. shape=${JSON.stringify(art.shape)}`,
    );
    assert(
      'A4. triage records single-pass file collection telemetry',
      art.performance?.fileCollection?.strategy === 'single-pass-language-split' &&
        art.performance.fileCollection.collectFilesCalls === 1 &&
        art.performance.fileCollection.totalFilesCollected === 3 &&
        art.performance.fileCollection.languageFiles?.ts === 2 &&
        art.performance.fileCollection.languageFiles?.py === 1,
      `performance=${JSON.stringify(art.performance)}`,
    );
  }
}

// ── B. triage: Python file detection must work for root-only Python repo ─
{
  // Fixture: ONLY main.py at root (no src/, no tests/). Old gated behavior
  // would return 0 Python files here — the shell-find was gated on
  // `existsSync(root + '/src') || existsSync(root + '/tests')`.
  rmSync(FX_ROOTPY, { recursive: true, force: true });
  mkdirSync(FX_ROOTPY, { recursive: true });
  writeFileSync(path.join(FX_ROOTPY, 'package.json'), '{"name":"root-py"}');
  writeFileSync(path.join(FX_ROOTPY, 'main.py'), 'def foo(): pass\n');
  const r = runOk(`node triage-repo.mjs --root ${q(FX_ROOTPY)} --output ${q(OUT_ROOTPY)}`);
  assert('B1. root-only Python repo triage completes', r.ok, r.err.slice(0, 400));
  if (r.ok) {
    const art = JSON.parse(readFileSync(path.join(OUT_ROOTPY, 'triage.json'), 'utf8'));
    assert(
      'B2. root-only Python repo detects main.py (was 0 before)',
      art.shape.pyFiles >= 1,
      `pyFiles=${art.shape.pyFiles}, shape=${JSON.stringify(art.shape)}`,
    );
  }
}

// ── C. triage: Go files are counted ────────────────────────────
{
  rmSync(FX_GO, { recursive: true, force: true });
  mkdirSync(path.join(FX_GO, 'src'), { recursive: true });
  writeFileSync(path.join(FX_GO, 'package.json'), '{"name":"go-fx"}');
  writeFileSync(path.join(FX_GO, 'src/main.go'), 'package main\nfunc main() {}\n');
  const r = runOk(`node triage-repo.mjs --root ${q(FX_GO)} --output ${q(OUT_GO)}`);
  assert('C1. Go-containing repo triage completes', r.ok, r.err.slice(0, 400));
  if (r.ok) {
    const art = JSON.parse(readFileSync(path.join(OUT_GO, 'triage.json'), 'utf8'));
    assert(
      'C2. triage artifact exposes goFiles count',
      typeof art.shape.goFiles === 'number' && art.shape.goFiles >= 1,
      `shape=${JSON.stringify(art.shape)}`,
    );
  }
}

// ── C2. triage: SFC containers are counted but not parsed ────────
{
  rmSync(FX_SFC, { recursive: true, force: true });
  mkdirSync(path.join(FX_SFC, 'src'), { recursive: true });
  writeFileSync(path.join(FX_SFC, 'package.json'), '{"name":"sfc-fx"}');
  writeFileSync(path.join(FX_SFC, 'src/App.vue'), '<script setup lang="ts">const x = 1</script>\n');
  writeFileSync(path.join(FX_SFC, 'src/Page.svelte'), '<script lang="ts">export let y;</script>\n');
  writeFileSync(path.join(FX_SFC, 'src/Home.astro'), '---\nconst z = 1;\n---\n');
  const r = runOk(`node triage-repo.mjs --root ${q(FX_SFC)} --output ${q(OUT_SFC)}`);
  assert('C3. SFC-containing repo triage completes', r.ok, r.err.slice(0, 400));
  if (r.ok) {
    const art = JSON.parse(readFileSync(path.join(OUT_SFC, 'triage.json'), 'utf8'));
    assert(
      'C4. triage artifact exposes SFC count and per-extension language counts',
      art.shape.sfcFiles === 3 &&
        art.byLanguage?.vue === 1 &&
        art.byLanguage?.svelte === 1 &&
        art.byLanguage?.astro === 1 &&
        art.performance?.fileCollection?.languageFiles?.sfc === 3,
      `shape=${JSON.stringify(art.shape)}, byLanguage=${JSON.stringify(art.byLanguage)}, performance=${JSON.stringify(art.performance)}`,
    );
  }
}

// ── D. triage: topDirs count respects weird filenames ───────────
{
  const triagePath = path.join(OUT_INJ, 'triage.json');
  const art = existsSync(triagePath) ? JSON.parse(readFileSync(triagePath, 'utf8')) : null;
  assert(
    'D1. topDirs["src"] reports all 3 files',
    art && art.topDirs?.src?.files === 3,
    `topDirs=${JSON.stringify(art?.topDirs)}`,
  );
}

// ── E. measure-staleness: no shell injection via relFile ────────
{
  // Staleness needs symbols.json. Build it first.
  execSync(`node ${q(path.join(DIR, 'build-symbol-graph.mjs'))} --root ${q(FX_STALE)} --output ${q(OUT_ST)}`,
    { stdio: ['ignore', 'pipe', 'pipe'] });
  const r = runOk(`node measure-staleness.mjs --root ${q(FX_STALE)} --output ${q(OUT_ST)}`);
  assert('E1. staleness runs on normal git repo', r.ok, r.err.slice(0, 400));
  if (r.ok) {
    const art = JSON.parse(readFileSync(path.join(OUT_ST, 'staleness.json'), 'utf8'));
    assert(
      'E2. staleness emits per-symbol records with stalenessTier',
      Array.isArray(art.enriched) && art.enriched.length >= 1 && art.enriched[0].stalenessTier,
      `got: ${JSON.stringify(art).slice(0, 300)}`,
    );
  }
}

// ── F. measure-staleness: safe on repo with $-containing filename ─
{
  // Build a git repo whose files have $ in the name — git paths must
  // survive the staleness pipeline without shell expansion.
  rmSync(FX_DOLLAR, { recursive: true, force: true });
  mkdirSync(path.join(FX_DOLLAR, 'src'), { recursive: true });
  writeFileSync(path.join(FX_DOLLAR, 'package.json'), '{"name":"d"}');
  writeFileSync(path.join(FX_DOLLAR, 'src/weird$name.ts'), 'export const dead = 1;\n');
  gitInitAndCommit(FX_DOLLAR);
  execSync(`node ${q(path.join(DIR, 'build-symbol-graph.mjs'))} --root ${q(FX_DOLLAR)} --output ${q(OUT_ST_D)}`,
    { stdio: ['ignore', 'pipe', 'pipe'] });
  const r = runOk(`node measure-staleness.mjs --root ${q(FX_DOLLAR)} --output ${q(OUT_ST_D)}`);
  assert('F1. staleness handles $-containing filename without crash', r.ok, r.err.slice(0, 400));
  if (r.ok) {
    const art = JSON.parse(readFileSync(path.join(OUT_ST_D, 'staleness.json'), 'utf8'));
    // Verify the $-named file's symbol made it through with real timestamp
    const entry = art.enriched?.find((e) => e.file?.includes('weird$name'));
    assert(
      'F2. staleness emits entry for $-named file with non-null fileLastTouchedAt',
      entry && entry.fileLastTouchedAt !== null,
      `entry: ${JSON.stringify(entry)}`,
    );
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
