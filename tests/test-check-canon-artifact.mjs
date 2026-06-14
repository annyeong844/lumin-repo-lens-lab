// tests/test-check-canon-artifact.mjs
//
// P5-1 Step 0 — RED test for `_lib/check-canon-artifact.mjs` (I/O layer).
//
// Confirms the split between pure parser (check-canon-utils) and fs I/O
// per reviewer P0-6. Verifies:
//   - loadTypeOwnershipCanon handles missing files with skipped-missing-canon
//   - writeCanonDriftArtifacts writes JSON always + MD only when provided
//   - no append-merge: prior JSON with foreign sources is overwritten

import { mkdtempSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

import {
  loadTypeOwnershipCanon,
  loadHelperRegistryCanon,
  loadTopologyCanon,
  loadNamingCanon,
  writeCanonDriftArtifacts,
} from '../_lib/check-canon-artifact.mjs';
import { HELPER_LABEL_SET, TOPOLOGY_LABEL_SET, NAMING_LABEL_SET } from '../_lib/check-canon-utils.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed += 1; console.log(`  PASS  ${label}`); }
  else { failed += 1; console.log(`  FAIL  ${label}`); if (detail) console.log(`        ${detail}`); }
}

const TYPE_LABEL_SET = new Set([
  'zero-internal-fan-in', 'low-signal-type-name', 'DUPLICATE_STRONG',
  'DUPLICATE_REVIEW', 'LOCAL_COMMON_NAME', 'single-owner-strong',
  'single-owner-weak', 'severely-any-contaminated', 'ANY_COLLISION',
]);

const workdir = mkdtempSync(path.join(tmpdir(), 'p5-1-artifact-'));

// ── loadTypeOwnershipCanon: missing file ───────────────────────

{
  const r = loadTypeOwnershipCanon({
    canonPath: path.join(workdir, 'does-not-exist.md'),
    canonLabelSet: TYPE_LABEL_SET,
  });
  assert('A-1. missing canon file → status=skipped-missing-canon',
    r.status === 'skipped-missing-canon',
    `status=${r.status}`);
  assert('A-2. missing-canon diagnostic present',
    Array.isArray(r.diagnostics) && r.diagnostics.some((d) => /absent|missing/i.test(d.reason ?? '')),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('A-3. missing-canon returns empty records Map',
    r.records instanceof Map && r.records.size === 0,
    `records=${r.records}`);
}

// ── loadTypeOwnershipCanon: real file delegates to parser ──────

{
  const canonPath = path.join(workdir, 'type-ownership.md');
  const canonText =
    '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
    '|------|----------|-------|-------:|--------|------|\n' +
    '| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | single-owner-strong ✅ | |\n';
  writeFileSync(canonPath, canonText, 'utf8');
  const r = loadTypeOwnershipCanon({ canonPath, canonLabelSet: TYPE_LABEL_SET });
  assert('A-4. real canon file → status=clean + 1 record',
    r.status === 'clean' && r.records.size === 1,
    `status=${r.status}, size=${r.records.size}`);
  assert('A-5. loader populates lineCount from file',
    typeof r.lineCount === 'number' && r.lineCount > 0,
    `lineCount=${r.lineCount}`);
}

// ── writeCanonDriftArtifacts ───────────────────────────────────

{
  const outDir = path.join(workdir, 'out-a');
  const driftObj = {
    meta: { tool: 'check-canon.mjs', generated: '2026-04-21T00:00:00Z', root: '/x', canonDir: '/x/canonical', scope: 'fixture', strict: false },
    summary: { sourcesRequested: 1, sourcesChecked: 1, sourcesSkipped: 0, driftCount: 0 },
    perSource: { 'type-ownership': { status: 'clean', driftCount: 0, reportPath: path.join(outDir, 'canon-drift.type-ownership.md'), diagnostics: [] } },
    drifts: [],
  };
  const paths = writeCanonDriftArtifacts({
    output: outDir,
    driftObject: driftObj,
    reportMarkdown: '# Type-ownership canon drift\n\nclean\n',
    source: 'type-ownership',
  });
  assert('A-6. writeCanonDriftArtifacts writes canon-drift.json',
    existsSync(paths.jsonPath) && paths.jsonPath.endsWith('canon-drift.json'),
    `jsonPath=${paths.jsonPath}`);
  assert('A-7. writeCanonDriftArtifacts writes per-source MD when provided',
    paths.reportPath && existsSync(paths.reportPath) &&
    paths.reportPath.endsWith('canon-drift.type-ownership.md'),
    `reportPath=${paths.reportPath}`);
}

// ── no MD when reportMarkdown is null ──────────────────────────

{
  const outDir = path.join(workdir, 'out-b');
  const driftObj = {
    meta: {}, summary: { sourcesRequested: 1, sourcesChecked: 0, sourcesSkipped: 1, driftCount: 0 },
    perSource: { 'type-ownership': { status: 'skipped-missing-canon', driftCount: 0, diagnostics: [] } },
    drifts: [],
  };
  const paths = writeCanonDriftArtifacts({
    output: outDir,
    driftObject: driftObj,
    reportMarkdown: null,
    source: 'type-ownership',
  });
  assert('A-8. null reportMarkdown → JSON written, MD skipped',
    existsSync(paths.jsonPath) && paths.reportPath === null,
    `jsonPath=${paths.jsonPath}, reportPath=${paths.reportPath}`);
}

// ── no append-merge: foreign prior sources overwritten ─────────

{
  const outDir = path.join(workdir, 'out-c');
  // Pre-seed with a JSON containing a foreign source
  const { mkdirSync } = await import('node:fs');
  mkdirSync(outDir, { recursive: true });
  const priorJson = {
    meta: {}, summary: { sourcesRequested: 1, sourcesChecked: 1, sourcesSkipped: 0, driftCount: 0 },
    perSource: { 'helper-registry': { status: 'drift', driftCount: 5, diagnostics: [] } },
    drifts: [{ kind: 'helper-drift', identity: 'old' }],
  };
  writeFileSync(path.join(outDir, 'canon-drift.json'), JSON.stringify(priorJson, null, 2), 'utf8');
  const newDriftObj = {
    meta: {}, summary: { sourcesRequested: 1, sourcesChecked: 1, sourcesSkipped: 0, driftCount: 0 },
    perSource: { 'type-ownership': { status: 'clean', driftCount: 0, diagnostics: [] } },
    drifts: [],
  };
  writeCanonDriftArtifacts({
    output: outDir,
    driftObject: newDriftObj,
    reportMarkdown: null,
    source: 'type-ownership',
  });
  const after = JSON.parse(readFileSync(path.join(outDir, 'canon-drift.json'), 'utf8'));
  assert('A-9. prior helper-registry entry is OVERWRITTEN, not merged (P0-6)',
    !after.perSource['helper-registry'] &&
    after.perSource['type-ownership'] &&
    after.drifts.length === 0,
    `perSource keys=${Object.keys(after.perSource)}, drifts=${after.drifts.length}`);
}

// ── P5-2: loadHelperRegistryCanon ──────────────────────────────

{
  const r = loadHelperRegistryCanon({
    canonPath: path.join(workdir, 'nonexistent-helper.md'),
    canonLabelSet: HELPER_LABEL_SET,
  });
  assert('AH-1. missing helper canon → status=skipped-missing-canon',
    r.status === 'skipped-missing-canon', `status=${r.status}`);
  assert('AH-2. missing helper canon diagnostic present',
    Array.isArray(r.diagnostics) && r.diagnostics.some((d) => /absent|missing/i.test(d.reason ?? '')),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

{
  const canonPath = path.join(workdir, 'helper-registry.md');
  const canonText =
    '| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n' +
    '|------|----------|-------|-----------|-------:|--------|------|----------------------|\n' +
    '| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | () => void | 3 | central-helper ✅ | | |\n';
  writeFileSync(canonPath, canonText, 'utf8');
  const r = loadHelperRegistryCanon({ canonPath, canonLabelSet: HELPER_LABEL_SET });
  assert('AH-3. real helper canon → status=clean + 1 record',
    r.status === 'clean' && r.records.size === 1, `status=${r.status}, size=${r.records.size}`);
  assert('AH-4. helper canon lineCount populated',
    typeof r.lineCount === 'number' && r.lineCount > 0, `lineCount=${r.lineCount}`);
}

// ── P5-3: loadTopologyCanon ────────────────────────────────────

{
  const r = loadTopologyCanon({
    canonPath: path.join(workdir, 'nonexistent-topology.md'),
    canonLabelSet: TOPOLOGY_LABEL_SET,
  });
  assert('AY-1. missing topology canon → status=skipped-missing-canon',
    r.status === 'skipped-missing-canon', `status=${r.status}`);
  assert('AY-2. missing topology canon diagnostic present',
    Array.isArray(r.diagnostics) && r.diagnostics.some((d) => /absent|missing/i.test(d.reason ?? '')),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

{
  const canonPath = path.join(workdir, 'topology.md');
  const canonText = [
    '## 1. Submodule inventory',
    '',
    '| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |',
    '|-----------|------:|----:|---------:|----------:|-----|--------|------|',
    '| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |',
    '',
    '## 3. Cycles (SCCs)',
    '',
    '✅ No submodule-level cycles observed. Repo is acyclic at submodule granularity.',
    '',
  ].join('\n');
  writeFileSync(canonPath, canonText, 'utf8');
  const r = loadTopologyCanon({ canonPath, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('AY-3. real topology canon → status=clean + inventory populated',
    r.status === 'clean' && r.inventory.size === 1, `status=${r.status}, size=${r.inventory.size}`);
  assert('AY-4. topology canon lineCount populated',
    typeof r.lineCount === 'number' && r.lineCount > 0, `lineCount=${r.lineCount}`);
}

// ── P5-4: loadNamingCanon ──────────────────────────────────────

{
  const r = loadNamingCanon({
    canonPath: path.join(workdir, 'nonexistent-naming.md'),
    canonLabelSet: NAMING_LABEL_SET,
  });
  assert('AN-1. missing naming canon → status=skipped-missing-canon',
    r.status === 'skipped-missing-canon', `status=${r.status}`);
  assert('AN-2. missing naming canon diagnostic present',
    Array.isArray(r.diagnostics) && r.diagnostics.some((d) => /absent|missing/i.test(d.reason ?? '')),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

{
  const canonPath = path.join(workdir, 'naming.md');
  const canonText = [
    '## 1. File-naming cohorts', '',
    '| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------|------:|--------------------|----------------:|--------------:|--------|',
    '| `src` | 3 | `kebab-case` | 100% | 0 | kebab-case-dominant ✅ |',
    '',
    '## 2. Symbol-naming cohorts', '',
    '| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------------|------:|--------------------|----------------:|--------------:|--------|',
    '',
  ].join('\n');
  writeFileSync(canonPath, canonText, 'utf8');
  const r = loadNamingCanon({ canonPath, canonLabelSet: NAMING_LABEL_SET });
  assert('AN-3. real naming canon → clean + fileCohorts populated',
    r.status === 'clean' && r.fileCohorts.size === 1, `status=${r.status}`);
  assert('AN-4. naming canon lineCount populated',
    typeof r.lineCount === 'number' && r.lineCount > 0, `lineCount=${r.lineCount}`);
}

rmSync(workdir, { recursive: true, force: true });

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
