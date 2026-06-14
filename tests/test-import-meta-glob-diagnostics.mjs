import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

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

const fx = mkdtempSync(path.join(tmpdir(), 'fx-import-meta-glob-'));
const out = path.join(fx, 'audit');

try {
  mkdirSync(path.join(fx, 'src/routes'), { recursive: true });
  mkdirSync(path.join(fx, 'src/many'), { recursive: true });
  mkdirSync(out, { recursive: true });

  writeFileSync(path.join(fx, 'package.json'), JSON.stringify({
    name: 'fx-import-meta-glob',
    type: 'module',
  }, null, 2));
  writeFileSync(path.join(fx, 'src/app.ts'),
    "const routes = import.meta.glob('./routes/*.ts');\n" +
    "const missing = import.meta.glob('./missing/*.ts');\n" +
    "const pattern = './routes/*.ts';\n" +
    'const dynamicRoutes = import.meta.glob(pattern);\n' +
    "const many = import.meta.glob('./many/*.ts');\n" +
    'export function routeCount() { return Object.keys(routes).length; }\n');
  writeFileSync(path.join(fx, 'src/routes/home.ts'), 'export const home = true;\n');
  writeFileSync(path.join(fx, 'src/routes/about.ts'), 'export const about = true;\n');
  writeFileSync(path.join(fx, 'src/routes/hidden.ts'), 'export const hidden = true;\n');
  for (let i = 0; i < 65; i++) {
    writeFileSync(path.join(fx, `src/many/route-${i}.ts`), `export const route${i} = true;\n`);
  }

  execFileSync(process.execPath, [
    path.join(DIR, 'skills/lumin-repo-lens-lab/_engine/producers/build-symbol-graph.mjs'),
    '--root', fx,
    '--output', out,
    '--exclude', 'src/routes/hidden.ts',
  ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

  execFileSync(process.execPath, [
    path.join(DIR, 'skills/lumin-repo-lens-lab/_engine/producers/build-resolver-diagnostics.mjs'),
    '--root', fx,
    '--output', out,
  ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

  const syms = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  const diag = JSON.parse(readFileSync(path.join(out, 'resolver-diagnostics.json'), 'utf8'));

  const routeEdges = syms.resolvedInternalEdges
    ?.filter((edge) => edge.source === './routes/*.ts')
    .map((edge) => ({ to: edge.to, kind: edge.kind }))
    .sort((a, b) => a.to.localeCompare(b.to));
  const routeFanInSpace = {
    about: syms.fanInByIdentitySpace?.['src/routes/about.ts::about'],
    home: syms.fanInByIdentitySpace?.['src/routes/home.ts::home'],
  };
  const trulyDeadRoute = syms.deadProdList?.find((entry) =>
    entry.file === 'src/routes/about.ts' || entry.file === 'src/routes/home.ts');
  const hiddenEdge = routeEdges?.find((edge) => edge.to === 'src/routes/hidden.ts');
  const supportedRecord = syms.unresolvedInternalSpecifierRecords?.find((item) =>
    item.specifier === './routes/*.ts');
  const missingRecord = syms.unresolvedInternalSpecifierRecords?.find((item) =>
    item.specifier === './missing/*.ts');
  const nonliteralRecord = syms.unresolvedInternalSpecifierRecords?.find((item) =>
    item.specifier === 'import.meta.glob(<nonliteral>)');
  const cappedRecord = syms.unresolvedInternalSpecifierRecords?.find((item) =>
    item.specifier === './many/*.ts');
  const manyEdges = syms.resolvedInternalEdges?.filter((edge) =>
    edge.source === './many/*.ts');

  const unsupportedImports = Object.fromEntries(
    (diag.unsupportedImports ?? []).map((item) => [item.specifier, item])
  );
  const blindZones = Object.fromEntries(
    (diag.blindZones ?? []).map((item) => [item.specifier, item])
  );

  assert('IMG1. supported literal import.meta.glob expands to concrete dynamic edges',
    routeEdges?.length === 2 &&
      routeEdges[0].to === 'src/routes/about.ts' &&
      routeEdges[1].to === 'src/routes/home.ts' &&
      routeEdges.every((edge) => edge.kind === 'dynamic-import-meta-glob') &&
      !supportedRecord,
    JSON.stringify({ routeEdges, supportedRecord }, null, 2));

  assert('IMG2. import.meta.glob expansion uses the scanned file set and honors excludes',
    !hiddenEdge,
    JSON.stringify({ routeEdges, hiddenEdge }, null, 2));

  assert('IMG3. supported import.meta.glob targets become broad consumers, not true dead exports',
    routeFanInSpace.about?.broad === 1 &&
      routeFanInSpace.home?.broad === 1 &&
      !trulyDeadRoute,
    JSON.stringify({ routeFanInSpace, trulyDeadRoute, deadProdList: syms.deadProdList }, null, 2));

  assert('IMG4. zero-match import.meta.glob remains an unsupported dynamic-module diagnostic',
    missingRecord?.reason === 'import-meta-glob-zero-matches' &&
      missingRecord.resolverStage === 'import-meta-glob' &&
      missingRecord.outputLevel === 'unsupported' &&
      missingRecord.unsupportedFamily === 'dynamic-modules' &&
      missingRecord.matchCount === 0 &&
      missingRecord.affectedPackageScope === 'src/missing',
    JSON.stringify(missingRecord, null, 2));

  assert('IMG5. non-literal import.meta.glob remains unsupported without creating graph edges',
    nonliteralRecord?.reason === 'import-meta-glob-nonliteral-unsupported' &&
      nonliteralRecord.outputLevel === 'unsupported',
    JSON.stringify(nonliteralRecord, null, 2));

  assert('IMG6. cap-exceeded import.meta.glob records evidence and creates no partial edges',
    cappedRecord?.reason === 'import-meta-glob-match-cap-exceeded' &&
      cappedRecord.matchCount === 65 &&
      cappedRecord.cap === 64 &&
      (manyEdges?.length ?? 0) === 0,
    JSON.stringify({ cappedRecord, manyEdges }, null, 2));

  assert('IMG7. resolver diagnostics exposes only unsupported import.meta.glob shapes',
    diag.summary?.unsupportedImportCount === 3 &&
      !unsupportedImports['./routes/*.ts'] &&
      unsupportedImports['./missing/*.ts']?.reason === 'import-meta-glob-zero-matches' &&
      unsupportedImports['import.meta.glob(<nonliteral>)']?.reason === 'import-meta-glob-nonliteral-unsupported' &&
      unsupportedImports['./many/*.ts']?.reason === 'import-meta-glob-match-cap-exceeded',
    JSON.stringify({ summary: diag.summary, unsupportedImports }, null, 2));

  assert('IMG8. unsupported import.meta.glob blind zones stay scoped where scope is known',
    blindZones['./missing/*.ts']?.family === 'dynamic-modules' &&
      blindZones['./missing/*.ts'].blocksAbsenceClaims === true &&
      blindZones['./missing/*.ts'].blockingScope === 'candidate-relevant' &&
      blindZones['./missing/*.ts'].affectedPackageScope === 'src/missing' &&
      blindZones['./many/*.ts']?.affectedPackageScope === 'src/many',
    JSON.stringify(blindZones, null, 2));
} finally {
  rmSync(fx, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
