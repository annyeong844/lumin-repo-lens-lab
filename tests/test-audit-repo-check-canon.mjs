// tests/test-audit-repo-check-canon.mjs
//
// P5-4 Step 0 — RED test for `audit-repo.mjs --check-canon` orchestrator.
//
// Pins the thin-spawn-wrapper contract + manifest.checkCanon shape +
// advisory vs --strict-check-canon exit code matrix + child-exit-1/2-are-
// legitimate-outcomes rule + mutex preservation + independence from
// --canon-draft.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const AUDIT_CLI = path.join(DIR, 'audit-repo.mjs');

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

function buildFixture(fx) {
  write(fx, 'package.json', JSON.stringify({ name: 'ar-cc-fx', type: 'module' }));
  write(fx, '_lib/util.mjs', `export function helper() { return 1 }\n`);
  write(fx, 'src/app.mjs',
    `import { helper } from '../_lib/util.mjs';\n` +
    `export const x = helper();\n`);
}

function readManifest(out) {
  return JSON.parse(readFileSync(path.join(out, 'manifest.json'), 'utf8'));
}

const cleanup = [];

function mkFixture() {
  const fx = mkdtempSync(path.join(tmpdir(), 'arcc-fx-'));
  const out = mkdtempSync(path.join(tmpdir(), 'arcc-out-'));
  buildFixture(fx);
  cleanup.push(fx, out);
  return { fx, out };
}

// ═══ CK-1: --check-canon invocation populates manifest.checkCanon ═══

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon',
  ], { encoding: 'utf8' });
  assert('CK-1a. --check-canon alone → exit 0 (advisory, no canon → all skipped, but ran=true)',
    res.status === 0, `exit=${res.status}, stderr=${(res.stderr ?? '').slice(0, 400)}`);
  const m = readManifest(out);
  assert('CK-1b. manifest.checkCanon.requested === true',
    m.checkCanon?.requested === true, `block=${JSON.stringify(m.checkCanon)}`);
  assert('CK-1c. manifest.checkCanon has all 4 perSource keys',
    m.checkCanon?.perSource &&
    ['type-ownership', 'helper-registry', 'topology', 'naming']
      .every((s) => s in m.checkCanon.perSource),
    `perSource keys=${Object.keys(m.checkCanon?.perSource ?? {})}`);
  assert('CK-1d. summary.sourcesRequested === 4',
    m.checkCanon?.summary?.sourcesRequested === 4,
    `summary=${JSON.stringify(m.checkCanon?.summary)}`);
  assert('CK-1e. summary has sourcesChecked / sourcesSkipped / sourcesFailed / driftCount fields',
    typeof m.checkCanon?.summary?.sourcesChecked === 'number' &&
    typeof m.checkCanon?.summary?.sourcesSkipped === 'number' &&
    typeof m.checkCanon?.summary?.sourcesFailed === 'number' &&
    typeof m.checkCanon?.summary?.driftCount === 'number', '');
  assert('CK-1f. driftCounts has all 4 keys',
    m.checkCanon?.driftCounts &&
    ['type-ownership', 'helper-registry', 'topology', 'naming']
      .every((s) => typeof m.checkCanon.driftCounts[s] === 'number'),
    `driftCounts=${JSON.stringify(m.checkCanon?.driftCounts)}`);
  const summaryMd = readFileSync(path.join(out, 'audit-summary.latest.md'), 'utf8');
  assert('CK-1g. first-read summary surfaces check-canon command result',
    summaryMd.includes('## Command Result') &&
    summaryMd.includes('Check-canon could not compare promoted canon yet'),
    summaryMd);
  assert('CK-1h. console preview surfaces check-canon result before JSON spelunking',
    res.stdout.includes('Command Result') &&
    res.stdout.includes('Check-canon could not compare promoted canon yet'),
    res.stdout.slice(-1000));
}

// ═══ CK-2: Unknown --sources value → ran=false, exit 1 ═══

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon', '--sources', 'not-a-real-source',
  ], { encoding: 'utf8' });
  assert('CK-2a. unknown --sources value → orchestrator exit 1',
    res.status === 1, `exit=${res.status}`);
  const m = readManifest(out);
  assert('CK-2b. manifest.checkCanon.ran === false',
    m.checkCanon?.ran === false, `ran=${m.checkCanon?.ran}`);
  assert('CK-2c. manifest.checkCanon.reason mentions unknown sources',
    typeof m.checkCanon?.reason === 'string' && /unknown/i.test(m.checkCanon.reason),
    `reason=${m.checkCanon?.reason}`);
}

// ═══ CK-3: --sources subset + exit 1 confined to unknown-source gate ═══

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon', '--sources', 'naming',
  ], { encoding: 'utf8' });
  assert('CK-3a. --sources naming valid value → exit 0 advisory',
    res.status === 0, `exit=${res.status}, stderr=${(res.stderr ?? '').slice(0, 400)}`);
  const m = readManifest(out);
  assert('CK-3b. requestedSources reflects subset',
    Array.isArray(m.checkCanon?.requestedSources) &&
    m.checkCanon.requestedSources.length === 1 &&
    m.checkCanon.requestedSources[0] === 'naming', '');
  assert('CK-3c. perSource has only the requested source',
    m.checkCanon?.perSource &&
    Object.keys(m.checkCanon.perSource).length === 1 &&
    'naming' in m.checkCanon.perSource, '');
}

// ═══ CK-4: child exit 2 (skipped-missing-canon) is a per-source result, not spawn failure ═══

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon', '--source', 'naming',
  ], { encoding: 'utf8' });
  assert('CK-3d. --source alias scopes check-canon',
    res.status === 0, `exit=${res.status}, stderr=${(res.stderr ?? '').slice(0, 400)}`);
  const m = readManifest(out);
  assert('CK-3e. requestedSources reflects --source alias',
    Array.isArray(m.checkCanon?.requestedSources) &&
    m.checkCanon.requestedSources.length === 1 &&
    m.checkCanon.requestedSources[0] === 'naming',
    JSON.stringify(m.checkCanon?.requestedSources));
}

{
  const { fx, out } = mkFixture();
  // No canonical/ dir → every source is skipped-missing-canon
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon',
  ], { encoding: 'utf8' });
  assert('CK-4a. all sources missing canon → orchestrator advisory exit 0',
    res.status === 0, `exit=${res.status}`);
  const m = readManifest(out);
  assert('CK-4b. every perSource entry has ran=true (structured result, not spawn failure)',
    m.checkCanon?.perSource &&
    Object.values(m.checkCanon.perSource).every((e) => e.ran === true),
    `perSource=${JSON.stringify(m.checkCanon?.perSource)}`);
  assert('CK-4c. every perSource.status === skipped-missing-canon',
    Object.values(m.checkCanon?.perSource ?? {}).every((e) => e.status === 'skipped-missing-canon'),
    `statuses=${Object.values(m.checkCanon?.perSource ?? {}).map((e) => e.status).join(',')}`);
  assert('CK-4d. every perSource.exitCode === 2',
    Object.values(m.checkCanon?.perSource ?? {}).every((e) => e.exitCode === 2), '');
  assert('CK-4e. summary.sourcesSkipped === 4',
    m.checkCanon?.summary?.sourcesSkipped === 4,
    `summary=${JSON.stringify(m.checkCanon?.summary)}`);
}

// ═══ CK-5: --strict-check-canon + all missing → exit 2 ═══

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon', '--strict-check-canon',
  ], { encoding: 'utf8' });
  assert('CK-5. --strict-check-canon + all sources missing → exit 2 (no checked source)',
    res.status === 2, `exit=${res.status}, stderr=${(res.stderr ?? '').slice(0, 400)}`);
  const m = readManifest(out);
  assert('CK-5b. manifest.checkCanon.strict === true',
    m.checkCanon?.strict === true, `strict=${m.checkCanon?.strict}`);
}

// ═══ CK-6: mutex — --pre-write + --post-write + --check-canon → exit 2 ═══

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out,
    '--pre-write', '--post-write', '--check-canon',
  ], { encoding: 'utf8' });
  assert('CK-6. pre-write + post-write + check-canon → exit 2 (pre/post mutex preserved)',
    res.status === 2, `exit=${res.status}, stderr=${(res.stderr ?? '').slice(0, 400)}`);
}

// ═══ CK-7: --canon-draft + --check-canon independence ═══

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--canon-draft', '--check-canon',
  ], { encoding: 'utf8' });
  assert('CK-7a. --canon-draft + --check-canon → exit 0 advisory',
    res.status === 0, `exit=${res.status}, stderr=${(res.stderr ?? '').slice(0, 400)}`);
  const m = readManifest(out);
  assert('CK-7b. both manifest blocks present',
    m.canonDraft?.requested === true && m.checkCanon?.requested === true,
    `canonDraft=${!!m.canonDraft}, checkCanon=${!!m.checkCanon}`);
  assert('CK-7c. check-canon sees no promoted canonical/*.md even though canon-draft wrote canonical-draft/*.md',
    m.checkCanon?.summary?.sourcesChecked === 0 &&
    m.checkCanon?.summary?.sourcesSkipped === 4,
    `summary=${JSON.stringify(m.checkCanon?.summary)}`);
}

// ═══ CK-8: canonical/ present → per-source clean/drift populated ═══

{
  const { fx, out } = mkFixture();
  // Add a promoted canonical/type-ownership.md that matches what the
  // orchestrator's pipeline will observe (empty type surface on the fixture).
  write(fx, 'canonical/type-ownership.md',
    '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
    '|------|----------|-------|-------:|--------|------|\n',
  );
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon', '--sources', 'type-ownership',
  ], { encoding: 'utf8' });
  assert('CK-8a. type-ownership with empty canon + empty fresh → exit 0',
    res.status === 0, `exit=${res.status}, stderr=${(res.stderr ?? '').slice(0, 400)}`);
  const m = readManifest(out);
  const entry = m.checkCanon?.perSource?.['type-ownership'];
  assert('CK-8b. perSource["type-ownership"].status = clean (no drift)',
    entry?.status === 'clean' && entry?.exitCode === 0 && entry?.driftCount === 0,
    `entry=${JSON.stringify(entry)}`);
  assert('CK-8c. summary.sourcesChecked === 1',
    m.checkCanon?.summary?.sourcesChecked === 1, '');
}

// ═══ CK-9: manifest.checkCanon.ran semantics ═══
//
// ran === true iff at least one source produced a structured perSource
// result (including skipped-missing-canon — which IS a structured outcome).
// Only a true spawn failure (ENOENT) or unknown-sources gate produces ran=false.

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon',
  ], { encoding: 'utf8' });
  assert('CK-9. ran === true when sources all skipped-missing-canon (structured outcomes still count)',
    res.status === 0 && readManifest(out).checkCanon?.ran === true, '');
}

// ═══ CK-10: --sources all expands to 4 named sources (Finding #1 post-landing) ═══

{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon', '--sources', 'all',
  ], { encoding: 'utf8' });
  assert('CK-10a. --sources all → exit 0 (advisory, all 4 skipped-missing-canon)',
    res.status === 0, `exit=${res.status}, stderr=${(res.stderr ?? '').slice(0, 400)}`);
  const m = readManifest(out);
  assert('CK-10b. manifest.checkCanon.ran === true',
    m.checkCanon?.ran === true, `ran=${m.checkCanon?.ran}`);
  assert('CK-10c. requestedSources expanded to 4 named sources',
    Array.isArray(m.checkCanon?.requestedSources) &&
    m.checkCanon.requestedSources.length === 4 &&
    ['type-ownership', 'helper-registry', 'topology', 'naming']
      .every((s) => m.checkCanon.requestedSources.includes(s)),
    `requestedSources=${JSON.stringify(m.checkCanon?.requestedSources)}`);
  assert('CK-10d. perSource has all 4 entries',
    m.checkCanon?.perSource &&
    Object.keys(m.checkCanon.perSource).length === 4, '');
  // NOT a false "unknown --sources" rejection.
  assert('CK-10e. no "unknown --sources" reason recorded',
    !m.checkCanon?.reason || !/unknown/i.test(m.checkCanon.reason), '');
  assert('CK-10f. all requested sources use single check-canon child invocation',
    m.checkCanon?.executionMode === 'single-invocation-all' &&
    m.checkCanon?.childInvocations === 1,
    `executionMode=${m.checkCanon?.executionMode}, childInvocations=${m.checkCanon?.childInvocations}`);
}

// CK-11: --sources "all,naming" dedupes
{
  const { fx, out } = mkFixture();
  const res = spawnSync(NODE, [AUDIT_CLI,
    '--root', fx, '--output', out, '--check-canon', '--sources', 'all,naming',
  ], { encoding: 'utf8' });
  assert('CK-11a. --sources "all,naming" → still 4 named sources (dedupe)',
    res.status === 0, `exit=${res.status}`);
  const m = readManifest(out);
  assert('CK-11b. requestedSources deduped to 4',
    m.checkCanon?.requestedSources?.length === 4, `length=${m.checkCanon?.requestedSources?.length}`);
  assert('CK-11c. deduped all-source request stays single-invocation',
    m.checkCanon?.executionMode === 'single-invocation-all' &&
    m.checkCanon?.childInvocations === 1,
    `executionMode=${m.checkCanon?.executionMode}, childInvocations=${m.checkCanon?.childInvocations}`);
}

// ═══ CK-12: audit-repo delegates check-canon lifecycle to helper ═══

{
  const src = readFileSync(path.join(DIR, 'audit-repo.mjs'), 'utf8');
  const helper = readFileSync(path.join(DIR, '_lib', 'audit-check-canon.mjs'), 'utf8');
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/\/\/[^\n]*/g, '');
  const strippedHelper = helper
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/\/\/[^\n]*/g, '');

  assert('CK-12a. audit-repo.mjs delegates check-canon lifecycle to helper',
    /runCheckCanonLifecycle/.test(stripped) &&
    /audit-check-canon\.mjs/.test(stripped));
  assert('CK-12b. audit-check-canon.mjs owns CHECK_CANON_SOURCES',
    /CHECK_CANON_SOURCES/.test(strippedHelper) &&
    /type-ownership/.test(strippedHelper) &&
    /helper-registry/.test(strippedHelper) &&
    /topology/.test(strippedHelper) &&
    /naming/.test(strippedHelper));
}

for (const p of cleanup) {
  try { rmSync(p, { recursive: true, force: true }); } catch { /* swallow cleanup errors */ }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
