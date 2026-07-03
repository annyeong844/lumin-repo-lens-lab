import { execFileSync } from 'node:child_process';
import { cpSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { createTempRepoFixture } from './_helpers/temp-repo-fixture.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE = process.execPath;

function createReadmeFixture() {
  const fx = createTempRepoFixture({
    packageJson: { type: 'module' },
    prefix: 'fx-vitest-test-doc-',
  });

  cpSync(path.join(ROOT, 'CHANGELOG.md'), fx.path('CHANGELOG.md'));
  cpSync(path.join(ROOT, 'scripts'), fx.path('scripts'), { recursive: true });
  cpSync(path.join(ROOT, 'tests'), fx.path('tests'), { recursive: true });

  return {
    fx,
    generator: fx.path('scripts/update-test-doc.mjs'),
    readme: fx.path('tests/README.md'),
  };
}

function withReadmeFixture(fn) {
  const fixture = createReadmeFixture();
  try {
    return fn(fixture);
  } finally {
    fixture.fx.cleanup();
  }
}

function runGenerator(fixture, ...args) {
  try {
    const stdout = execFileSync(NODE, [fixture.generator, ...args], {
      cwd: fixture.fx.root,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    return { ok: true, out: stdout };
  } catch (error) {
    return {
      ok: false,
      out: `${error.stdout || ''}${error.stderr || ''}`,
    };
  }
}

describe('generated tests README', () => {
  it('passes check mode when the fixture README is in sync', () => {
    withReadmeFixture((fixture) => {
      const result = runGenerator(fixture, '--check');

      expect(result.ok).toBe(true);
      expect(result.out).toContain('up to date');
    });
  });

  it('fails check mode on README drift and points at regeneration', () => {
    withReadmeFixture((fixture) => {
      const original = fixture.fx.read('tests/README.md');
      fixture.fx.write('tests/README.md', `${original}\n<!-- injected drift -->\n`);

      const result = runGenerator(fixture, '--check');

      expect(result.ok).toBe(false);
      expect(result.out).toContain('DRIFT');
      expect(result.out).toContain('update-test-doc');
    });
  });

  it('regenerates drifted README content and then passes check mode', () => {
    withReadmeFixture((fixture) => {
      fixture.fx.write('tests/README.md', `${fixture.fx.read('tests/README.md')}\n<!-- drift -->\n`);

      const regenerate = runGenerator(fixture);
      const check = runGenerator(fixture, '--check');

      expect(regenerate.ok).toBe(true);
      expect(regenerate.out).toContain('wrote');
      expect(check.ok).toBe(true);
    });
  });

  it('keeps the do-not-edit marker and avoids authoritative assertion counts', () => {
    withReadmeFixture((fixture) => {
      const regenerate = runGenerator(fixture);
      const content = readFileSync(fixture.readme, 'utf8');
      const countPatterns = [
        /\*\*v\d+\.\d+\.\d+\*\*\s*\(\d+\)/,
        /\*\*\d+\s+assertions\*\*/i,
        /\*\*total:?\s+\d+\*\*/i,
        /\d+\s+assertions\s+across\s+\d+\s+suites/i,
      ];

      expect(regenerate.ok).toBe(true);
      expect(content).toContain('GENERATED FILE');
      expect(content).toContain('do not edit');
      expect(countPatterns.find((pattern) => pattern.test(content))).toBeUndefined();
    });
  });

  it('documents legacy umbrella suites outside the default npm test gate', () => {
    withReadmeFixture((fixture) => {
      const regenerate = runGenerator(fixture);
      const content = readFileSync(fixture.readme, 'utf8');

      expect(regenerate.ok).toBe(true);
      expect(content).toContain('## Legacy Umbrella Suites');
      expect(content).toContain('npm run test:node:legacy-audit-repo');
      expect(content).toContain('excluded\nfrom `npm test`');
      expect(content).not.toContain('node tests/test-audit-repo.mjs');
    });
  });

  it('surfaces a maintainer note for a suite without a description', () => {
    withReadmeFixture((fixture) => {
      fixture.fx.write('tests/test-z-zz-temp.mjs', 'console.log("fixture-only test doc probe");\n');

      const regenerate = runGenerator(fixture);
      const content = readFileSync(fixture.readme, 'utf8');

      expect(regenerate.ok).toBe(true);
      expect(content).toContain('## Maintainer note');
      expect(content).toContain('test-z-zz-temp.mjs');
    });
  });

  it('gives pre-write suites explicit generated README descriptions', () => {
    withReadmeFixture((fixture) => {
      const regenerate = runGenerator(fixture);
      const content = readFileSync(fixture.readme, 'utf8');
      const preWriteLines = content
        .split('\n')
        .filter((line) => line.includes('tests/test-pre-write-'));
      const missingPreWriteDescriptions = preWriteLines.filter((line) =>
        line.includes('(no description'));

      expect(regenerate.ok).toBe(true);
      expect(missingPreWriteDescriptions).toEqual([]);
    });
  });

  it('gives every current suite an explicit generated README description', () => {
    withReadmeFixture((fixture) => {
      const regenerate = runGenerator(fixture);
      const content = readFileSync(fixture.readme, 'utf8');
      const missingDescriptionLines = content
        .split('\n')
        .filter((line) => line.includes('(no description'));

      expect(regenerate.ok).toBe(true);
      expect(missingDescriptionLines).toEqual([]);
    });
  });

  it('regenerates only the fixture README, not the real repo README', () => {
    const realReadme = path.join(ROOT, 'tests/README.md');
    const before = readFileSync(realReadme, 'utf8');

    withReadmeFixture((fixture) => {
      fixture.fx.write('tests/README.md', `${fixture.fx.read('tests/README.md')}\n<!-- fixture drift -->\n`);

      const regenerate = runGenerator(fixture);

      expect(regenerate.ok).toBe(true);
      expect(readFileSync(fixture.readme, 'utf8')).not.toContain('fixture drift');
    });

    expect(readFileSync(realReadme, 'utf8')).toBe(before);
  });
});
