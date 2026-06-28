// PCEF P2b: entry-surface.json records entry files separately from reachability.

import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

function writeFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'entry-surface-fixture',
      private: true,
      exports: {
        '.': './src/index.ts',
      },
      scripts: {
        build: 'tsup src/cli.ts',
      },
      dependencies: {
        next: '15.0.0',
      },
    }, null, 2),
    'index.html': '<script type="module" src="./src/browser.ts"></script>\n',
    'vite.config.ts': 'export default { plugins: [] };\n',
    'src/index.ts': 'export { feature } from "./feature";\n',
    'src/feature.ts': 'export const feature = 1;\n',
    'src/cli.ts': 'export function cli() {}\n',
    'src/browser.ts': 'export const browser = true;\n',
    'src/internal.ts': 'export const internal = true;\n',
    'src/app/dashboard/page.tsx': 'export default function Page() { return null; }\n',
    'cloudflare/worker/wrangler.toml': 'main = "src/index.js"\n',
    'cloudflare/worker/src/index.js': 'export default { async fetch() { return new Response("ok"); } };\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makeFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-entry-surface-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writeFixture(dir);
  return { dir, out };
}

function writeStaticRootMismatchFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'entry-surface-static-root-mismatch',
      private: true,
      type: 'module',
    }, null, 2),
    'index.html': '<script type="module" src="/assets/app.js"></script>\n',
    'server.ts': [
      'import path from "node:path";',
      'export const STATIC_ROOT = path.join(process.cwd(), "public");',
    ].join('\n'),
    'public/assets/app.js': [
      'import { boot } from "../../src/boot.js";',
      'export function createTerminalInputMessage(value) {',
      '  return { type: "input", value };',
      '}',
      'export function serializeTerminalInputMessage(value) {',
      '  return JSON.stringify(createTerminalInputMessage(value));',
      '}',
      'boot();',
      'serializeTerminalInputMessage("hello");',
    ].join('\n'),
    'src/boot.js': 'export function boot() {}\n',
    'src/unused.js': 'export const unused = true;\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makeStaticRootMismatchFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-entry-static-mismatch-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writeStaticRootMismatchFixture(dir);
  return { dir, out };
}

function writeNestedHtmlAppFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'entry-surface-nested-html-app',
      private: true,
      type: 'module',
    }, null, 2),
    'apps/web/index.html': '<script type="module" src="/src/main.tsx"></script>\n',
    'apps/web/src/main.tsx': 'export function mountApp() {}\n',
    'src/main.tsx': 'export function wrongRoot() {}\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makeNestedHtmlAppFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-entry-nested-html-app-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writeNestedHtmlAppFixture(dir);
  return { dir, out };
}

function writeExcludedHtmlFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'entry-surface-excluded-html',
      private: true,
      type: 'module',
    }, null, 2),
    'src/main.ts': 'export const main = 1;\n',
    'output/corpus/sample/index.html':
      '<script type="module" src="/src/missing.ts"></script>\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makeExcludedHtmlFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-entry-excluded-html-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writeExcludedHtmlFixture(dir);
  return { dir, out };
}

function writeRuntimeScriptFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'entry-surface-runtime-script',
      private: true,
      type: 'module',
      scripts: {
        start: 'tsx src/server.ts',
      },
    }, null, 2),
    'src/server.ts': [
      'import { app } from "./app";',
      'app.listen();',
    ].join('\n'),
    'src/app.ts': 'export const app = { listen() {} };\n',
    'src/isolated.ts': 'export const isolated = true;\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makeRuntimeScriptFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-entry-runtime-script-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writeRuntimeScriptFixture(dir);
  return { dir, out };
}

function writeUnknownScriptWrapperFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'entry-surface-unknown-script-wrapper',
      private: true,
      type: 'module',
      scripts: {
        start: 'custom-runner src/server.ts',
      },
    }, null, 2),
    'src/server.ts': 'export function listen() {}\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makeUnknownScriptWrapperFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-entry-unknown-script-wrapper-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writeUnknownScriptWrapperFixture(dir);
  return { dir, out };
}

function writePackageScriptRecursionFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'entry-surface-script-recursion',
      private: true,
      type: 'module',
      scripts: {
        start: 'npm run server',
      },
    }, null, 2),
    'src/server.ts': 'export function listen() {}\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makePackageScriptRecursionFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-entry-script-recursion-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writePackageScriptRecursionFixture(dir);
  return { dir, out };
}

function writeRuntimeScriptArgvFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'entry-surface-runtime-script-argv',
      private: true,
      type: 'module',
      scripts: {
        start: 'node src/main.ts src/config.ts',
      },
    }, null, 2),
    'src/main.ts': 'export function main() {}\n',
    'src/config.ts': 'export const config = {};\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makeRuntimeScriptArgvFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-entry-runtime-script-argv-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writeRuntimeScriptArgvFixture(dir);
  return { dir, out };
}

function runEntrySurface() {
  const { dir, out } = makeFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-entry-surface.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    return JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runAuditRepo() {
  const { dir, out } = makeFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'audit-repo.mjs'),
      '--root', dir,
      '--output', out,
      '--profile', 'quick',
      '--production',
    ], { encoding: 'utf8' });
    return {
      manifest: JSON.parse(readFileSync(path.join(out, 'manifest.json'), 'utf8')),
      entrySurface: JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8')),
    };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runStaticRootMismatchAudit() {
  const { dir, out } = makeStaticRootMismatchFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'audit-repo.mjs'),
      '--root', dir,
      '--output', out,
      '--profile', 'quick',
      '--production',
    ], { encoding: 'utf8' });
    return {
      manifest: JSON.parse(readFileSync(path.join(out, 'manifest.json'), 'utf8')),
      entrySurface: JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8')),
      reachability: JSON.parse(readFileSync(path.join(out, 'module-reachability.json'), 'utf8')),
      fixPlan: JSON.parse(readFileSync(path.join(out, 'fix-plan.json'), 'utf8')),
    };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runNestedHtmlAppEntrySurface() {
  const { dir, out } = makeNestedHtmlAppFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-entry-surface.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    return JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runExcludedHtmlEntrySurface() {
  const { dir, out } = makeExcludedHtmlFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--exclude', 'output/corpus',
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-entry-surface.mjs'),
      '--root', dir,
      '--output', out,
      '--exclude', 'output/corpus',
      '--production',
    ], { encoding: 'utf8' });

    return JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runRuntimeScriptAudit() {
  const { dir, out } = makeRuntimeScriptFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-entry-surface.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-module-reachability.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    return {
      entrySurface: JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8')),
      reachability: JSON.parse(readFileSync(path.join(out, 'module-reachability.json'), 'utf8')),
    };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runUnknownScriptWrapperEntrySurface() {
  const { dir, out } = makeUnknownScriptWrapperFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-entry-surface.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    return JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runPackageScriptRecursionEntrySurface() {
  const { dir, out } = makePackageScriptRecursionFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-entry-surface.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    return JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runRuntimeScriptArgvEntrySurface() {
  const { dir, out } = makeRuntimeScriptArgvFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-entry-surface.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    return JSON.parse(readFileSync(path.join(out, 'entry-surface.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

const artifact = runEntrySurface();
const asSet = (value) => new Set(value ?? []);
const publicApiFiles = asSet(artifact.publicApiFiles);
const scriptEntrypointFiles = asSet(artifact.scriptEntrypointFiles);
const htmlEntrypointFiles = asSet(artifact.htmlEntrypointFiles);
const frameworkEntrypointFiles = asSet(artifact.frameworkEntrypointFiles);
const configEntrypointFiles = asSet(artifact.configEntrypointFiles);
const entryFiles = asSet(artifact.entryFiles);

assert('E1. artifact meta names build-entry-surface.mjs',
  artifact.meta?.tool === 'build-entry-surface.mjs',
  JSON.stringify(artifact.meta));
assert('E2. publicApiFiles includes package export root',
  publicApiFiles.has('src/index.ts'),
  JSON.stringify(artifact.publicApiFiles));
assert('E3. publicApiFiles includes transitive public re-export target',
  publicApiFiles.has('src/feature.ts'),
  JSON.stringify(artifact.publicApiFiles));
assert('E4. scriptEntrypointFiles includes script-driven source entry',
  scriptEntrypointFiles.has('src/cli.ts'),
  JSON.stringify(artifact.scriptEntrypointFiles));
assert('E5. htmlEntrypointFiles includes module script target',
  htmlEntrypointFiles.has('src/browser.ts'),
  JSON.stringify(artifact.htmlEntrypointFiles));
assert('E6. frameworkEntrypointFiles includes Next app route file',
  frameworkEntrypointFiles.has('src/app/dashboard/page.tsx'),
  JSON.stringify(artifact.frameworkEntrypointFiles));
assert('E6b. frameworkEntrypointFiles includes Cloudflare Worker default export from wrangler scope',
  frameworkEntrypointFiles.has('cloudflare/worker/src/index.js'),
  JSON.stringify(artifact.frameworkEntrypointFiles));
assert('E7. configEntrypointFiles includes tool config',
  configEntrypointFiles.has('vite.config.ts'),
  JSON.stringify(artifact.configEntrypointFiles));
assert('E8. entryFiles is the union and excludes ordinary internals',
  entryFiles.has('src/index.ts') &&
    entryFiles.has('src/feature.ts') &&
    entryFiles.has('src/cli.ts') &&
    entryFiles.has('src/browser.ts') &&
    entryFiles.has('src/app/dashboard/page.tsx') &&
    entryFiles.has('vite.config.ts') &&
    !entryFiles.has('src/internal.ts'),
  JSON.stringify(artifact.entryFiles));
assert('E9. evidenceByFile preserves public re-export evidence',
  artifact.evidenceByFile?.['src/feature.ts']?.some((e) => e.source === 'public-reexport'),
  JSON.stringify(artifact.evidenceByFile?.['src/feature.ts']));
assert('E10. globalCompleteness is high for a clean fixture',
  artifact.globalCompleteness === 'high',
  JSON.stringify({ globalCompleteness: artifact.globalCompleteness, meta: artifact.meta }));
assert('E11. completenessBySubmodule carries high local labels',
  Object.values(artifact.completenessBySubmodule ?? {}).every((value) => value === 'high') &&
    artifact.completenessBySubmodule?.root === 'high' &&
    artifact.completenessBySubmodule?.src === 'high',
  JSON.stringify(artifact.completenessBySubmodule));

const audit = runAuditRepo();
assert('E12. audit-repo quick profile runs build-entry-surface.mjs',
  audit.manifest.commandsRun?.some((step) =>
    step.step === 'build-entry-surface.mjs' && step.status === 'ok'),
  JSON.stringify(audit.manifest.commandsRun));
assert('E13. audit-repo artifactsProduced lists entry-surface.json',
  audit.manifest.artifactsProduced?.includes('entry-surface.json'),
  JSON.stringify(audit.manifest.artifactsProduced));
assert('E14. pipeline entry-surface artifact keeps public API evidence',
  audit.entrySurface.publicApiFiles?.includes('src/feature.ts'),
  JSON.stringify(audit.entrySurface.publicApiFiles));

const staticMismatch = runStaticRootMismatchAudit();
assert('E15. absolute HTML module path is not promoted when the repo path is missing',
  !staticMismatch.entrySurface.htmlEntrypointFiles?.includes('assets/app.js') &&
    !staticMismatch.reachability.reachableFiles?.includes('assets/app.js'),
  JSON.stringify({
    htmlEntrypointFiles: staticMismatch.entrySurface.htmlEntrypointFiles,
    reachableFiles: staticMismatch.reachability.reachableFiles,
  }));
assert('E16. missing HTML module target records an unresolved entry-surface diagnostic',
  staticMismatch.entrySurface.unresolvedHtmlEntrypoints?.some((entry) =>
    entry.htmlFile === 'index.html' &&
    entry.src === '/assets/app.js' &&
    entry.reason === 'html-module-script-target-missing'),
  JSON.stringify(staticMismatch.entrySurface.unresolvedHtmlEntrypoints));
assert('E17. missing HTML module target lowers entry-surface completeness',
  staticMismatch.entrySurface.globalCompleteness === 'medium',
  JSON.stringify({
    globalCompleteness: staticMismatch.entrySurface.globalCompleteness,
    completenessBySubmodule: staticMismatch.entrySurface.completenessBySubmodule,
  }));
assert('E18. manifest reports an HTML entry-surface blind zone',
  staticMismatch.manifest.blindZones?.some((zone) =>
    zone.area === 'html-entry-surface' &&
    zone.details?.unresolvedHtmlEntrypoints === 1),
  JSON.stringify(staticMismatch.manifest.blindZones));
assert('E19. HTML entry-surface blind zone prevents SAFE_FIX on matching static asset exports',
  !staticMismatch.fixPlan.safeFixes?.some((score) =>
    score.finding?.file === 'public/assets/app.js'),
  JSON.stringify(staticMismatch.fixPlan.safeFixes));
assert('E20. matching static asset exports remain review-visible with blocked promotion details',
  staticMismatch.fixPlan.reviewFixes?.filter((score) =>
    score.finding?.file === 'public/assets/app.js' &&
    score.reason === 'html-entry-surface-blind-zone' &&
    score.blockedPromotion === true &&
    score.blockedBy?.[0]?.area === 'html-entry-surface').length === 2,
  JSON.stringify(staticMismatch.fixPlan.reviewFixes));

const nestedHtmlApp = runNestedHtmlAppEntrySurface();
assert('E21. nested HTML app root resolves absolute module script relative to HTML directory',
  nestedHtmlApp.htmlEntrypointFiles?.includes('apps/web/src/main.tsx') &&
    !nestedHtmlApp.htmlEntrypointFiles?.includes('src/main.tsx'),
  JSON.stringify(nestedHtmlApp.htmlEntrypointFiles));
assert('E22. nested HTML app root does not emit phantom extension-probe entries',
  !nestedHtmlApp.htmlEntrypointFiles?.includes('apps/web/src/main.jsx') &&
    !nestedHtmlApp.entryFiles?.includes('apps/web/src/main.jsx') &&
    nestedHtmlApp.evidenceByFile?.['apps/web/src/main.jsx'] === undefined,
  JSON.stringify({
    htmlEntrypointFiles: nestedHtmlApp.htmlEntrypointFiles,
    entryFiles: nestedHtmlApp.entryFiles,
    evidenceByFile: nestedHtmlApp.evidenceByFile?.['apps/web/src/main.jsx'],
  }));
assert('E23. nested HTML app root does not create an unresolved HTML blind zone',
  (nestedHtmlApp.unresolvedHtmlEntrypoints ?? []).length === 0 &&
    nestedHtmlApp.globalCompleteness === 'high',
  JSON.stringify({
    unresolvedHtmlEntrypoints: nestedHtmlApp.unresolvedHtmlEntrypoints,
    globalCompleteness: nestedHtmlApp.globalCompleteness,
  }));

const excludedHtml = runExcludedHtmlEntrySurface();
assert('E24. excluded HTML files do not create unresolved entry-surface blind zones',
  (excludedHtml.unresolvedHtmlEntrypoints ?? []).length === 0 &&
    excludedHtml.globalCompleteness === 'high',
  JSON.stringify({
    unresolvedHtmlEntrypoints: excludedHtml.unresolvedHtmlEntrypoints,
    globalCompleteness: excludedHtml.globalCompleteness,
  }));

const runtimeScript = runRuntimeScriptAudit();
assert('E25. package runtime scripts seed module reachability',
  runtimeScript.entrySurface.scriptEntrypointFiles?.includes('src/server.ts') &&
    runtimeScript.entrySurface.entryFiles?.includes('src/server.ts') &&
    runtimeScript.entrySurface.evidenceByFile?.['src/server.ts']?.some((entry) =>
      entry.source === 'package.scripts' &&
      entry.scriptName === 'start' &&
      entry.tool === 'tsx' &&
      entry.runtime === true) &&
    runtimeScript.reachability.runtimeReachableFiles?.includes('src/server.ts') &&
    !runtimeScript.reachability.unreachableFiles?.includes('src/server.ts') &&
    runtimeScript.reachability.unreachableFiles?.includes('src/isolated.ts'),
  JSON.stringify(runtimeScript));

const unknownScriptWrapper = runUnknownScriptWrapperEntrySurface();
assert('E26. unknown script wrappers do not create runtime entry evidence',
  !unknownScriptWrapper.scriptEntrypointFiles?.includes('src/server.ts') &&
    !unknownScriptWrapper.entryFiles?.includes('src/server.ts') &&
    unknownScriptWrapper.evidenceByFile?.['src/server.ts'] === undefined &&
    unknownScriptWrapper.meta?.supports?.unsupportedScriptEntrypoints === true &&
    unknownScriptWrapper.unsupportedScriptEntrypointCount === 1 &&
    unknownScriptWrapper.unsupportedScriptEntrypointSampleLimit === 50 &&
    unknownScriptWrapper.unsupportedScriptEntrypoints?.some((entry) =>
      entry.reason === 'unknown-script-wrapper' &&
      entry.scriptName === 'start' &&
      entry.tool === 'custom-runner' &&
      entry.targetCandidates?.includes('src/server.ts') &&
      entry.confidence === 'advisory'),
  JSON.stringify(unknownScriptWrapper));

const runtimeScriptArgv = runRuntimeScriptArgvEntrySurface();
assert('E27. runtime script argv tokens do not become entry evidence',
  runtimeScriptArgv.scriptEntrypointFiles?.includes('src/main.ts') &&
    !runtimeScriptArgv.scriptEntrypointFiles?.includes('src/config.ts') &&
    runtimeScriptArgv.entryFiles?.includes('src/main.ts') &&
    !runtimeScriptArgv.entryFiles?.includes('src/config.ts') &&
    runtimeScriptArgv.evidenceByFile?.['src/config.ts'] === undefined,
  JSON.stringify(runtimeScriptArgv));

const packageScriptRecursion = runPackageScriptRecursionEntrySurface();
assert('E28. package script wrappers record scoped unsupported diagnostics without entry evidence',
  !packageScriptRecursion.scriptEntrypointFiles?.includes('src/server.ts') &&
    !packageScriptRecursion.entryFiles?.includes('src/server.ts') &&
    packageScriptRecursion.unsupportedScriptEntrypointCount === 1 &&
    packageScriptRecursion.unsupportedScriptEntrypoints?.some((entry) =>
      entry.reason === 'package-script-recursion-unsupported' &&
      entry.scriptName === 'start' &&
      entry.tool === 'npm' &&
      entry.targetScript === 'server' &&
      entry.confidence === 'advisory'),
  JSON.stringify(packageScriptRecursion));

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
