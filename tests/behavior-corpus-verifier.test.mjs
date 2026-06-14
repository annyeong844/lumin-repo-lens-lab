import { execFileSync, spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { afterAll, describe, expect, it } from 'vitest';

import { createTempRepoFixture } from './_helpers/temp-repo-fixture.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const CLI = path.join(ROOT, 'test-harness/lib/verify-behavior-corpus.mjs');
const DEFAULT_CORPUS = path.join(ROOT, 'test-harness/behavior/cases.json');
const NODE = process.execPath;
const CLI_URL = pathToFileURL(CLI).href;

function runVerifierExpression(expression, payload) {
  const code = [
    `import { verifyBehaviorAnswer, verifyBehaviorCorpus, verifyReadTrace } from ${JSON.stringify(CLI_URL)};`,
    `const payload = JSON.parse(process.env.LRL_TEST_PAYLOAD);`,
    `const result = ${expression};`,
    `process.stdout.write(JSON.stringify(result));`,
  ].join('\n');
  const stdout = execFileSync(NODE, ['--input-type=module', '--eval', code], {
    cwd: ROOT,
    encoding: 'utf8',
    env: {
      ...process.env,
      LRL_TEST_PAYLOAD: JSON.stringify(payload),
    },
  });
  return JSON.parse(stdout);
}

describe('behavior corpus verifier', () => {
  const fx = createTempRepoFixture({ prefix: 'fx-vitest-behavior-verify-' });

  afterAll(() => {
    fx.cleanup();
  });

  it('accepts plain answers with required cues and no internal jargon', () => {
    const text = [
      'I scanned 42 files and found a 4-file cycle.',
      'The next step is one focused change, then rerun the audit.',
    ].join('\n');

    const result = runVerifierExpression('verifyBehaviorAnswer(payload.text, payload.spec)', {
      text,
      spec: {
        id: 'good',
        forbidJargon: true,
        mustMatch: ['4-file cycle', 'next step'],
        maxNonEmptyLines: 5,
      },
    });

    expect(result).toMatchObject({ ok: true, failures: [] });
  });

  it('rejects normal chat answers that leak internal jargon', () => {
    const result = runVerifierExpression('verifyBehaviorAnswer(payload.text, payload.spec)', {
      text: 'SAFE_FIX says Tier C cleanup in fix-plan.json.summary.REVIEW_FIX.',
      spec: {
        id: 'jargon',
        forbidJargon: true,
      },
    });

    expect(result.ok).toBe(false);
    expect(result.failures).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'internal-jargon-leak' })])
    );
  });

  it('accepts caveated review-only dead export wording', () => {
    const result = runVerifierExpression('verifyBehaviorAnswer(payload.text, payload.spec)', {
      text: 'The candidate needs one public-surface check before demoting the export.',
      spec: {
        id: 'review-fix',
        forbidJargon: true,
        mustMatch: ['candidate', 'check'],
        mustNotMatch: ['safe to remove', 'definitely dead'],
      },
    });

    expect(result).toMatchObject({ ok: true, failures: [] });
  });

  it('rejects overconfident review-only dead export wording', () => {
    const result = runVerifierExpression('verifyBehaviorAnswer(payload.text, payload.spec)', {
      text: 'This is definitely dead and safe to remove.',
      spec: {
        id: 'overclaim',
        mustNotMatch: ['safe to remove', 'definitely dead'],
      },
    });

    expect(result.ok).toBe(false);
    expect(result.failures).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'must-not-match-present' })])
    );
  });

  it('verifies the checked-in behavior corpus', () => {
    const result = runVerifierExpression('verifyBehaviorCorpus(payload.corpusPath)', {
      corpusPath: DEFAULT_CORPUS,
    });

    expect(result.ok).toBe(true);
    expect(result.checked).toBe(11);
  });

  it('CLI exits zero for the checked-in corpus', () => {
    const stdout = execFileSync(NODE, [CLI, DEFAULT_CORPUS], {
      cwd: ROOT,
      encoding: 'utf8',
    });

    expect(stdout).toContain('[verify-behavior-corpus] OK (11 case(s))');
  });

  it('CLI exits non-zero when an expected-pass case fails', () => {
    const dir = fx.path('bad-corpus');
    fx.write('bad-corpus/answers/bad.md', 'SAFE_FIX cleanup.\n');
    fx.writeJson('bad-corpus/cases.json', {
      schemaVersion: 'lumin-behavior-corpus.v1',
      cases: [
        {
          id: 'unexpected-jargon',
          answer: 'answers/bad.md',
          forbidJargon: true,
        },
      ],
    });

    const result = spawnSync(NODE, [CLI, path.join(dir, 'cases.json')], {
      cwd: ROOT,
      encoding: 'utf8',
    });

    expect(result.status).toBe(1);
    expect(result.stderr).toContain('unexpected-failure');
  });

  it('read-trace expectations fail when a required artifact is absent', () => {
    const result = runVerifierExpression('verifyReadTrace(payload.trace, payload.spec)', {
      trace: 'read manifest.json\nread symbols.json\n',
      spec: {
        id: 'trace',
        mustReadArtifacts: ['topology.json'],
      },
    });

    expect(result.ok).toBe(false);
    expect(result.failures).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'required-artifact-not-read' })])
    );
  });
});
