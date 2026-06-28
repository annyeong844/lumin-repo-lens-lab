import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

import { createTempRepoFixture } from './_helpers/temp-repo-fixture.mjs';

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

function assertThrowsPathGuard(label, fn) {
  assert.throws(fn, /fixture path/i, label);
}

check('TRF1. fixture creates root, output, and default package.json', () => {
  const fx = createTempRepoFixture({ prefix: 'lrl-temp-helper-defaults-' });
  try {
    assert.ok(existsSync(fx.root), fx.root);
    assert.ok(existsSync(fx.output), fx.output);
    assert.deepEqual(fx.readJson('package.json'), {
      name: 'fixture',
      private: true,
      type: 'module',
    });
  } finally {
    fx.cleanup();
  }
});

check('TRF2. nested text files round-trip and create parents', () => {
  const fx = createTempRepoFixture({ prefix: 'lrl-temp-helper-write-' });
  try {
    fx.write('src/nested/file.ts', 'export const value = 1;\n');
    assert.equal(fx.read('src/nested/file.ts'), 'export const value = 1;\n');
    assert.ok(existsSync(path.join(fx.root, 'src', 'nested')));
  } finally {
    fx.cleanup();
  }
});

check('TRF3. root JSON writes with a trailing newline and reads back', () => {
  const fx = createTempRepoFixture({ prefix: 'lrl-temp-helper-json-' });
  try {
    fx.writeJson('tsconfig.json', { compilerOptions: { baseUrl: '.' } });
    assert.deepEqual(fx.readJson('tsconfig.json'), {
      compilerOptions: { baseUrl: '.' },
    });
    assert.ok(readFileSync(fx.path('tsconfig.json'), 'utf8').endsWith('\n'));
  } finally {
    fx.cleanup();
  }
});

check('TRF4. output JSON writes and reads from the output root', () => {
  const fx = createTempRepoFixture({ prefix: 'lrl-temp-helper-output-' });
  try {
    fx.writeJson('symbols.json', { schemaVersion: 'symbols.v1' }, { to: 'output' });
    assert.deepEqual(fx.readJson('symbols.json', { from: 'output' }), {
      schemaVersion: 'symbols.v1',
    });
    assert.ok(existsSync(fx.outputPath('symbols.json')));
  } finally {
    fx.cleanup();
  }
});

check('TRF5. unsafe paths are rejected before write/read resolution', () => {
  const fx = createTempRepoFixture({ prefix: 'lrl-temp-helper-paths-' });
  try {
    const unsafe = [
      '',
      '/absolute.ts',
      'C:/absolute.ts',
      'C:\\absolute.ts',
      'C:drive-relative.ts',
      '../outside.ts',
      'src/../../outside.ts',
      'src/bad\0name.ts',
    ];
    for (const rel of unsafe) {
      assertThrowsPathGuard(`write ${JSON.stringify(rel)}`, () => fx.write(rel, 'x\n'));
      assertThrowsPathGuard(`path ${JSON.stringify(rel)}`, () => fx.path(rel));
      assertThrowsPathGuard(`outputPath ${JSON.stringify(rel)}`, () => fx.outputPath(rel));
    }
    assert.throws(
      () => fx.write('safe.txt', 'x\n', { to: 'elsewhere' }),
      /unsupported fixture location/
    );
    assert.throws(
      () => fx.read('safe.txt', { from: 'elsewhere' }),
      /unsupported fixture location/
    );
  } finally {
    fx.cleanup();
  }
});

check('TRF6. cleanup removes only the helper-created fixture root', () => {
  const fx = createTempRepoFixture({ prefix: 'lrl-temp-helper-cleanup-' });
  const root = fx.root;
  const output = fx.output;
  fx.write('src/file.ts', 'export const value = 1;\n');
  assert.ok(existsSync(root));
  assert.ok(existsSync(output));

  fx.cleanup();

  assert.equal(existsSync(root), false);
  assert.equal(existsSync(output), false);
});

if (failed > 0) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, 0 failed`);
