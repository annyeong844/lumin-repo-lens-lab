import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0;
let failed = 0;

function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

const fx = mkdtempSync(path.join(tmpdir(), 'fx-output-source-layout-'));
const out = path.join(fx, 'audit');

try {
  mkdirSync(path.join(fx, 'apps/web/src'), { recursive: true });
  mkdirSync(path.join(fx, 'packages/weird/main'), { recursive: true });
  mkdirSync(out, { recursive: true });

  writeFileSync(path.join(fx, 'package.json'), JSON.stringify({
    name: 'fx-output-source-layout',
    type: 'module',
    workspaces: ['apps/*', 'packages/*'],
  }, null, 2));
  writeFileSync(path.join(fx, 'apps/web/package.json'), JSON.stringify({
    name: '@fixture/web',
    type: 'module',
    dependencies: {
      '@fixture/weird': 'workspace:*',
    },
  }, null, 2));
  writeFileSync(path.join(fx, 'packages/weird/package.json'), JSON.stringify({
    name: '@fixture/weird',
    type: 'module',
    exports: {
      './*': './compiled/*.js',
    },
  }, null, 2));
  writeFileSync(path.join(fx, 'apps/web/src/app.ts'),
    "import { value } from '@fixture/weird/foo';\n" +
    'export const appValue = value;\n');
  writeFileSync(path.join(fx, 'packages/weird/main/foo.ts'),
    'export const value = 1;\n' +
    'export const sibling = 2;\n');

  execFileSync(process.execPath, [
    path.join(DIR, 'skills/lumin-repo-lens-lab/_engine/producers/build-symbol-graph.mjs'),
    '--root', fx,
    '--output', out,
  ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

  execFileSync(process.execPath, [
    path.join(DIR, 'skills/lumin-repo-lens-lab/_engine/producers/build-resolver-diagnostics.mjs'),
    '--root', fx,
    '--output', out,
  ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

  const syms = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  const diag = JSON.parse(readFileSync(path.join(out, 'resolver-diagnostics.json'), 'utf8'));

  const record = syms.unresolvedInternalSpecifierRecords?.find((item) =>
    item.specifier === '@fixture/weird/foo');
  const unsupportedImport = diag.unsupportedImports?.find((item) =>
    item.specifier === '@fixture/weird/foo');
  const blindZone = diag.blindZones?.find((item) =>
    item.specifier === '@fixture/weird/foo');
  const blockedHint = diag.blockedCandidateHints?.find((item) =>
    item.specifier === '@fixture/weird/foo');
  const graphEdge = syms.resolvedInternalEdges?.find((edge) =>
    edge.source === '@fixture/weird/foo');

  const { buildAliasMap } = await import(`${pathToFileURL(DIR).href}/skills/lumin-repo-lens-lab/_engine/lib/alias-map.mjs`);
  const { makeResolver } = await import(`${pathToFileURL(DIR).href}/skills/lumin-repo-lens-lab/_engine/lib/resolver-core.mjs`);
  const { detectRepoMode } = await import(`${pathToFileURL(DIR).href}/skills/lumin-repo-lens-lab/_engine/lib/repo-mode.mjs`);
  const aliasMap = buildAliasMap(fx, detectRepoMode(fx));
  const resolve = makeResolver(fx, aliasMap);
  const direct = resolve(path.join(fx, 'apps/web/src/app.ts'), '@fixture/weird/foo');

  assert('OSL1. unsupported package output/source layout does not fake a resolved edge',
    direct === 'UNRESOLVED_INTERNAL' && !graphEdge,
    `direct=${direct}\nedge=${JSON.stringify(graphEdge, null, 2)}`);

  assert('OSL2. symbols records unsupported output-to-source mapping diagnostic',
    record?.reason === 'output-source-layout-unsupported' &&
      record.resolverStage === 'wildcard-alias' &&
      record.outputLevel === 'unsupported' &&
      record.unsupportedFamily === 'output-to-source-mapping' &&
      record.source === 'exports' &&
      record.targetCandidates?.includes('packages/weird/compiled/foo.js'),
    JSON.stringify(record, null, 2));

  assert('OSL3. resolver diagnostics exposes output layout in unsupportedImports',
    diag.summary?.unsupportedImportCount === 1 &&
      unsupportedImport?.family === 'output-to-source-mapping' &&
      unsupportedImport.outputLevel === 'unsupported' &&
      unsupportedImport.reason === 'output-source-layout-unsupported',
    JSON.stringify({ summary: diag.summary, unsupportedImport }, null, 2));

  assert('OSL4. output layout blind zone is candidate-scoped, not repo-global',
    blindZone?.family === 'output-to-source-mapping' &&
      blindZone.outputLevel === 'unsupported' &&
      blindZone.blocksAbsenceClaims === true &&
      blindZone.blockingScope === 'candidate-relevant' &&
      blindZone.affectedPackageScope === 'packages/weird',
    JSON.stringify(blindZone, null, 2));

  assert('OSL5. blockedCandidateHints points reviewers at the affected package surface',
    blockedHint?.family === 'output-to-source-mapping' &&
      blockedHint.reason === 'output-source-layout-unsupported' &&
      blockedHint.affectedPackageScope === 'packages/weird' &&
      blockedHint.candidatePath === 'packages/weird/compiled/foo.js',
    JSON.stringify(blockedHint, null, 2));
} finally {
  rmSync(fx, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
