import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { createTempRepoFixture } from './_helpers/temp-repo-fixture.mjs';

function expectPathGuard(fn) {
  expect(fn).toThrow(/fixture path/i);
}

describe('temp repo fixture helper', () => {
  it('creates root, output, and default package.json', () => {
    const fx = createTempRepoFixture({ prefix: 'lrl-vitest-temp-helper-defaults-' });
    try {
      expect(existsSync(fx.root)).toBe(true);
      expect(existsSync(fx.output)).toBe(true);
      expect(fx.readJson('package.json')).toEqual({
        name: 'fixture',
        private: true,
        type: 'module',
      });
    } finally {
      fx.cleanup();
    }
  });

  it('round-trips nested text files and creates parent directories', () => {
    const fx = createTempRepoFixture({ prefix: 'lrl-vitest-temp-helper-write-' });
    try {
      fx.write('src/nested/file.ts', 'export const value = 1;\n');
      expect(fx.read('src/nested/file.ts')).toBe('export const value = 1;\n');
      expect(existsSync(path.join(fx.root, 'src', 'nested'))).toBe(true);
    } finally {
      fx.cleanup();
    }
  });

  it('writes root JSON with a trailing newline and reads it back', () => {
    const fx = createTempRepoFixture({ prefix: 'lrl-vitest-temp-helper-json-' });
    try {
      fx.writeJson('tsconfig.json', { compilerOptions: { baseUrl: '.' } });
      expect(fx.readJson('tsconfig.json')).toEqual({
        compilerOptions: { baseUrl: '.' },
      });
      expect(readFileSync(fx.path('tsconfig.json'), 'utf8').endsWith('\n')).toBe(true);
    } finally {
      fx.cleanup();
    }
  });

  it('writes and reads JSON from the output root', () => {
    const fx = createTempRepoFixture({ prefix: 'lrl-vitest-temp-helper-output-' });
    try {
      fx.writeJson('symbols.json', { schemaVersion: 'symbols.v1' }, { to: 'output' });
      expect(fx.readJson('symbols.json', { from: 'output' })).toEqual({
        schemaVersion: 'symbols.v1',
      });
      expect(existsSync(fx.outputPath('symbols.json'))).toBe(true);
    } finally {
      fx.cleanup();
    }
  });

  it('rejects unsafe paths before write/read resolution', () => {
    const fx = createTempRepoFixture({ prefix: 'lrl-vitest-temp-helper-paths-' });
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
        expectPathGuard(() => fx.write(rel, 'x\n'));
        expectPathGuard(() => fx.path(rel));
        expectPathGuard(() => fx.outputPath(rel));
      }
      expect(() => fx.write('safe.txt', 'x\n', { to: 'elsewhere' })).toThrow(
        /unsupported fixture location/
      );
      expect(() => fx.read('safe.txt', { from: 'elsewhere' })).toThrow(
        /unsupported fixture location/
      );
    } finally {
      fx.cleanup();
    }
  });

  it('cleanup removes only the helper-created fixture root', () => {
    const fx = createTempRepoFixture({ prefix: 'lrl-vitest-temp-helper-cleanup-' });
    const root = fx.root;
    const output = fx.output;
    fx.write('src/file.ts', 'export const value = 1;\n');

    expect(existsSync(root)).toBe(true);
    expect(existsSync(output)).toBe(true);

    fx.cleanup();

    expect(existsSync(root)).toBe(false);
    expect(existsSync(output)).toBe(false);
  });
});
