import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { detectRepoMode } from '../_lib/repo-mode.mjs';
import {
  collectHtmlModuleEntrypointFiles,
  collectPackagePublicSurfaceFiles,
  collectScriptEntrypointFiles,
  collectScriptEntrypoints,
} from '../_lib/public-surface.mjs';

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else { failed++; console.error(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function collect(root) {
  return collectPackagePublicSurfaceFiles({
    root,
    repoMode: detectRepoMode(root),
  });
}

function collectScripts(root) {
  return collectScriptEntrypointFiles({
    root,
    repoMode: detectRepoMode(root),
  });
}

function collectScriptSurface(root) {
  return collectScriptEntrypoints({
    root,
    repoMode: detectRepoMode(root),
  });
}

function collectHtml(root) {
  return collectHtmlModuleEntrypointFiles({
    root,
    repoMode: detectRepoMode(root),
  });
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-root-'));
  try {
    write(fx, 'pnpm-workspace.yaml', 'packages:\n  - examples/*\n');
    write(fx, 'package.json', JSON.stringify({
      name: 'root-public',
      type: 'module',
      exports: {
        '.': {
          types: './dist/index.d.ts',
          default: './dist/index.js',
        },
        './types': {
          types: './types/index.d.ts',
        },
      },
      main: './dist/index.cjs',
      module: './dist/index.js',
      types: './dist/index.d.ts',
      bin: { 'root-public': './dist/cli.js' },
    }));
    write(fx, 'src/index.ts', 'export const publicValue = 1;\n');
    write(fx, 'src/cli.ts', 'export const cli = 1;\n');
    write(fx, 'types/index.d.ts', 'export interface PublicOptions {}\n');
    write(fx, 'examples/app/package.json', JSON.stringify({ name: 'example-app' }));

    const files = new Set(collect(fx).map((e) => e.file));

    assert('PS-1a. monorepo root package contributes public surface entries',
      files.has('src/index.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-1b. direct declaration subpath stays public as its own .d.ts file',
      files.has('types/index.d.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-1c. top-level bin targets are mapped back to source files',
      files.has('src/cli.ts'),
      `files=${[...files].sort().join(', ')}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-bare-fields-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'bare-field-targets',
      type: 'module',
      main: 'server.js',
      types: 'types/index.d.ts',
      bin: { 'bare-field-targets': 'bin/cli.js' },
    }));
    write(fx, 'server.js', 'export const server = 1;\n');
    write(fx, 'bin/cli.js', 'export const cli = 1;\n');
    write(fx, 'types/index.d.ts', 'export interface PublicTypes {}\n');

    const files = new Set(collect(fx).map((e) => e.file));

    assert('PS-1d. bare package main target is treated as package-relative file',
      files.has('server.js'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-1e. bare package bin target is treated as package-relative file',
      files.has('bin/cli.js'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-1f. bare package types target is treated as package-relative file',
      files.has('types/index.d.ts'),
      `files=${[...files].sort().join(', ')}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-conditions-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'all-conditions',
      type: 'module',
      exports: {
        '.': {
          import: {
            types: './dist/import.d.ts',
            default: './dist/import.js',
          },
          require: {
            types: './dist/require.d.cts',
            default: './dist/require.cjs',
          },
        },
      },
    }));
    write(fx, 'src/import.ts', 'export const importPublic = 1;\n');
    write(fx, 'src/require.ts', 'export const requirePublic = 1;\n');

    const entries = collect(fx);
    const files = new Set(entries.map((e) => e.file));
    const importEvidence = entries.find((e) => e.file === 'src/import.ts')?.evidence ?? {};
    const requireEvidence = entries.find((e) => e.file === 'src/require.ts')?.evidence ?? {};

    assert('PS-2a. exports import.default condition is collected',
      files.has('src/import.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-2b. exports require.default condition is collected',
      files.has('src/require.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-2c. evidence records the package.exports condition path',
      importEvidence.conditionPath === 'import.types' &&
        requireEvidence.conditionPath === 'require.types',
      `import=${JSON.stringify(importEvidence)}, require=${JSON.stringify(requireEvidence)}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-dist-source-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'dist-source',
      type: 'module',
      exports: {
        '.': {
          types: './dist/index.d.ts',
          import: './dist/index.js',
        },
      },
    }));
    write(fx, 'dist/index.js', 'export const compiled = 1;\n');
    write(fx, 'dist/index.d.ts', 'export declare const compiled: number;\n');
    write(fx, 'src/index.ts', 'export const authored = 1;\n');

    const entries = collect(fx);
    const files = new Set(entries.map((e) => e.file));
    const sourceEvidence = entries.find((e) => e.file === 'src/index.ts')?.evidence ?? {};

    assert('PS-2d. dist targets prefer authored source when both exist',
      files.has('src/index.ts') && !files.has('dist/index.js') && !files.has('dist/index.d.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-2e. dist/source evidence keeps the original package target',
      sourceEvidence.target === './dist/index.d.ts' ||
        sourceEvidence.target === './dist/index.js',
      JSON.stringify(sourceEvidence));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-wildcard-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'wild-public',
      type: 'module',
      exports: {
        './features/*': './src/features/*.ts',
      },
    }));
    write(fx, 'src/features/alpha.ts', 'export const alpha = 1;\n');
    write(fx, 'src/features/beta.ts', 'export const beta = 1;\n');
    write(fx, 'src/private.ts', 'export const privateValue = 1;\n');

    const entries = collect(fx);
    const files = new Set(entries.map((e) => e.file));
    const alphaEvidence = entries.find((e) => e.file === 'src/features/alpha.ts')?.evidence ?? {};

    assert('PS-2f. exports wildcard subpath expands public surface files',
      files.has('src/features/alpha.ts') &&
        files.has('src/features/beta.ts') &&
        !files.has('src/private.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-2g. wildcard public-surface evidence records source pattern',
      alphaEvidence.source === 'package.exports' &&
        alphaEvidence.subpath === './features/*' &&
        alphaEvidence.sourcePattern === 'src/features/*.ts' &&
        alphaEvidence.wildcard === true,
      JSON.stringify(alphaEvidence));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-wildcard-js-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'wild-public-js',
      type: 'module',
      exports: {
        './features/*': './src/features/*.js',
      },
    }));
    write(fx, 'src/features/alpha.js', 'export const alpha = 1;\n');
    write(fx, 'src/features/beta.js', 'export const beta = 1;\n');
    write(fx, 'src/features/gamma.ts', 'export const gamma = 1;\n');
    write(fx, 'src/private.js', 'export const privateValue = 1;\n');

    const entries = collect(fx);
    const files = new Set(entries.map((e) => e.file));
    const alphaEvidence = entries.find((e) => e.file === 'src/features/alpha.js')?.evidence ?? {};

    assert('PS-2h. exports wildcard keeps authored JS source files public',
      files.has('src/features/alpha.js') &&
        files.has('src/features/beta.js') &&
        files.has('src/features/gamma.ts') &&
        !files.has('src/private.js'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-2i. JS wildcard evidence records the matching JS source pattern',
      alphaEvidence.source === 'package.exports' &&
        alphaEvidence.subpath === './features/*' &&
        alphaEvidence.sourcePattern === 'src/features/*.js' &&
        alphaEvidence.wildcard === true,
      JSON.stringify(alphaEvidence));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-scripts-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'script-entrypoints',
      type: 'module',
      scripts: {
        build: 'rimraf dist && esno scripts/build.ts',
        bundle: 'tsup src/direct.ts --format esm',
      },
    }));
    write(fx, 'src/direct.ts', 'export const direct = 1;\n');
    write(fx, 'src/client/dev/react.ts', 'export const useRegisterSW = () => null;\n');
    write(fx, 'src/not-entry.ts', 'export const notEntry = 1;\n');
    write(fx, 'scripts/build.ts',
      `export const commands = [\n` +
      `  'npx tsup src/client/dev/react.ts --external react --target esnext',\n` +
      `  'this mentions src/not-entry.ts but is not a tsup command',\n` +
      `]\n`);

    const entries = collectScripts(fx);
    const scriptSurface = collectScriptSurface(fx);
    const files = new Set(entries.map((e) => e.file));
    const reactEvidence = entries.find((e) => e.file === 'src/client/dev/react.ts')?.evidence ?? {};

    assert('PS-3a. package script tsup entrypoint is collected',
      files.has('src/direct.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-3b. scripts/*.ts string literal tsup entrypoint is collected',
      files.has('src/client/dev/react.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-3c. non-command string mentions are ignored',
      !files.has('src/not-entry.ts') &&
        !scriptSurface.unsupported.some((entry) =>
          entry.source === 'script-string-literal' ||
          entry.targetCandidates?.includes('src/not-entry.ts')),
      `files=${[...files].sort().join(', ')}, unsupported=${JSON.stringify(scriptSurface.unsupported)}`);
    assert('PS-3d. script-entrypoint evidence records source file and tool',
      reactEvidence.source === 'script-string-literal' &&
        reactEvidence.scriptFile === 'scripts/build.ts' &&
        reactEvidence.tool === 'tsup',
      JSON.stringify(reactEvidence));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-rollup-esbuild-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'script-entrypoint-tools',
      type: 'module',
      scripts: {
        rollupExplicit: 'rollup --input src/explicit.ts --format esm',
        rollupDynamic: 'rollup -c rollup.config.js --input',
        esbuildBundle: 'esbuild --bundle ./src/esbuild-entry.ts --outfile=dist/out.js',
      },
    }));
    write(fx, 'src/explicit.ts', 'export const explicit = 1;\n');
    write(fx, 'src/esbuild-entry.ts', 'export const esbuildEntry = 1;\n');
    write(fx, 'zod-full.ts', 'export const schema = 1;\n');
    write(fx, 'rollup.config.js', 'export default {};\n');
    write(fx, 'src/internal.ts', 'export const internal = 1;\n');

    const entries = collectScripts(fx);
    const files = new Set(entries.map((e) => e.file));
    const dynamicEvidence = entries.find((e) => e.file === 'zod-full.ts')?.evidence ?? {};

    assert('PS-3e. package script rollup --input entrypoint is collected',
      files.has('src/explicit.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-3f. package script esbuild positional entrypoint is collected',
      files.has('src/esbuild-entry.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-3g. rollup dynamic --input protects root-level script-fed entrypoints',
      files.has('zod-full.ts') && !files.has('rollup.config.js') && !files.has('src/internal.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-3h. dynamic rollup evidence is marked explicitly',
      dynamicEvidence.tool === 'rollup' &&
        dynamicEvidence.dynamicInput === true &&
        dynamicEvidence.scriptName === 'rollupDynamic',
      JSON.stringify(dynamicEvidence));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'public-surface-html-'));
  try {
    write(fx, 'package.json', JSON.stringify({
      name: 'html-entrypoints',
      type: 'module',
    }));
    write(fx, 'index.html',
      `<div id="app"></div>\n` +
      `<script type="module" src="/src/main.ts"></script>\n` +
      `<script src="/src/legacy.ts"></script>\n`);
    write(fx, 'src/main.ts', 'export default {};\n');
    write(fx, 'src/legacy.ts', 'export default {};\n');

    const entries = collectHtml(fx);
    const files = new Set(entries.map((e) => e.file));
    const evidence = entries.find((e) => e.file === 'src/main.ts')?.evidence ?? {};

    assert('PS-4a. HTML module script entrypoint is collected',
      files.has('src/main.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-4b. non-module script is ignored',
      !files.has('src/legacy.ts'),
      `files=${[...files].sort().join(', ')}`);
    assert('PS-4c. HTML entrypoint evidence records htmlFile',
      evidence.source === 'html-module-script' &&
        evidence.htmlFile === 'index.html',
      JSON.stringify(evidence));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
