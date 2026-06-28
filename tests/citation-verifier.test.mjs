import { execFileSync, spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { describe, expect, it } from 'vitest';

import { createTempRepoFixture } from './_helpers/temp-repo-fixture.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(ROOT, 'test-harness/lib/verify-citations.mjs');
const CLI_URL = pathToFileURL(CLI).href;

function createCitationFixture() {
  const fx = createTempRepoFixture({
    prefix: 'fx-vitest-citation-',
    packageJson: {
      dependencies: { dayjs: '1.0.0' },
      private: true,
      type: 'module',
    },
  });

  fx.writeJson('topology.json', {
    summary: { sccCount: 0 },
    nodes: {
      'src/a.ts': { loc: 12 },
    },
    largestFiles: [
      { file: 'src/a.ts', loc: 12 },
      { file: 'src/b.ts', loc: 8 },
    ],
  }, { to: 'output' });

  fx.writeJson('checklist-facts.json', {
    A2_function_size: {
      buckets: { big: 4, medium: 4, small: 491 },
    },
  }, { to: 'output' });

  fx.writeJson('symbols.json', {
    fanInByIdentity: {
      'src/utils/date.ts::formatDate': 8,
    },
    deadProdList: [{ name: 'a' }, { name: 'b' }, { name: 'c' }],
  }, { to: 'output' });

  return fx;
}

function withCitationFixture(fn) {
  const fx = createCitationFixture();
  try {
    return fn(fx);
  } finally {
    fx.cleanup();
  }
}

function verifyInFixture(fx, text) {
  const code = [
    `import { verifyGroundedCitations } from ${JSON.stringify(CLI_URL)};`,
    'const payload = JSON.parse(process.env.LRL_TEST_PAYLOAD);',
    'const result = verifyGroundedCitations(payload.text, { artifactsDir: payload.artifactsDir, rootDir: payload.rootDir });',
    'process.stdout.write(JSON.stringify(result));',
  ].join('\n');
  const stdout = execFileSync(NODE, ['--input-type=module', '--eval', code], {
    cwd: ROOT,
    encoding: 'utf8',
    env: {
      ...process.env,
      LRL_TEST_PAYLOAD: JSON.stringify({
        artifactsDir: fx.output,
        rootDir: fx.root,
        text,
      }),
    },
  });
  return JSON.parse(stdout);
}

function runCli(args, input) {
  return spawnSync(NODE, [CLI, ...args], {
    cwd: ROOT,
    encoding: 'utf8',
    input,
  });
}

describe('citation verifier', () => {
  it('accepts valid scalar, bracket, length, object, and root package citations', () => {
    withCitationFixture((fx) => {
      const text = [
        '- Cycles are clear [grounded, topology.json.summary.sccCount = 0]',
        "- Fan-in is known [grounded, symbols.json.fanInByIdentity['src/utils/date.ts::formatDate'] = 8]",
        '- Length works [grounded, symbols.json.deadProdList.length = 3]',
        '- Object literals work [grounded, checklist-facts.json.A2_function_size.buckets = {big: 4, medium: 4, small: 491}]',
        "- Root package fallback works [grounded, package.json.dependencies['dayjs'] = '1.0.0']",
      ].join('\n');

      const result = verifyInFixture(fx, text);

      expect(result.ok).toBe(true);
      expect(result.checked).toBe(5);
      expect(result.citationsFound).toBe(5);
    });
  });

  it('rejects mismatched grounded citation values', () => {
    withCitationFixture((fx) => {
      const result = verifyInFixture(
        fx,
        'Wrong value [grounded, topology.json.summary.sccCount = 1]',
      );

      expect(result.ok).toBe(false);
      expect(result.failures.some((failure) => failure.code === 'value-mismatch')).toBe(true);
    });
  });

  it('rejects grounded citations without a falsifiable path assignment', () => {
    withCitationFixture((fx) => {
      const result = verifyInFixture(fx, 'Unfalsifiable [grounded, source: topology.json]');

      expect(result.ok).toBe(false);
      expect(
        result.failures.some((failure) => failure.code === 'unfalsifiable-grounded-citation'),
      ).toBe(true);
    });
  });

  it('rejects missing artifact paths', () => {
    withCitationFixture((fx) => {
      const result = verifyInFixture(
        fx,
        'Missing path [grounded, topology.json.summary.nope = 0]',
      );

      expect(result.ok).toBe(false);
      expect(result.failures.some((failure) => failure.code === 'artifact-path-missing')).toBe(true);
    });
  });

  it('rejects placeholder expected values', () => {
    withCitationFixture((fx) => {
      const result = verifyInFixture(
        fx,
        'Placeholder [grounded, topology.json.summary.sccCount = N]',
      );

      expect(result.ok).toBe(false);
      expect(
        result.failures.some((failure) => failure.code === 'expected-value-uncheckable'),
      ).toBe(true);
    });
  });

  it('warns for trailing unverified clauses after validating the first assignment', () => {
    withCitationFixture((fx) => {
      const result = verifyInFixture(
        fx,
        'Extra clause [grounded, topology.json.summary.sccCount = 0, lens = runtime]',
      );

      expect(result.ok).toBe(true);
      expect(result.checked).toBe(1);
      expect(result.warnings.some((warning) => warning.code === 'trailing-unverified-clause')).toBe(true);
    });
  });

  it('CLI exits 0 for a valid citation file', () => {
    withCitationFixture((fx) => {
      const goodFile = fx.write('good.md', 'OK [grounded, topology.json.summary.sccCount = 0]\n');

      const result = runCli(['--artifacts', fx.output, goodFile]);

      expect(result.status).toBe(0);
      expect(result.stdout).toContain('[verify-citations] OK');
    });
  });

  it('CLI exits 1 for a mismatched citation file', () => {
    withCitationFixture((fx) => {
      const badFile = fx.write('bad.md', 'Bad [grounded, topology.json.summary.sccCount = 9]\n');

      const result = runCli(['--artifacts', fx.output, badFile]);

      expect(result.status).toBe(1);
      expect(result.stderr).toContain('value-mismatch');
    });
  });

  it('CLI reads Markdown from stdin', () => {
    withCitationFixture((fx) => {
      const result = runCli(['--artifacts', fx.output, '-'], 'STDIN [grounded, topology.json.largestFiles[0].loc = 12]\n');

      expect(result.status).toBe(0);
      expect(result.stdout).toContain('checked 1/1');
    });
  });
});
