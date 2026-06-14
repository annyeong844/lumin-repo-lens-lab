import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  buildUnusedDepsArtifact,
  collectPackageScriptToolEvidence,
  packageNameFromSpecifier,
} from '../_lib/unused-deps-artifact.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

let passed = 0;
let failed = 0;

function check(label, fn) {
  try {
    fn();
    passed++;
    console.log(`  PASS  ${label}`);
  } catch (error) {
    failed++;
    console.log(`  FAIL  ${label}\n        ${error?.message ?? error}`);
  }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function makeFixture() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-unused-deps-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  write(dir, 'package.json', JSON.stringify({
    name: 'unused-deps-fixture',
    private: true,
    type: 'module',
    scripts: {
      start: 'tsx src/server.ts',
      dev: 'vite --host 0.0.0.0',
      lint: 'eslint .',
      wrapped: 'npm run start',
    },
    dependencies: {
      react: '^19.0.0',
      'left-pad': '^1.3.0',
    },
    devDependencies: {
      tsx: '^4.0.0',
      vite: '^7.0.0',
      eslint: '^9.0.0',
      '@types/node': '^22.0.0',
    },
    peerDependencies: {
      '@storybook/react': '^8.0.0',
    },
    optionalDependencies: {
      fsevents: '^2.3.0',
    },
  }, null, 2));
  write(dir, 'src/app.tsx', 'import React from "react";\nexport const App = React.Fragment;\n');
  write(dir, 'src/server.ts', 'export const server = true;\n');
  return { dir, out };
}

function runAudit() {
  const { dir, out } = makeFixture();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'audit-repo.mjs'),
      '--root', dir,
      '--output', out,
      '--profile', 'quick',
    ], { encoding: 'utf8' });
    return {
      artifact: JSON.parse(readFileSync(path.join(out, 'unused-deps.json'), 'utf8')),
      manifest: JSON.parse(readFileSync(path.join(out, 'manifest.json'), 'utf8')),
    };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function depByName(pkg, name) {
  return pkg.dependencies.find((entry) => entry.name === name);
}

check('UD1. normalizes external package specifiers and rejects non-packages', () => {
  assert.equal(packageNameFromSpecifier('react'), 'react');
  assert.equal(packageNameFromSpecifier('react/jsx-runtime'), 'react');
  assert.equal(packageNameFromSpecifier('@scope/pkg/sub/path'), '@scope/pkg');
  assert.equal(packageNameFromSpecifier('node:fs'), null);
  assert.equal(packageNameFromSpecifier('./local'), null);
  assert.equal(packageNameFromSpecifier('../local'), null);
  assert.equal(packageNameFromSpecifier('/abs/local'), null);
  assert.equal(packageNameFromSpecifier('C:/abs/local'), null);
  assert.equal(packageNameFromSpecifier('https://cdn.example/pkg.js'), null);
  assert.equal(packageNameFromSpecifier('data:text/javascript,export{}'), null);
  assert.equal(packageNameFromSpecifier('#internal'), null);
  assert.equal(packageNameFromSpecifier('virtual:foo'), null);
  assert.equal(packageNameFromSpecifier('@broken'), null);
  assert.equal(packageNameFromSpecifier(''), null);
  assert.equal(packageNameFromSpecifier(null), null);
});

check('UD2. extracts direct package script tool evidence without following wrappers', () => {
  const packageRecord = {
    root: 'C:/repo',
    relRoot: '.',
    packageJson: {
      scripts: {
        start: 'tsx src/server.ts',
        dev: 'vite --host 0.0.0.0',
        lint: 'pnpm eslint .',
        bunvite: 'bunx vite build',
        npxlint: 'npx eslint .',
        npmexec: 'npm exec eslint .',
        npmstart: 'npm start',
        npmtest: 'npm test',
        wrapped: 'npm run start',
      },
    },
  };
  const evidence = collectPackageScriptToolEvidence(packageRecord);
  const keys = evidence.map((entry) => `${entry.tool}:${entry.scriptName}`).sort();
  assert.deepEqual(keys, [
    'eslint:lint',
    'eslint:npmexec',
    'eslint:npxlint',
    'tsx:start',
    'vite:bunvite',
    'vite:dev',
  ]);
  assert.equal(evidence.some((entry) => entry.scriptName === 'wrapped'), false);
  assert.equal(evidence.some((entry) => entry.scriptName === 'npmstart'), false);
  assert.equal(evidence.some((entry) => entry.scriptName === 'npmtest'), false);
});

check('UD3. classifies used, muted, and review-unused dependencies deterministically', () => {
  const artifact = buildUnusedDepsArtifact({
    root: 'C:/repo',
    includeTests: true,
    exclude: [],
    packageRecords: [{
      root: 'C:/repo',
      relRoot: '.',
      packageJson: {
        name: 'app',
        scripts: { start: 'tsx src/server.ts' },
        dependencies: { react: '^19.0.0', 'left-pad': '^1.3.0' },
        devDependencies: { tsx: '^4.0.0', '@types/node': '^22.0.0' },
        peerDependencies: { '@storybook/react': '^8.0.0' },
        optionalDependencies: { fsevents: '^2.3.0' },
      },
    }],
    symbols: {
      meta: { supports: { dependencyImportConsumers: true } },
      dependencyImportConsumers: [
        { file: 'src/app.tsx', fromSpec: 'react/jsx-runtime', depRoot: 'react', kind: 'import', source: 'source-import' },
      ],
    },
  });

  assert.equal(artifact.schemaVersion, 'unused-deps.v1');
  assert.equal(artifact.policyVersion, 'unused-deps-review-policy-v1');
  assert.equal(artifact.status, 'complete');
  assert.deepEqual(artifact.scanRange, {
    root: 'C:/repo',
    includeTests: true,
    exclude: [],
    source: 'producer-cli',
  });
  assert.equal(artifact.summary.packageCount, 1);
  assert.equal(artifact.summary.declaredDependencyCount, 6);
  assert.equal(artifact.summary.usedCount, 1);
  assert.equal(artifact.summary.mutedCount, 4);
  assert.equal(artifact.summary.reviewUnusedCount, 1);

  const pkg = artifact.packages[0];
  assert.equal(depByName(pkg, 'react').status, 'used');
  assert.equal(depByName(pkg, 'react').reason, 'external-import-consumer');
  assert.equal(depByName(pkg, 'left-pad').status, 'review-unused');
  assert.equal(depByName(pkg, 'left-pad').reason, 'no-observed-consumer');
  assert.equal(depByName(pkg, 'tsx').status, 'muted');
  assert.equal(depByName(pkg, 'tsx').reason, 'package-script-tool');
  assert.equal(depByName(pkg, '@types/node').status, 'muted');
  assert.equal(depByName(pkg, '@types/node').reason, 'ambient-types');
  assert.equal(depByName(pkg, '@storybook/react').status, 'muted');
  assert.equal(depByName(pkg, '@storybook/react').reason, 'peer-contract');
  assert.equal(depByName(pkg, 'fsevents').status, 'muted');
  assert.equal(depByName(pkg, 'fsevents').reason, 'optional-runtime');
});

check('UD4. attributes consumers to the nearest workspace package and mutes workspace internals', () => {
  const artifact = buildUnusedDepsArtifact({
    root: 'C:/repo',
    includeTests: true,
    exclude: [],
    packageRecords: [
      {
        root: 'C:/repo',
        relRoot: '.',
        packageJson: {
          name: 'root-app',
          dependencies: {
            react: '^19.0.0',
            '@repo/shared': 'workspace:*',
          },
        },
      },
      {
        root: 'C:/repo/packages/app',
        relRoot: 'packages/app',
        packageJson: {
          name: '@repo/app',
          dependencies: {
            react: '^19.0.0',
            '@repo/shared': 'workspace:*',
          },
        },
      },
      {
        root: 'C:/repo/packages/shared',
        relRoot: 'packages/shared',
        packageJson: {
          name: '@repo/shared',
        },
      },
    ],
    symbols: {
      meta: { supports: { dependencyImportConsumers: true } },
      dependencyImportConsumers: [
        { file: 'packages/app/src/App.tsx', fromSpec: 'react', depRoot: 'react', kind: 'import', source: 'source-import' },
      ],
    },
  });

  const rootPkg = artifact.packages.find((entry) => entry.packageDir === '.');
  const appPkg = artifact.packages.find((entry) => entry.packageDir === 'packages/app');
  assert.equal(depByName(rootPkg, 'react').status, 'review-unused');
  assert.equal(depByName(rootPkg, 'react').reason, 'no-observed-consumer');
  assert.equal(depByName(appPkg, 'react').status, 'used');
  assert.equal(depByName(appPkg, '@repo/shared').status, 'muted');
  assert.equal(depByName(appPkg, '@repo/shared').reason, 'workspace-internal');
});

check('UD5. unsupported dependency import consumer lane writes unavailable artifact', () => {
  const artifact = buildUnusedDepsArtifact({
    root: 'C:/repo',
    includeTests: true,
    exclude: [],
    packageRecords: [{
      root: 'C:/repo',
      relRoot: '.',
      packageJson: { name: 'app', dependencies: { react: '^19.0.0' } },
    }],
    symbols: {
      meta: { supports: {} },
      dependencyImportConsumers: [],
    },
  });

  assert.equal(artifact.status, 'unavailable');
  assert.equal(artifact.reason, 'input-artifact-missing');
  assert.equal(artifact.inputs.symbols.supportsDependencyImportConsumers, false);
  assert.equal(artifact.summary.declaredDependencyCount, 0);
  assert.deepEqual(artifact.packages, []);
});

check('UD6. audit-repo emits unused-deps.json and records it as produced', () => {
  const { artifact, manifest } = runAudit();
  assert.equal(artifact.schemaVersion, 'unused-deps.v1');
  assert.equal(artifact.status, 'complete');
  assert.ok(manifest.commandsRun.some((entry) =>
    entry.step === 'build-unused-deps.mjs' &&
    entry.status === 'ok'));
  assert.ok(manifest.artifactsProduced.includes('unused-deps.json'));
  assert.equal(manifest.unusedDependencies?.artifact, 'unused-deps.json');
  assert.equal(manifest.unusedDependencies?.schemaVersion, 'unused-deps.v1');
  assert.equal(manifest.unusedDependencies?.policyVersion, 'unused-deps-review-policy-v1');
  assert.equal(manifest.unusedDependencies?.status, 'complete');
  assert.equal(manifest.unusedDependencies?.reviewUnusedCount, 1);
  assert.equal(manifest.unusedDependencies?.mutedCount, 6);
  assert.deepEqual(manifest.unusedDependencies?.topReviewUnused, [
    {
      packageDir: '.',
      manifestPath: 'package.json',
      name: 'left-pad',
      field: 'dependencies',
      reason: 'no-observed-consumer',
      confidence: 'review',
    },
  ]);
  const pkg = artifact.packages[0];
  assert.equal(depByName(pkg, 'react').status, 'used');
  assert.equal(depByName(pkg, 'left-pad').status, 'review-unused');
  assert.equal(depByName(pkg, 'tsx').status, 'muted');
  assert.equal(depByName(pkg, 'vite').status, 'muted');
  assert.equal(depByName(pkg, 'eslint').status, 'muted');
});

if (failed > 0) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, 0 failed`);
