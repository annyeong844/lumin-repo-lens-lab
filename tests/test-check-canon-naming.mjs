// tests/test-check-canon-naming.mjs
//
// P5-4 Step 0 — RED test for `_lib/check-canon-naming.mjs`.
//
// Pins naming-drift 5-category enum, 2 sub-diffs (cohort + outlier),
// per-category identity format per canon-drift.md §4 (PF-4):
//   file cohort   → <submodule>
//   symbol cohort → <submodule>::<kind>   (kind ∈ {type-export, helper-export, constant-export})
//   file outlier  → <ownerFile>
//   symbol outlier→ <ownerFile>::<exportedName>

import { writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';

import { detectNamingDrift } from '../_lib/check-canon-naming.mjs';
import { NAMING_LABEL_SET } from '../_lib/check-canon-utils.mjs';

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed += 1; console.log(`  PASS  ${label}`); }
  else { failed += 1; console.log(`  FAIL  ${label}`); if (detail) console.log(`        ${detail}`); }
}

const workdir = mkdtempSync(path.join(tmpdir(), 'p5-4-engine-'));

// ── Canon MD builders ──────────────────────────────────────────

function buildCanonNamingMd({ fileCohorts = [], symbolCohorts = [], outliers = null }) {
  const lines = [];
  lines.push('# Naming canon (fixture)', '');

  lines.push('## 1. File-naming cohorts', '');
  lines.push('| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |');
  lines.push('|--------------------|------:|--------------------|----------------:|--------------:|--------|');
  for (const c of fileCohorts) {
    lines.push(`| \`${c.cohort}\` | ${c.files} | \`${c.convention}\` | ${c.rate}% | ${c.outliers ?? 0} | ${c.label} ✅ |`);
  }
  lines.push('');

  lines.push('## 2. Symbol-naming cohorts', '');
  lines.push('| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |');
  lines.push('|--------------------------|------:|--------------------|----------------:|--------------:|--------|');
  for (const c of symbolCohorts) {
    lines.push(`| \`${c.cohort}\` | ${c.items} | \`${c.convention}\` | ${c.rate}% | ${c.outliers ?? 0} | ${c.label} ✅ |`);
  }
  lines.push('');

  if (outliers !== null) {
    lines.push('## 3. Outliers', '');
    lines.push('| Identity | Cohort | Name | ObservedConvention | DominantConvention | Status |');
    lines.push('|----------|--------|------|--------------------|--------------------|--------|');
    for (const o of outliers) {
      lines.push(`| \`${o.identity}\` | \`${o.cohort}\` | \`${o.name}\` | \`${o.observed}\` | \`${o.dominant}\` | ${o.label} ⚠ |`);
    }
    lines.push('');
  }
  return lines.join('\n');
}

function writeCanon(canonPath, spec) {
  writeFileSync(canonPath, buildCanonNamingMd(spec), 'utf8');
}

// ── Stub scanContext builder ───────────────────────────────────
//
// The engine calls `collectNamingCohorts({ files, root, extractFn,
// submoduleOf, lowInfoNames, lowInfoHelperNames })`. For unit-test purity
// we inject a stub extractFn + submoduleOf + files list. Tests control the
// fresh-side cohorts/outliers directly through what the stubs return.

function makeExtractStub(byFile, failing = new Set()) {
  return (absFile) => {
    if (failing.has(absFile)) throw new Error(`forced-throw on ${absFile}`);
    return byFile.get(absFile) ?? { defs: [], uses: [], reExports: [] };
  };
}

function submoduleOfTopDir(root) {
  return (absFile) => {
    const rel = path.relative(root, absFile).replace(/\\/g, '/');
    const firstSlash = rel.indexOf('/');
    return firstSlash < 0 ? rel : rel.slice(0, firstSlash);
  };
}

function buildScan({ root, files, defsByFile = new Map(), failing = new Set(), lowInfoNames = new Set(), lowInfoHelperNames = new Set() }) {
  return {
    files,
    root,
    extractFn: makeExtractStub(defsByFile, failing),
    submoduleOf: submoduleOfTopDir(root),
    lowInfoNames,
    lowInfoHelperNames,
  };
}

// ── N-1: missing canon → skipped-missing-canon ────────────────

{
  const r = detectNamingDrift({
    canonPath: path.join(workdir, 'nope.md'),
    scanContext: buildScan({ root: workdir, files: [] }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  assert('N-1a. missing canon → skipped-missing-canon',
    r.status === 'skipped-missing-canon', `status=${r.status}`);
  assert('N-1b. missing canon → drifts empty + reportMarkdown null',
    r.drifts.length === 0 && r.reportMarkdown === null, '');
}

// ── N-2: NAMING_LABEL_SET has exactly 10 entries (§12.3) ───────

{
  const expected = new Set([
    'camelCase-dominant', 'PascalCase-dominant', 'kebab-case-dominant',
    'snake_case-dominant', 'UPPER_SNAKE-dominant', 'mixed-convention',
    'insufficient-evidence', 'convention-match', 'convention-outlier',
    'low-info-excluded',
  ]);
  assert('N-2a. NAMING_LABEL_SET size = 10',
    NAMING_LABEL_SET.size === 10, `size=${NAMING_LABEL_SET.size}`);
  assert('N-2b. NAMING_LABEL_SET matches §12.3 verbatim',
    [...expected].every((l) => NAMING_LABEL_SET.has(l)),
    `missing=${[...expected].filter((l) => !NAMING_LABEL_SET.has(l)).join(',')}`);
}

// ── N-3: cohort-added (file cohort) ───────────────────────────

{
  const canonPath = path.join(workdir, 'n3-added.md');
  // Canon has only `src` file cohort; fresh will see `src` + `lib` dirs.
  writeCanon(canonPath, {
    fileCohorts: [{ cohort: 'src', files: 3, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' }],
    symbolCohorts: [],
  });
  const srcA = path.join(workdir, 'src/a.ts');
  const srcB = path.join(workdir, 'src/b.ts');
  const srcC = path.join(workdir, 'src/c.ts');
  const libD = path.join(workdir, 'lib/d.ts');
  const libE = path.join(workdir, 'lib/e.ts');
  const libF = path.join(workdir, 'lib/f.ts');
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({ root: workdir, files: [srcA, srcB, srcC, libD, libE, libF] }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  const added = r.drifts.filter((d) => d.category === 'cohort-added');
  assert('N-3a. cohort-added fires when fresh gains a file cohort',
    added.length === 1 && added[0].identity === 'lib',
    `drifts=${JSON.stringify(r.drifts.map((d) => ({ c: d.category, id: d.identity })))}`);
  assert('N-3b. cohort-added family = added',
    added[0]?.family === 'added', `family=${added[0]?.family}`);
  assert('N-3c. PF-4: file cohort identity has no "::" and no "→"',
    !added[0]?.identity.includes('::') && !added[0]?.identity.includes('→'), '');
}

// ── N-4: cohort-removed (file cohort) ─────────────────────────

{
  const canonPath = path.join(workdir, 'n4-removed.md');
  writeCanon(canonPath, {
    fileCohorts: [
      { cohort: 'src', files: 3, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' },
      { cohort: 'gone', files: 2, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' },
    ],
    symbolCohorts: [],
  });
  const srcA = path.join(workdir, 'src/a.ts');
  const srcB = path.join(workdir, 'src/b.ts');
  const srcC = path.join(workdir, 'src/c.ts');
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({ root: workdir, files: [srcA, srcB, srcC] }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  const removed = r.drifts.filter((d) => d.category === 'cohort-removed');
  assert('N-4a. cohort-removed fires when canon has an extra cohort',
    removed.length === 1 && removed[0].identity === 'gone', '');
  assert('N-4b. cohort-removed family = removed',
    removed[0]?.family === 'removed', `family=${removed[0]?.family}`);
}

// ── N-5: cohort-convention-shifted ────────────────────────────

{
  const canonPath = path.join(workdir, 'n5-shifted.md');
  // Canon claims kebab-case-dominant; fresh has camelCase filenames → classifier returns camelCase-dominant
  writeCanon(canonPath, {
    fileCohorts: [{ cohort: 'src', files: 3, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' }],
    symbolCohorts: [],
  });
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({
      root: workdir,
      files: [
        path.join(workdir, 'src/alphaBeta.ts'),
        path.join(workdir, 'src/gammaDelta.ts'),
        path.join(workdir, 'src/epsilonZeta.ts'),
      ],
    }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  const shifted = r.drifts.filter((d) => d.category === 'cohort-convention-shifted');
  assert('N-5a. cohort-convention-shifted fires when convention differs',
    shifted.length === 1 && shifted[0].identity === 'src',
    `drifts=${JSON.stringify(r.drifts.map((d) => ({ c: d.category, id: d.identity, ccv: d.canon?.dominantConvention, fcv: d.fresh?.dominantConvention })))}`);
  assert('N-5b. shifted record carries canon.dominantConvention + fresh.dominantConvention',
    shifted[0]?.canon?.dominantConvention === 'kebab-case' &&
    typeof shifted[0]?.fresh?.dominantConvention === 'string' &&
    shifted[0].fresh.dominantConvention !== 'kebab-case',
    `canon=${shifted[0]?.canon?.dominantConvention}, fresh=${shifted[0]?.fresh?.dominantConvention}`);
  assert('N-5c. cohort-convention-shifted family = label-changed',
    shifted[0]?.family === 'label-changed', `family=${shifted[0]?.family}`);
}

// ── N-6: symbol cohort identity uses "::<kind>" suffix ────────

{
  const canonPath = path.join(workdir, 'n6-symcohort.md');
  writeCanon(canonPath, {
    fileCohorts: [],
    symbolCohorts: [{ cohort: 'src::helper-export', items: 3, convention: 'camelCase', rate: 100, label: 'camelCase-dominant' }],
  });
  // Fresh: no symbol cohort at src → cohort-removed for `src::helper-export`
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({ root: workdir, files: [] }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  const removed = r.drifts.filter((d) => d.category === 'cohort-removed');
  assert('N-6a. PF-4: symbol cohort identity preserves "::<kind>" suffix',
    removed.some((d) => d.identity === 'src::helper-export'),
    `removed=${JSON.stringify(removed.map((d) => d.identity))}`);
  assert('N-6b. PF-4: symbol cohort identity kind ∈ {type-export, helper-export, constant-export}',
    removed.filter((d) => d.identity.includes('::')).every((d) => {
      const kind = d.identity.split('::')[1];
      return ['type-export', 'helper-export', 'constant-export'].includes(kind);
    }), '');
}

// ── N-7: clean run (zero drift) ───────────────────────────────

{
  const canonPath = path.join(workdir, 'n7-clean.md');
  writeCanon(canonPath, { fileCohorts: [], symbolCohorts: [] });
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({ root: workdir, files: [] }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  assert('N-7a. empty canon + empty fresh → clean',
    r.status === 'clean' && r.drifts.length === 0,
    `status=${r.status}, drifts=${r.drifts.length}`);
  assert('N-7b. clean → reportMarkdown has §1 Summary',
    typeof r.reportMarkdown === 'string' && r.reportMarkdown.includes('## 1. Summary'), '');
  assert('N-7c. clean → MD omits category sections',
    !r.reportMarkdown.includes('## 2. cohort-added') &&
    !r.reportMarkdown.includes('## 3. cohort-removed'), '');
}

// ── N-8: kind=naming-drift invariant ──────────────────────────

{
  const canonPath = path.join(workdir, 'n8-kind.md');
  writeCanon(canonPath, {
    fileCohorts: [{ cohort: 'gone', files: 1, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' }],
    symbolCohorts: [],
  });
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({ root: workdir, files: [] }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  assert('N-8. every drift record has kind=naming-drift',
    r.drifts.length > 0 && r.drifts.every((d) => d.kind === 'naming-drift'),
    `kinds=${JSON.stringify(r.drifts.map((d) => d.kind))}`);
}

// ── N-9: outlier records (new-outlier-introduced + outlier-resolved) ─

{
  const canonPath = path.join(workdir, 'n9-outliers.md');
  // Canon declares one outlier: src/OLD.ts. No canon outlier for src/NEW.ts.
  writeCanon(canonPath, {
    fileCohorts: [{ cohort: 'src', files: 4, convention: 'kebab-case', rate: 75, outliers: 1, label: 'kebab-case-dominant' }],
    symbolCohorts: [],
    outliers: [
      { identity: 'src/OLD.ts', cohort: 'src', name: 'OLD.ts', observed: 'UPPER_SNAKE', dominant: 'kebab-case', label: 'convention-outlier' },
    ],
  });
  // Fresh has a different outlier: src/NEW.ts (camelCase-shaped) + OLD.ts resolved (renamed to conforming old.ts not present)
  // i.e. canon's OLD.ts is not in the fresh outlier set → outlier-resolved.
  // fresh sees a new outlier src/FOO.ts (camelCase) → new-outlier-introduced.
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({
      root: workdir,
      files: [
        path.join(workdir, 'src/a.ts'),
        path.join(workdir, 'src/b.ts'),
        path.join(workdir, 'src/c.ts'),
        path.join(workdir, 'src/FOO.ts'), // UPPER → outlier in kebab cohort
      ],
    }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  const introduced = r.drifts.filter((d) => d.category === 'new-outlier-introduced');
  const resolved = r.drifts.filter((d) => d.category === 'outlier-resolved');
  assert('N-9a. new-outlier-introduced records the fresh-only outlier',
    introduced.length === 1 && /FOO\.ts/.test(introduced[0]?.identity ?? ''),
    `introduced=${JSON.stringify(introduced.map((d) => d.identity))}`);
  assert('N-9b. outlier-resolved records the canon-only outlier',
    resolved.length === 1 && /OLD\.ts/.test(resolved[0]?.identity ?? ''),
    `resolved=${JSON.stringify(resolved.map((d) => d.identity))}`);
  assert('N-9c. outlier identity is a file path (no ::) for file outliers',
    [...introduced, ...resolved].every((d) => !d.identity.includes('::')), '');
  assert('N-9d. new-outlier-introduced family = content-shifted',
    introduced[0]?.family === 'content-shifted', `family=${introduced[0]?.family}`);
  assert('N-9e. outlier-resolved family = content-shifted',
    resolved[0]?.family === 'content-shifted', `family=${resolved[0]?.family}`);
}

// ── N-10: Finding #3 post-landing — extractor-throw promotes to parse-error ─

{
  const canonPath = path.join(workdir, 'n10-extractor-throw.md');
  writeCanon(canonPath, {
    fileCohorts: [{ cohort: 'src', files: 3, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' }],
    symbolCohorts: [],
  });
  const fooAbs = path.join(workdir, 'src/foo.ts');
  const barAbs = path.join(workdir, 'src/bar.ts');
  const badAbs = path.join(workdir, 'src/broken.ts');
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({
      root: workdir,
      files: [fooAbs, barAbs, badAbs],
      failing: new Set([badAbs]),  // forces extractor throw
    }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  assert('N-10a. extractor-throw → status=parse-error (not drift on partial fresh set)',
    r.status === 'parse-error', `status=${r.status}`);
  assert('N-10b. extractor-throw → drifts empty + reportMarkdown null',
    r.drifts.length === 0 && r.reportMarkdown === null, '');
  assert('N-10c. original parse-error diagnostic preserved with file target',
    r.diagnostics.some((d) => d.kind === 'parse-error' && /broken\.ts/.test(d.target ?? '')),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// ── N-11: P3 display dash and P5 null are equivalent ───────────
//
// Regression for FP-43: a freshly generated P3 naming draft renders a mixed
// cohort's DominantConvention as `—`, while the P5 fresh collector represents
// the same absence as null. Promoting that draft must not immediately produce
// cohort-convention-shifted noise.

{
  const canonPath = path.join(workdir, 'n11-dash-null-clean.md');
  writeCanon(canonPath, {
    fileCohorts: [{ cohort: 'src', files: 5, convention: '—', rate: 40, label: 'mixed-convention' }],
    symbolCohorts: [],
  });
  const r = detectNamingDrift({
    canonPath,
    scanContext: buildScan({
      root: workdir,
      files: [
        path.join(workdir, 'src/fooBar.ts'),
        path.join(workdir, 'src/bazQux.ts'),
        path.join(workdir, 'src/FooBar.ts'),
        path.join(workdir, 'src/BazQux.ts'),
        path.join(workdir, 'src/foo-bar.ts'),
      ],
    }),
    canonLabelSet: NAMING_LABEL_SET,
  });
  assert('N-11a. promoted mixed-convention draft with "—" dominant stays clean',
    r.status === 'clean' && r.drifts.length === 0,
    `status=${r.status}, drifts=${JSON.stringify(r.drifts)}`);
}

rmSync(workdir, { recursive: true, force: true });

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
