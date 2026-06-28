// Saved-answer behavior verifier: offline contract checks for final model
// answers. This pins the "no live telemetry, no model subprocess" harness.

import { execFileSync, spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createTempRepoFixture } from './_helpers/temp-repo-fixture.mjs';
import {
  verifyBehaviorAnswer,
  verifyBehaviorCorpus,
  verifyReadTrace,
} from '../test-harness/lib/verify-behavior-corpus.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const CLI = path.join(ROOT, 'test-harness/lib/verify-behavior-corpus.mjs');
const DEFAULT_CORPUS = path.join(ROOT, 'test-harness/behavior/cases.json');
const NODE = process.execPath;
const FX = createTempRepoFixture({ prefix: 'fx-behavior-verify-' });

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

try {
  {
    const text = [
      'I scanned 42 files and found a 4-file cycle.',
      'The next step is one focused change, then rerun the audit.',
    ].join('\n');
    const result = verifyBehaviorAnswer(text, {
      id: 'good',
      forbidJargon: true,
      mustMatch: ['4-file cycle', 'next step'],
      maxNonEmptyLines: 5,
    });
    assert('B1. plain answer with dense cue and no internal jargon passes',
      result.ok,
      JSON.stringify(result.failures));
  }

  {
    const result = verifyBehaviorAnswer('SAFE_FIX says Tier C cleanup in fix-plan.json.summary.REVIEW_FIX.', {
      id: 'jargon',
      forbidJargon: true,
    });
    assert('B2. normal chat answer with internal jargon fails',
      !result.ok && result.failures.some((e) => e.code === 'internal-jargon-leak'),
      JSON.stringify(result.failures));
  }

  {
    const result = verifyBehaviorAnswer('The candidate needs one public-surface check before demoting the export.', {
      id: 'review-fix',
      forbidJargon: true,
      mustMatch: ['candidate', 'check'],
      mustNotMatch: ['safe to remove', 'definitely dead'],
    });
    assert('B3. review-only dead export wording can pass when caveated',
      result.ok,
      JSON.stringify(result.failures));
  }

  {
    const result = verifyBehaviorAnswer('This is definitely dead and safe to remove.', {
      id: 'overclaim',
      mustNotMatch: ['safe to remove', 'definitely dead'],
    });
    assert('B4. review-only dead export overclaim fails',
      !result.ok && result.failures.some((e) => e.code === 'must-not-match-present'),
      JSON.stringify(result.failures));
  }

  {
    const result = verifyBehaviorCorpus(DEFAULT_CORPUS);
    assert('B5. checked-in behavior corpus behaves as expected',
      result.ok && result.checked === 11,
      JSON.stringify(result.failures));
  }

  {
    const stdout = execFileSync(NODE, [CLI, DEFAULT_CORPUS], {
      cwd: ROOT,
      encoding: 'utf8',
    });
    assert('B6. CLI exits 0 for checked-in corpus',
      stdout.includes('[verify-behavior-corpus] OK (11 case(s))'),
      stdout);
  }

  {
    const dir = FX.path('bad-corpus');
    FX.write('bad-corpus/answers/bad.md', 'SAFE_FIX cleanup.\n');
    const corpus = {
      schemaVersion: 'lumin-behavior-corpus.v1',
      cases: [
        {
          id: 'unexpected-jargon',
          answer: 'answers/bad.md',
          forbidJargon: true,
        },
      ],
    };
    FX.writeJson('bad-corpus/cases.json', corpus);
    const result = spawnSync(NODE, [CLI, path.join(dir, 'cases.json')], {
      cwd: ROOT,
      encoding: 'utf8',
    });
    assert('B7. CLI exits non-zero when an expected-pass case fails',
      result.status === 1 && result.stderr.includes('unexpected-failure'),
      `${result.status}\n${result.stdout}\n${result.stderr}`);
  }

  {
    const result = verifyReadTrace('read manifest.json\nread symbols.json\n', {
      id: 'trace',
      mustReadArtifacts: ['topology.json'],
    });
    assert('B8. read-trace expectations fail when a required artifact is absent',
      !result.ok && result.failures.some((e) => e.code === 'required-artifact-not-read'),
      JSON.stringify(result.failures));
  }
} finally {
  FX.cleanup();
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
