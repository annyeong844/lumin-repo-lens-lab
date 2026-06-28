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

const fx = mkdtempSync(path.join(tmpdir(), 'fx-node-imports-unsupported-'));
const out = path.join(fx, 'audit');

try {
  mkdirSync(path.join(fx, 'src'), { recursive: true });
  mkdirSync(out, { recursive: true });

  writeFileSync(path.join(fx, 'package.json'), JSON.stringify({
    name: 'fx-node-imports-unsupported',
    type: 'module',
  }, null, 2));
  writeFileSync(path.join(fx, 'src/app.ts'),
    "import { config } from '#app/config';\n" +
    'export function boot() { return config; }\n');

  execFileSync(process.execPath, [
    path.join(DIR, 'build-symbol-graph.mjs'),
    '--root', fx,
    '--output', out,
  ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

  execFileSync(process.execPath, [
    path.join(DIR, 'build-resolver-diagnostics.mjs'),
    '--root', fx,
    '--output', out,
  ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

  const syms = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  const diag = JSON.parse(readFileSync(path.join(out, 'resolver-diagnostics.json'), 'utf8'));

  const record = syms.unresolvedInternalSpecifierRecords?.find((item) =>
    item.specifier === '#app/config');
  const unresolvedImport = diag.unresolvedImports?.find((item) =>
    item.specifier === '#app/config');
  const unsupportedImport = diag.unsupportedImports?.find((item) =>
    item.specifier === '#app/config');
  const blindZone = diag.blindZones?.find((item) =>
    item.specifier === '#app/config');
  const graphEdge = syms.resolvedInternalEdges?.find((edge) =>
    edge.source === '#app/config');

  const { buildAliasMap } = await import(`${pathToFileURL(DIR).href}/_lib/alias-map.mjs`);
  const { makeResolver } = await import(`${pathToFileURL(DIR).href}/_lib/resolver-core.mjs`);
  const { detectRepoMode } = await import(`${pathToFileURL(DIR).href}/_lib/repo-mode.mjs`);
  const aliasMap = buildAliasMap(fx, detectRepoMode(fx));
  const resolve = makeResolver(fx, aliasMap);
  const direct = resolve(path.join(fx, 'src/app.ts'), '#app/config');

  assert('NI1. package-local #imports without a supported imports map returns UNRESOLVED_INTERNAL',
    direct === 'UNRESOLVED_INTERNAL',
    `got: ${direct}`);

  assert('NI2. symbols.json records Node #imports as explicit unsupported diagnostic',
    record?.reason === 'hash-imports-unsupported' &&
      record.resolverStage === 'hash-imports' &&
      record.outputLevel === 'unsupported' &&
      record.unsupportedFamily === 'node-imports',
    JSON.stringify(record, null, 2));

  assert('NI3. unsupported #imports creates no concrete graph edge',
    !graphEdge &&
      syms.uses?.resolvedInternal === 0 &&
      syms.uses?.unresolvedInternal === 1,
    `edge=${JSON.stringify(graphEdge)}\nuses=${JSON.stringify(syms.uses)}`);

  assert('NI4. resolver-diagnostics preserves unsupported output level in unresolvedImports',
    unresolvedImport?.family === 'node-imports' &&
      unresolvedImport.reason === 'hash-imports-unsupported' &&
      unresolvedImport.outputLevel === 'unsupported' &&
      unresolvedImport.createsGraphEdge === false,
    JSON.stringify(unresolvedImport, null, 2));

  assert('NI5. resolver-diagnostics exposes a dedicated unsupportedImports lane',
    diag.summary?.unsupportedImportCount === 1 &&
      unsupportedImport?.family === 'node-imports' &&
      unsupportedImport.outputLevel === 'unsupported' &&
      unsupportedImport.reason === 'hash-imports-unsupported',
    JSON.stringify({ summary: diag.summary, unsupportedImport }, null, 2));

  assert('NI6. unsupported #imports blind zone is confidence-limited, not candidate proof',
    blindZone?.family === 'node-imports' &&
      blindZone.outputLevel === 'unsupported' &&
      blindZone.blocksAbsenceClaims === true &&
      blindZone.blockingScope === 'repo-confidence-limited' &&
      !blindZone.targetCandidates,
    JSON.stringify(blindZone, null, 2));
} finally {
  rmSync(fx, { recursive: true, force: true });
}

{
  const ambiguousFx = mkdtempSync(path.join(tmpdir(), 'fx-node-imports-condition-ambiguous-'));
  const ambiguousOut = path.join(ambiguousFx, 'audit');

  try {
    mkdirSync(path.join(ambiguousFx, 'src'), { recursive: true });
    mkdirSync(ambiguousOut, { recursive: true });

    writeFileSync(path.join(ambiguousFx, 'package.json'), JSON.stringify({
      name: 'fx-node-imports-condition-ambiguous',
      type: 'module',
      imports: {
        '#env': {
          browser: './src/browser.ts',
          'react-native': './src/native.ts',
        },
      },
    }, null, 2));
    writeFileSync(path.join(ambiguousFx, 'src/browser.ts'), 'export const env = "browser";\n');
    writeFileSync(path.join(ambiguousFx, 'src/native.ts'), 'export const env = "native";\n');
    writeFileSync(path.join(ambiguousFx, 'src/app.ts'),
      "import { env } from '#env';\n" +
      'export function boot() { return env; }\n');

    execFileSync(process.execPath, [
      path.join(DIR, 'build-symbol-graph.mjs'),
      '--root', ambiguousFx,
      '--output', ambiguousOut,
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    execFileSync(process.execPath, [
      path.join(DIR, 'build-resolver-diagnostics.mjs'),
      '--root', ambiguousFx,
      '--output', ambiguousOut,
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(ambiguousOut, 'symbols.json'), 'utf8'));
    const diag = JSON.parse(readFileSync(path.join(ambiguousOut, 'resolver-diagnostics.json'), 'utf8'));
    const record = syms.unresolvedInternalSpecifierRecords?.find((item) =>
      item.specifier === '#env');
    const unsupportedImport = diag.unsupportedImports?.find((item) =>
      item.specifier === '#env');
    const blindZone = diag.blindZones?.find((item) =>
      item.specifier === '#env');
    const graphEdge = syms.resolvedInternalEdges?.find((edge) =>
      edge.source === '#env');

    const { buildAliasMap } = await import(`${pathToFileURL(DIR).href}/_lib/alias-map.mjs`);
    const { makeResolver } = await import(`${pathToFileURL(DIR).href}/_lib/resolver-core.mjs`);
    const { detectRepoMode } = await import(`${pathToFileURL(DIR).href}/_lib/repo-mode.mjs`);
    const aliasMap = buildAliasMap(ambiguousFx, detectRepoMode(ambiguousFx));
    const resolve = makeResolver(ambiguousFx, aliasMap);
    const direct = resolve(path.join(ambiguousFx, 'src/app.ts'), '#env');

    assert('NI7. unsupported condition-profile #imports map returns UNRESOLVED_INTERNAL',
      direct === 'UNRESOLVED_INTERNAL',
      `got: ${direct}`);

    assert('NI8. symbols.json records condition-profile ambiguity as unsupported node-imports',
      record?.reason === 'condition-profile-ambiguous' &&
        record.resolverStage === 'hash-imports' &&
        record.outputLevel === 'unsupported' &&
        record.unsupportedFamily === 'node-imports' &&
        record.targetCandidates?.includes('src/browser.ts') &&
        record.targetCandidates?.includes('src/native.ts'),
      JSON.stringify(record, null, 2));

    assert('NI9. ambiguous #imports condition map creates no concrete graph edge',
      !graphEdge &&
        syms.uses?.resolvedInternal === 0 &&
        syms.uses?.unresolvedInternal === 1,
      `edge=${JSON.stringify(graphEdge)}\nuses=${JSON.stringify(syms.uses)}`);

    assert('NI10. resolver-diagnostics exposes condition-profile ambiguity in unsupportedImports',
      diag.summary?.unsupportedImportCount === 1 &&
        unsupportedImport?.family === 'node-imports' &&
        unsupportedImport.outputLevel === 'unsupported' &&
        unsupportedImport.reason === 'condition-profile-ambiguous' &&
        unsupportedImport.targetCandidates?.includes('src/browser.ts') &&
        unsupportedImport.targetCandidates?.includes('src/native.ts'),
      JSON.stringify({ summary: diag.summary, unsupportedImport }, null, 2));

    assert('NI11. ambiguous #imports blind zone stays diagnostic-only with candidate relevance',
      blindZone?.family === 'node-imports' &&
        blindZone.reason === 'condition-profile-ambiguous' &&
        blindZone.outputLevel === 'unsupported' &&
        blindZone.blocksAbsenceClaims === true &&
        blindZone.blockingScope === 'candidate-relevant' &&
        blindZone.targetCandidates?.includes('src/browser.ts') &&
        blindZone.targetCandidates?.includes('src/native.ts'),
      JSON.stringify(blindZone, null, 2));
  } finally {
    rmSync(ambiguousFx, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
