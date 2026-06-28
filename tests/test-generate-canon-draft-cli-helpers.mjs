// Tests for `generate-canon-draft.mjs --source helper-registry` — P3-2 Step 3.
//
// Pinning rules from docs/history/phases/p3/p3-2.md v2 §5.4:
//   - --source helper-registry accepted (was rejected in P3-1).
//   - --source type-ownership still works (P3-1 regression).
//   - call-graph.json absent → draft emits normally, meta records absence.
//   - Non-overwrite versioning: second run writes helper-registry.v2.md.
//   - Existing-canon observational header (⚠ Existing canon detected).
//   - --canon-output override respected.
//   - Shell safety: path with spaces + $.
//   - Scan-range flags forwarded; scope string reflects actual.
//   - Mode: "fresh-ast" when helperOwnersByIdentity absent;
//     "fresh-ast + helper-owner enrichment" when present.
//   - FanInKind: consumer-file-count always.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(DIR, 'generate-canon-draft.mjs');

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

function buildHelperFixture(fx) {
  write(fx, 'package.json', JSON.stringify({ name: 'hr-fx', type: 'module' }));
  // A non-low-info helper that will have fan-in 1 → shared-helper
  write(fx, 'src/util.ts',
    `export function renderHelperThing(x: number): number { return x + 1 }\n`);
  write(fx, 'src/consumer.ts',
    `import { renderHelperThing } from './util';\n` +
    `export const y = renderHelperThing(1);\n`);
}

// ═══ T1. Happy path — helper draft emitted, exit 0 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-happy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-happy-out-'));
  try {
    buildHelperFixture(fx);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'helper-registry',
    ], { encoding: 'utf8' });

    assert('T1a. exit 0 on happy path', res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);

    const draftPath = path.join(fx, 'canonical-draft', 'helper-registry.md');
    assert('T1b. default canon-output writes to <root>/canonical-draft/helper-registry.md',
      existsSync(draftPath));

    const md = readFileSync(draftPath, 'utf8');
    assert('T1c. draft contains "# Helper registry draft" header',
      md.includes('# Helper registry draft'));
    assert('T1d. draft includes the renderHelperThing identity',
      md.includes('src/util.ts::renderHelperThing'));
    assert('T1e. draft status column carries a helper label marker',
      /shared-helper|central-helper|zero-internal-fan-in-helper|HELPER_LOCAL_COMMON/.test(md));
    assert('T1f. FanInKind line present',
      md.includes('FanInKind: consumer-file-count'));
    assert('T1g. Mode line = "fresh-ast" (no helperOwnersByIdentity enrichment)',
      md.includes('Mode: fresh-ast'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. --source helper-registry accepted; unknown value rejected ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-src-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-src-out-'));
  try {
    buildHelperFixture(fx);
    const resBad = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'foobar',
    ], { encoding: 'utf8' });
    assert('T2a. --source foobar → exit 1 (unknown source)',
      resBad.status === 1);
    assert('T2b. stderr lists all 4 canonical sources (P3 closed)',
      /type-ownership/.test(resBad.stderr) &&
      /helper-registry/.test(resBad.stderr) &&
      /topology/.test(resBad.stderr) &&
      /naming/.test(resBad.stderr));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. --source type-ownership still works (P3-1 regression guard) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-reg-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-reg-out-'));
  try {
    buildHelperFixture(fx);
    // Add a type so type-ownership has something to emit.
    write(fx, 'src/types.ts', `export type User = { id: string };\n`);
    // Minimal symbols.json snapshot so type-ownership has the def.
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      meta: { tool: 'build-symbol-graph.mjs', generated: '2026-04-21T00:00:00Z', root: fx, supports: { identityFanIn: true } },
      defIndex: { 'src/types.ts': { User: { name: 'User', kind: 'TSTypeAliasDeclaration', line: 1 } } },
      fanInByIdentity: {},
      reExportsByFile: {},
    }));
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'type-ownership',
    ], { encoding: 'utf8' });
    assert('T3. --source type-ownership unchanged — exit 0 + draft emitted (P3-1 regression green)',
      res.status === 0 && existsSync(path.join(fx, 'canonical-draft', 'type-ownership.md')),
      `stderr=${res.stderr.slice(0, 300)}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. Missing --root → exit 1 ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-noroot-'));
  try {
    const res = spawnSync(NODE, [CLI,
      '--output', out, '--source', 'helper-registry',
    ], { encoding: 'utf8' });
    assert('T4. missing --root → exit 1',
      res.status === 1);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. Non-overwrite versioning ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-ver-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-ver-out-'));
  try {
    buildHelperFixture(fx);
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const firstContent = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    // Add a new helper so the second run differs.
    write(fx, 'src/extra.ts', `export function anotherHelper() { return 42 }\n`);
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });

    const files = readdirSync(path.join(fx, 'canonical-draft'));
    assert('T5a. second run produces helper-registry.v2.md',
      files.includes('helper-registry.v2.md'));
    assert('T5b. first helper-registry.md preserved byte-for-byte',
      readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8') === firstContent);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T6. Existing canon observational header ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-canon-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-canon-out-'));
  try {
    buildHelperFixture(fx);
    write(fx, 'canonical/helper-registry.md', '# Existing canon (prior content)\n');
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    assert('T6. existing canon → draft carries "⚠ Existing canon detected" for helper-registry',
      md.includes('⚠ Existing canon detected') && md.includes('helper-registry'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T7. --canon-output override ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-cout-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-cout-out-'));
  const cout = mkdtempSync(path.join(tmpdir(), 'cdh-cout-custom-'));
  try {
    buildHelperFixture(fx);
    execFileSync(NODE, [CLI,
      '--root', fx, '--output', out, '--canon-output', cout, '--source', 'helper-registry',
    ], { stdio: 'ignore' });
    assert('T7a. --canon-output custom dir receives the helper draft',
      existsSync(path.join(cout, 'helper-registry.md')));
    assert('T7b. default <root>/canonical-draft/ NOT created',
      !existsSync(path.join(fx, 'canonical-draft', 'helper-registry.md')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
    rmSync(cout, { recursive: true, force: true });
  }
}

// ═══ T8. Path with spaces + $ ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), 'cdh-shell-'));
  const fx = path.join(parent, 'my $root');
  const out = path.join(parent, 'my $out');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildHelperFixture(fx);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'helper-registry',
    ], { encoding: 'utf8' });
    assert('T8. path with spaces + $ survives end-to-end',
      res.status === 0 && existsSync(path.join(fx, 'canonical-draft', 'helper-registry.md')));
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ T9. Scope reflects --production ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-scope-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-scope-out-'));
  try {
    buildHelperFixture(fx);
    execFileSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'helper-registry', '--production',
    ], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    assert('T9. --production → scope "TS/JS production files"',
      md.includes('Scope: TS/JS production files'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T10. call-graph.json absent — draft emits normally, no `[확인 불가]` for absence ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-nocg-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-nocg-out-'));
  try {
    buildHelperFixture(fx);
    // No call-graph.json in `out/`.
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'helper-registry',
    ], { encoding: 'utf8' });
    assert('T10a. exit 0 when call-graph.json absent',
      res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    // v2: call-graph absence is NOT a [확인 불가] reason. The draft should
    // NOT emit a Notes row specifically for absence.
    const noCallGraphMissingNote = !md.includes('call-graph-missing');
    assert('T10b. no `call-graph-missing` Notes row (withdrawn in v2)',
      noCallGraphMissingNote);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T11. Enrichment mode when symbols.json.helperOwnersByIdentity present ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-enrich-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-enrich-out-'));
  try {
    buildHelperFixture(fx);
    // Plant a synthetic symbols.json with helperOwnersByIdentity.
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      meta: { tool: 'build-symbol-graph.mjs', generated: '2026-04-21T00:00:00Z', root: fx, supports: { identityFanIn: true } },
      defIndex: {},
      fanInByIdentity: {},
      reExportsByFile: {},
      helperOwnersByIdentity: {
        'src/util.ts::renderHelperThing': {
          anyContamination: { label: 'has-any', labels: ['has-any'], measurements: {} },
          signature: '(x: number) => number',
        },
      },
    }));
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'helper-registry',
    ], { encoding: 'utf8' });
    assert('T11a. exit 0 with helperOwnersByIdentity enrichment',
      res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    assert('T11b. Mode line = "fresh-ast + helper-owner enrichment"',
      md.includes('Mode: fresh-ast + helper-owner enrichment'));
    assert('T11c. signature surfaces in row',
      md.includes('(x: number) => number'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T12. stale call-graph.json triggers header warning ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-stale-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-stale-out-'));
  try {
    buildHelperFixture(fx);
    // Plant a 30-hour-old call-graph.json
    const oldTs = new Date(Date.now() - 30 * 3600 * 1000).toISOString();
    writeFileSync(path.join(out, 'call-graph.json'), JSON.stringify({
      meta: { generated: oldTs, root: fx, tool: 'build-call-graph.mjs' },
      summary: {},
      topCallees: [],
    }));
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'helper-registry',
    ], { encoding: 'utf8' });
    assert('T12a. exit 0 with stale call-graph',
      res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    assert('T12b. draft header contains stale warning',
      md.includes('stale'));
    // stderr carries staleness status too
    assert('T12c. stderr reports callGraph=stale',
      res.stderr.includes('callGraph=stale'), `stderr=${res.stderr}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
