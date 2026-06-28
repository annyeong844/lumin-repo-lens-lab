// Tests for `collectNamingCohorts` + `renderNaming` — P3-4 Step 2.
//
// DI-style: supply pre-canned file list + extractFn + submoduleOf.
// Integration tests (real extractor + resolver) live in
// test-canon-draft-integration-naming.mjs (covered by CLI smoke + Step 5
// dogfood — no separate integration file in v1).

import {
  LOW_INFO_NAMES,
  LOW_INFO_HELPER_NAMES,
} from '../_lib/canon-draft-utils.mjs';
import {
  collectNamingCohorts,
  renderNaming,
} from '../_lib/canon-draft-naming.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function makeExtractFn(perFile) {
  return function extract(absFile) {
    const entry = perFile.get(absFile);
    if (!entry) return { defs: [], uses: [], reExports: [] };
    return entry;
  };
}

function makeSubmoduleOf(resolves) {
  return function submoduleOf(abs) {
    return resolves.get(abs) ?? 'root';
  };
}

const ROOT = '/fx';

const LOW_INFO_ALL = new Set(LOW_INFO_NAMES);
const LOW_INFO_HELP = new Set(LOW_INFO_HELPER_NAMES);

// ═══ I1. 5-file kebab cohort + 1 snake_case outlier ═══

{
  const files = [
    '/fx/_lib/canon-draft.mjs',
    '/fx/_lib/alias-map.mjs',
    '/fx/_lib/extract-ts.mjs',
    '/fx/_lib/resolver-core.mjs',
    '/fx/_lib/legacy_helper.mjs',
  ];
  const perFile = new Map();
  const resolves = new Map(files.map((f) => [f, '_lib']));
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn: makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  assert('I1a. 1 file cohort (_lib)',
    result.fileCohorts.size === 1 && result.fileCohorts.has('_lib'));
  const cohort = result.fileCohorts.get('_lib');
  assert('I1b. 5 members in cohort',
    cohort.members.length === 5);
  assert('I1c. cohort classification = kebab-case-dominant (4 of 5)',
    cohort.classification.label === 'kebab-case-dominant' &&
    Math.abs(cohort.classification.consistencyRate - 0.8) < 0.001);
  assert('I1d. 1 outlier in perItemRows (the snake_case file)',
    result.perItemRows.length === 1 &&
    result.perItemRows[0].identity === '_lib/legacy_helper.mjs' &&
    result.perItemRows[0].itemLabel === 'convention-outlier');
}

// ═══ I2. Symbol cohort: 10 helper exports camelCase-dominant ═══

{
  const files = ['/fx/_lib/u.mjs'];
  const perFile = new Map([
    ['/fx/_lib/u.mjs', {
      defs: [
        { name: 'parseJson', kind: 'FunctionDeclaration', line: 1 },
        { name: 'stringifyJson', kind: 'FunctionDeclaration', line: 2 },
        { name: 'fetchData', kind: 'FunctionDeclaration', line: 3 },
        { name: 'renderThing', kind: 'FunctionDeclaration', line: 4 },
        { name: 'computeThing', kind: 'FunctionDeclaration', line: 5 },
        { name: 'doTheThing', kind: 'FunctionDeclaration', line: 6 },
        { name: 'mkLogger', kind: 'FunctionDeclaration', line: 7 },
        { name: 'validateInput', kind: 'FunctionDeclaration', line: 8 },
        { name: 'normalizePath', kind: 'FunctionDeclaration', line: 9 },
        { name: 'MyBadFunc', kind: 'FunctionDeclaration', line: 10 },   // PascalCase outlier
      ],
      uses: [], reExports: [],
    }],
  ]);
  const resolves = new Map([['/fx/_lib/u.mjs', '_lib']]);
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn: makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  assert('I2a. _lib::helper-export cohort exists',
    result.symbolCohorts.has('_lib::helper-export'));
  const cohort = result.symbolCohorts.get('_lib::helper-export');
  assert('I2b. cohort has 10 members',
    cohort.members.length === 10);
  assert('I2c. cohort classification = camelCase-dominant',
    cohort.classification.label === 'camelCase-dominant' &&
    Math.abs(cohort.classification.consistencyRate - 0.9) < 0.001);
  const outlier = result.perItemRows.find((r) => r.name === 'MyBadFunc');
  assert('I2d. MyBadFunc is convention-outlier',
    outlier && outlier.itemLabel === 'convention-outlier');
  assert('I2e. outlier identity uses <ownerFile>::<exportedName>',
    outlier?.identity === '_lib/u.mjs::MyBadFunc');
}

// ═══ I3. Kind classification: type / helper / constant ═══

{
  const files = ['/fx/_lib/mixed.mjs'];
  const perFile = new Map([
    ['/fx/_lib/mixed.mjs', {
      defs: [
        { name: 'UserType', kind: 'TSInterfaceDeclaration', line: 1 },
        { name: 'FooAlias', kind: 'TSTypeAliasDeclaration', line: 2 },
        { name: 'MAX_RETRY', kind: 'const-var', line: 3 },     // constant-export (no initType)
        { name: 'DEFAULT', kind: 'const-var', line: 4 },       // constant-export
        { name: 'parseIt', kind: 'const-var', line: 5, initType: 'ArrowFunctionExpression' },  // helper-export
        { name: 'helperFn', kind: 'FunctionDeclaration', line: 6 },   // helper-export
      ],
      uses: [], reExports: [],
    }],
  ]);
  const resolves = new Map([['/fx/_lib/mixed.mjs', '_lib']]);
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn: makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  const typeCohort = result.symbolCohorts.get('_lib::type-export');
  const helperCohort = result.symbolCohorts.get('_lib::helper-export');
  const constCohort = result.symbolCohorts.get('_lib::constant-export');
  assert('I3a. type-export cohort has 2 members',
    typeCohort?.members.length === 2);
  assert('I3b. helper-export cohort has 2 members (FunctionDeclaration + arrow const-var)',
    helperCohort?.members.length === 2);
  assert('I3c. constant-export cohort has 2 members (const-var no initType)',
    constCohort?.members.length === 2);
}

// ═══ I4. File cohort authoritativeness — symbols absent doesn't shrink inventory ═══

{
  const files = [
    '/fx/_lib/a.mjs', '/fx/_lib/b.mjs', '/fx/_lib/c.mjs',
    '/fx/_lib/d.mjs', '/fx/_lib/e.mjs',
  ];
  // Only 3 files have exports.
  const perFile = new Map([
    ['/fx/_lib/a.mjs', { defs: [{ name: 'aFn', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    ['/fx/_lib/b.mjs', { defs: [{ name: 'bFn', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    ['/fx/_lib/c.mjs', { defs: [{ name: 'cFn', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
  ]);
  const resolves = new Map(files.map((f) => [f, '_lib']));
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn: makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  assert('I4. file cohort has all 5 files even though only 3 have exports (P0-6)',
    result.fileCohorts.get('_lib').members.length === 5);
}

// ═══ I5. Parse error propagates — other files still aggregate ═══

{
  const files = ['/fx/_lib/ok.mjs', '/fx/_lib/broken.mjs'];
  function extractFn(abs) {
    if (abs === '/fx/_lib/broken.mjs') throw new Error('simulated parse error');
    return { defs: [{ name: 'okFn', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] };
  }
  const resolves = new Map(files.map((f) => [f, '_lib']));
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn,
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  assert('I5a. broken file surfaces parse-error diagnostic',
    result.diagnostics.some((d) => d.reason === 'parse-error' &&
      d.target === '_lib/broken.mjs'));
  assert('I5b. ok.mjs helper still aggregated into symbol cohort',
    result.symbolCohorts.get('_lib::helper-export')?.members.length === 1);
  assert('I5c. both files still in file cohort (parse error doesn\'t drop file from inventory)',
    result.fileCohorts.get('_lib').members.length === 2);
}

// ═══ I6. Low-info item surfaces in Outliers (§3) as low-info-excluded ═══

{
  const files = ['/fx/_lib/u.mjs'];
  const perFile = new Map([
    ['/fx/_lib/u.mjs', {
      defs: [
        { name: 'parseJson', kind: 'FunctionDeclaration', line: 1 },
        { name: 'stringifyJson', kind: 'FunctionDeclaration', line: 2 },
        { name: 'fetchData', kind: 'FunctionDeclaration', line: 3 },
        { name: 'renderThing', kind: 'FunctionDeclaration', line: 4 },
        { name: 'get', kind: 'FunctionDeclaration', line: 5 },   // low-info
      ],
      uses: [], reExports: [],
    }],
  ]);
  const resolves = new Map([['/fx/_lib/u.mjs', '_lib']]);
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn: makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  const getRow = result.perItemRows.find((r) => r.name === 'get');
  assert('I6a. `get` appears in perItemRows as low-info-excluded',
    getRow && getRow.itemLabel === 'low-info-excluded');
  assert('I6b. cohort classification = camelCase-dominant (effective 4 of 4)',
    result.symbolCohorts.get('_lib::helper-export').classification.label === 'camelCase-dominant');
}

// ═══ I7. Meta counts ═══

{
  const files = ['/fx/_lib/a.mjs'];
  const perFile = new Map([
    ['/fx/_lib/a.mjs', {
      defs: [
        { name: 'camelCaseFn', kind: 'FunctionDeclaration', line: 1 },
        { name: 'AnotherFn', kind: 'FunctionDeclaration', line: 2 },
        { name: 'YetAnother', kind: 'FunctionDeclaration', line: 3 },
        { name: 'PascalFn', kind: 'FunctionDeclaration', line: 4 },   // outlier
      ],
      uses: [], reExports: [],
    }],
  ]);
  const resolves = new Map([['/fx/_lib/a.mjs', '_lib']]);
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn: makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  assert('I7a. meta.filesScanned === 1',
    result.meta.filesScanned === 1);
  assert('I7b. meta.fileCohortCount === 1',
    result.meta.fileCohortCount === 1);
  assert('I7c. meta.symbolCohortCount === 1',
    result.meta.symbolCohortCount === 1);
}

// ═══ R1. Renderer — all three sections present ═══

{
  const files = ['/fx/_lib/canon-draft.mjs', '/fx/_lib/alias-map.mjs', '/fx/_lib/extract-ts.mjs'];
  const perFile = new Map([
    ['/fx/_lib/canon-draft.mjs', { defs: [
      { name: 'foo', kind: 'FunctionDeclaration', line: 1 },
      { name: 'bar', kind: 'FunctionDeclaration', line: 2 },
      { name: 'baz', kind: 'FunctionDeclaration', line: 3 },
      { name: 'BAD_NAME', kind: 'FunctionDeclaration', line: 4 },
    ], uses: [], reExports: [] }],
    ['/fx/_lib/alias-map.mjs', { defs: [], uses: [], reExports: [] }],
    ['/fx/_lib/extract-ts.mjs', { defs: [], uses: [], reExports: [] }],
  ]);
  const resolves = new Map(files.map((f) => [f, '_lib']));
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn: makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  const md = renderNaming({
    ...result,
    meta: { ...result.meta, scope: 'TS/JS including tests' },
  });
  assert('R1a. §1 File-naming cohorts section present',
    md.includes('## 1. File-naming cohorts') && md.includes('`_lib`'));
  assert('R1b. §2 Symbol-naming cohorts section present',
    md.includes('## 2. Symbol-naming cohorts') && md.includes('`_lib::helper-export`'));
  assert('R1c. §3 Outliers section present with BAD_NAME row',
    md.includes('## 3. Outliers') && md.includes('BAD_NAME'));
  assert('R1d. CohortIdentityShape meta line present',
    md.includes('CohortIdentityShape: submodule | submodule::kind'));
}

// ═══ R2. Renderer — §3 OMITTED when zero outliers + zero low-info ═══

{
  const files = ['/fx/_lib/a.mjs', '/fx/_lib/b.mjs', '/fx/_lib/c.mjs'];
  const perFile = new Map([
    ['/fx/_lib/a.mjs', { defs: [], uses: [], reExports: [] }],
    ['/fx/_lib/b.mjs', { defs: [], uses: [], reExports: [] }],
    ['/fx/_lib/c.mjs', { defs: [], uses: [], reExports: [] }],
  ]);
  const resolves = new Map(files.map((f) => [f, '_lib']));
  const result = collectNamingCohorts({
    files, root: ROOT,
    extractFn: makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(resolves),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  const md = renderNaming({
    ...result,
    meta: { ...result.meta, scope: 'x' },
  });
  assert('R2. zero outliers + zero low-info → §3 OMITTED',
    !md.includes('## 3. Outliers'));
}

// ═══ R3. Renderer — existing-canon header ═══

{
  const result = collectNamingCohorts({
    files: [], root: ROOT,
    extractFn: makeExtractFn(new Map()),
    submoduleOf: makeSubmoduleOf(new Map()),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  const md = renderNaming({
    ...result,
    meta: { ...result.meta, scope: 'x', existingCanon: true },
  });
  assert('R3. existingCanon=true → ⚠ Existing canon detected header for naming.md',
    md.includes('⚠ Existing canon detected') && md.includes('naming.md'));
}

// ═══ R4. Renderer — empty repo still valid ═══

{
  const result = collectNamingCohorts({
    files: [], root: ROOT,
    extractFn: makeExtractFn(new Map()),
    submoduleOf: makeSubmoduleOf(new Map()),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
  const md = renderNaming({
    ...result,
    meta: { ...result.meta, scope: 'x' },
  });
  assert('R4a. empty repo → header + "No file-naming cohorts observed"',
    md.includes('# Naming conventions draft') &&
    md.includes('_No file-naming cohorts observed._'));
  assert('R4b. §3 omitted (zero outliers)',
    !md.includes('## 3. Outliers'));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
