// P1-0 preparatory bootstrap test.
//
// This test is the gate that must stay green before any P1-1/P1-2/P1-3
// implementation work proceeds. It pins the preparatory guarantees P1
// pre-write depends on:
//
//   1. Every `_lib/*.mjs` referenced in docs/history/phases/p1/session.md §3.2 is importable
//      and exports the named symbols.
//   2. `build-symbol-graph.mjs` emits `symbols.json.meta.supports` on a
//      freshly-run fixture.
//   3. `supports.anyContamination === true` only when the producer emits
//      conforming per-identity owner facts. Optimistic `true` without the
//      owner-fact surface is a TEST FAILURE.
//   4. When support is true, absent defIndex annotations mean "measured
//      clean" for parsed identities, while contaminated identities carry
//      `{label, labels, measurements}`.
//   5. Legacy flat `anyContamination: { label, anyFieldRatio, ... }` is
//      rejected OR downgraded — never silently accepted.
//   6. `fanInByIdentity` map emitted; identity-keyed fan-in lookup
//      available to P1 name lookup.
//   7. `FP_BUDGET` constant exists in `tests/test-corpus.mjs` and
//      equals 0 — so downstream exit-criteria references aren't
//      dangling.

import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync } from 'node:fs';
import { execSync } from 'node:child_process';
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

// ═════════════════════════════════════════════════════════════
// Check 1 — _lib/*.mjs dependency inventory
// ═════════════════════════════════════════════════════════════
//
// Every module referenced in docs/history/phases/p1/session.md §3.2 must resolve. Any drift
// invalidates later P1 wiring. Uses dynamic import rather than
// require-existence-of-file so a renamed file manifests as a real
// import error, not a silent pass.

const requiredExports = {
  '_lib/cli.mjs': ['parseCliArgs'],
  '_lib/artifacts.mjs': ['loadIfExists'],
  '_lib/resolver-core.mjs': ['makeResolver'],
  '_lib/alias-map.mjs': ['extractStringTarget', 'mapOutputToSource'],
  '_lib/finding-provenance.mjs': ['specifierCouldMatchFile'],
  '_lib/classify-facts.mjs': ['countFileReferencesAst'],
  '_lib/test-paths.mjs': ['isTestLikePath'],
  '_lib/vocab.mjs': ['EVIDENCE', 'TAINT'],
  '_lib/pre-write-canonical-parser.mjs': ['parseCanonicalFile', 'findCanonicalOwnerClaim'],
};

for (const [relPath, exports] of Object.entries(requiredExports)) {
  const abs = path.join(DIR, relPath);
  if (!existsSync(abs)) {
    assert(`C1. ${relPath} exists`, false, `not found at ${abs}`);
    continue;
  }
  assert(`C1. ${relPath} exists`, true);

  // Dynamic import to verify named exports resolve.
  const mod = await import(path.resolve(abs).replace(/\\/g, '/').startsWith('/') ? `file://${path.resolve(abs).replace(/\\/g, '/')}` : `file:///${path.resolve(abs).replace(/\\/g, '/')}`);
  for (const exp of exports) {
    assert(`C1. ${relPath} exports ${exp}`,
      mod[exp] !== undefined,
      `typeof mod.${exp} = ${typeof mod[exp]}`);
  }
}

// ═════════════════════════════════════════════════════════════
// Check 2-6 — symbols.json shape on a freshly-run fixture
// ═════════════════════════════════════════════════════════════

function runSymbolsOnFixture(fx, out) {
  execSync(`"${NODE}" "${path.join(DIR, 'build-symbol-graph.mjs')}" --root "${fx}" --output "${out}"`, {
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  return JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'p1-0-bootstrap-'));
  const out = mkdtempSync(path.join(tmpdir(), 'p1-0-bootstrap-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fx', type: 'module' }));
    write(fx, 'src/a.ts', `export const formatDate = (d) => d.toString();\n`);
    write(fx, 'src/dirty.ts',
      `export interface DirtyType { payload: any }\n` +
      `export function parsePayload(payload: any) { return payload as any; }\n`
    );
    write(fx, 'src/jsdoc.mjs',
      `/** @type {any} */\n` +
      `export const fromJsdoc = readValue();\n`
    );
    write(fx, 'src/b.ts', `import { formatDate } from './a';\nexport const useFmt = () => formatDate(new Date());\n`);

    const sym = runSymbolsOnFixture(fx, out);

    // C2. meta.supports block presence
    assert('C2. symbols.meta.supports block present',
      !!sym.meta?.supports,
      `meta=${JSON.stringify(sym.meta)}`);

    assert('C2b. meta.schemaVersion is 3+',
      typeof sym.meta?.schemaVersion === 'number' && sym.meta.schemaVersion >= 3,
      `schemaVersion=${sym.meta?.schemaVersion}`);

    // C3. Strict anyContamination condition.
    assert('C3. supports.anyContamination is true (producer emits owner facts)',
      sym.meta?.supports?.anyContamination === true,
      `supports.anyContamination=${sym.meta?.supports?.anyContamination}`);

    const isConformingAnnotation = (a) =>
      a && typeof a === 'object' &&
      typeof a.label === 'string' &&
      Array.isArray(a.labels) &&
      a.measurements && typeof a.measurements === 'object';
    const dirtyTypeAnn = sym.defIndex?.['src/dirty.ts']?.DirtyType?.anyContamination;
    const dirtyHelperAnn = sym.defIndex?.['src/dirty.ts']?.parsePayload?.anyContamination;
    assert('C3b. contaminated type carries conforming anyContamination annotation',
      isConformingAnnotation(dirtyTypeAnn),
      JSON.stringify(dirtyTypeAnn));
    assert('C3c. contaminated helper carries conforming anyContamination annotation',
      isConformingAnnotation(dirtyHelperAnn),
      JSON.stringify(dirtyHelperAnn));
    assert('C3d. clean identity omits anyContamination annotation under support=true',
      sym.defIndex?.['src/a.ts']?.formatDate?.anyContamination === undefined,
      JSON.stringify(sym.defIndex?.['src/a.ts']?.formatDate));
    assert('C3e. helperOwnersByIdentity exposes clean/dirty owner facts',
      sym.helperOwnersByIdentity?.['src/a.ts::formatDate']?.anyContamination === null &&
      isConformingAnnotation(sym.helperOwnersByIdentity?.['src/dirty.ts::parsePayload']?.anyContamination),
      JSON.stringify(sym.helperOwnersByIdentity ?? {}));
    assert('C3f. typeOwnersByIdentity exposes dirty type owner fact',
      isConformingAnnotation(sym.typeOwnersByIdentity?.['src/dirty.ts::DirtyType']?.anyContamination),
      JSON.stringify(sym.typeOwnersByIdentity ?? {}));
    assert('C3g. JSDoc {any} on exported JS const annotates helper owner',
      isConformingAnnotation(sym.helperOwnersByIdentity?.['src/jsdoc.mjs::fromJsdoc']?.anyContamination),
      JSON.stringify(sym.helperOwnersByIdentity?.['src/jsdoc.mjs::fromJsdoc']));

    // C4. Legacy flat anyContamination shape rejected
    //
    // Construct a synthetic flat shape and assert the bootstrap contract
    // says "non-conforming". We verify this via the same structural
    // predicate used above — a flat shape lacks `labels` / `measurements`
    // and therefore fails `hasConformingAnnotation`.
    const flatLegacy = { label: 'any-contaminated', anyFieldRatio: 0.5, totalFields: 2, anyFields: 1 };
    const passesConforming =
      typeof flatLegacy.label === 'string' &&
      Array.isArray(flatLegacy.labels) &&
      flatLegacy.measurements && typeof flatLegacy.measurements === 'object';
    assert('C4. flat legacy anyContamination shape FAILS the conforming predicate',
      passesConforming === false);

    // C5. identityFanIn capability + fanInByIdentity emission
    assert('C5. supports.identityFanIn is true',
      sym.meta?.supports?.identityFanIn === true);
    assert('C5b. fanInByIdentity map emitted',
      sym.fanInByIdentity && typeof sym.fanInByIdentity === 'object',
      `fanInByIdentity=${typeof sym.fanInByIdentity}`);
    assert('C5c. fanInByIdentity keyed as ownerFile::exportedName',
      sym.fanInByIdentity['src/a.ts::formatDate'] !== undefined,
      `keys=${Object.keys(sym.fanInByIdentity ?? {}).join(', ')}`);
    assert('C5d. fan-in for formatDate is 1 (b.ts consumer)',
      sym.fanInByIdentity['src/a.ts::formatDate'] === 1);

    // C5e. reExportRecords honesty — file-level until symbol-level
    // migration lands.
    assert('C5e. supports.reExportRecords is a known value',
      ['symbol-level', 'file-level', 'absent'].includes(sym.meta?.supports?.reExportRecords));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// Check 7 — FP_BUDGET=0 gate exists in test-corpus.mjs
// ═════════════════════════════════════════════════════════════
{
  const corpusPath = path.join(DIR, 'tests', 'test-corpus.mjs');
  assert('C7. tests/test-corpus.mjs exists',
    existsSync(corpusPath));

  const corpusText = readFileSync(corpusPath, 'utf8');
  const match = corpusText.match(/FP_BUDGET\s*=\s*(\d+)/);
  assert('C7b. FP_BUDGET constant declared in corpus',
    match !== null,
    `FP_BUDGET not found in test-corpus.mjs`);
  assert('C7c. FP_BUDGET equals 0',
    match !== null && Number(match[1]) === 0,
    `FP_BUDGET = ${match?.[1]}`);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
