// Tests for Issue 5: hardcoded package names removed, method-calls focus-class parameterized.
import { execSync } from 'node:child_process';
import { writeFileSync, mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const DIR = path.resolve(__dirname, '..');
const FX = '/tmp/fx-monorepo';
const OUT = '/tmp/mono';

// ── Build hermetic monorepo fixture ──
rmSync(FX, { recursive: true, force: true });
mkdirSync(path.join(FX, 'packages/alpha/src'), { recursive: true });
mkdirSync(path.join(FX, 'apps/beta/src'), { recursive: true });
writeFileSync(path.join(FX, 'package.json'), JSON.stringify({
  name: 'monorepo-root',
  type: 'module',
  workspaces: ['packages/*', 'apps/*'],
}));
writeFileSync(path.join(FX, 'packages/alpha/package.json'),
  '{"name":"@mono/alpha","type":"module","exports":{".":"./src/index.ts"}}');
writeFileSync(path.join(FX, 'packages/alpha/src/helpers.ts'),
  'export const alphaHelper = 1;\nexport const alphaDeadSymbol = "never-used";\n');
writeFileSync(path.join(FX, 'packages/alpha/src/index.ts'),
  "import { alphaHelper } from './helpers';\nexport const used = alphaHelper;\n");
writeFileSync(path.join(FX, 'apps/beta/package.json'),
  '{"name":"@mono/beta","type":"module","exports":{".":"./src/index.ts"}}');
writeFileSync(path.join(FX, 'apps/beta/src/utils.ts'),
  'export const betaUtil = 1;\nexport const betaUnused1 = "x";\nexport const betaUnused2 = "y";\n');
writeFileSync(path.join(FX, 'apps/beta/src/index.ts'),
  "import { used } from '@mono/alpha';\nimport { betaUtil } from './utils';\n" +
  'export const beta = used + betaUtil;\n');

let passed = 0, failed = 0;
function assert(label, ok, detail) {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function run(cmd) {
  try {
    return execSync(cmd, { cwd: DIR, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  } catch (e) {
    return (e.stdout || '') + (e.stderr || '');
  }
}

// Pre-build symbol graph
rmSync(OUT, { recursive: true, force: true });
run(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`);

// ── T1: classify must derive package labels from actual workspaces ─
{
  const out = run(`node classify-dead-exports.mjs --root ${FX} --output ${OUT}`);
  // Expect: at least one label that matches workspace dir (not generic 'other')
  // Workspaces: packages/alpha, apps/beta
  const hasAlpha = /packages\/alpha|alpha/.test(
    out.split('package별 × category')[1]?.split('C (완전 dead)')[0] || '',
  );
  const hasBeta = /apps\/beta|beta/.test(
    out.split('package별 × category')[1]?.split('C (완전 dead)')[0] || '',
  );
  assert(
    'T1. classify labels include workspace "alpha"',
    hasAlpha,
    `did not see alpha label:\n---\n${out.split('package별')[1]?.slice(0, 400)}`,
  );
  assert(
    'T2. classify labels include workspace "beta"',
    hasBeta,
    `did not see beta label:\n---\n${out.split('package별')[1]?.slice(0, 400)}`,
  );
  // Also: make sure hardcoded "protocol" / "daemon" / "web-shell" / "shared-utils"
  // labels never appear for this fixture (no such dirs in fixture)
  const badLabels = ['protocol', 'daemon', 'web-shell', 'shared-utils'];
  for (const bad of badLabels) {
    const block = out.split('package별 × category')[1]?.split('C (완전 dead)')[0] || '';
    assert(
      `T3.${bad}. classify does NOT fabricate "${bad}" label`,
      !new RegExp(`\\b${bad}\\b`).test(block),
      `found "${bad}" in:\n---\n${block}`,
    );
  }
}

// ── T4: method-calls without --focus-class must NOT print RunChannelClient block ─
{
  const out = run(`node resolve-method-calls.mjs --root ${FX} --output ${OUT}`);
  assert(
    'T4. resolve-method-calls has no RunChannelClient block without --focus-class',
    !out.includes('RunChannelClient'),
    `unexpected RunChannelClient mention:\n---\n${out.slice(-600)}`,
  );
}

// ── T5: --focus-class <name> prints a block for that name ─
{
  const out = run(`node resolve-method-calls.mjs --root ${FX} --output ${OUT} --focus-class MyClass`);
  assert(
    'T5. --focus-class MyClass prints a MyClass-specific block',
    /MyClass method 사용 실태|MyClass\s+method/.test(out),
    `did not find MyClass block:\n---\n${out.slice(-600)}`,
  );
  assert(
    'T6. --focus-class MyClass does NOT print RunChannelClient block',
    !out.includes('RunChannelClient'),
    `unexpected RunChannelClient:\n---\n${out.slice(-600)}`,
  );
}

// ── T7/T8: v1.3.0 — focusClassReport is emitted as structured JSON ─
{
  const { readFileSync } = await import('node:fs');
  const art = JSON.parse(readFileSync(`${OUT}/level2-methods.json`, 'utf8'));
  assert(
    'T7. level2-methods.json.focusClassReport carries className when flag set',
    art.focusClassReport && art.focusClassReport.className === 'MyClass',
    `got focusClassReport=${JSON.stringify(art.focusClassReport)}`,
  );
  // Re-run without the flag — field should be null
  run(`node resolve-method-calls.mjs --root ${FX} --output ${OUT}`);
  const art2 = JSON.parse(readFileSync(`${OUT}/level2-methods.json`, 'utf8'));
  assert(
    'T8. focusClassReport is null when --focus-class omitted',
    art2.focusClassReport === null,
    `got focusClassReport=${JSON.stringify(art2.focusClassReport)}`,
  );
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
