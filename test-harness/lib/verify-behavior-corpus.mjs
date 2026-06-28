#!/usr/bin/env node
// Offline behavior verifier for saved model answers.
//
// This is not live telemetry and does not call a model. Maintainers save
// representative answers and optional read traces, then this checker verifies
// behavior-level contracts such as "plain mode does not leak internal jargon",
// "review-only dead exports are not described as automatic deletions", and
// "claims that require a cold artifact actually read that artifact".

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const INTERNAL_JARGON = [
  ['tier-summary', /\b(?:SAFE_FIX|REVIEW_FIX|DEGRADED|MUTED)\b/],
  ['tier-letter', /\bTier\s+[A-Z]\b/i],
  ['hca-phase', /\bHCA-\d+\b/i],
  ['phase-code', /\bP[0-6]\b/],
  ['fp-family', /\b(?:FP-?\d+|publicApi_FP\d+)\b/i],
  ['shape-field', /\bshape\.(?:hash|typeLiteral|fields)\b/i],
  ['domain-cluster', /\bDOMAIN_CLUSTER_DETECTED\b/],
  ['duplicate-label', /\b(?:DUPLICATE_STRONG|DUPLICATE_REVIEW|ANY_COLLISION|LOCAL_COMMON_NAME)\b/],
  ['single-owner-label', /\bsingle-owner-(?:strong|weak)\b/i],
  ['raw-json-path', /\b(?:checklist-facts|fix-plan|topology|symbols|discipline|barrels|call-graph|manifest)\.json\.[A-Za-z0-9_[\].'":-]+/i],
];

function add(failures, code, detail) {
  failures.push({ code, detail });
}

function compileRegex(source, field, id) {
  if (typeof source !== 'string' || source.length === 0) {
    throw new Error(`${id}.${field} entries must be non-empty regex strings`);
  }
  try {
    return new RegExp(source, 'im');
  } catch (e) {
    throw new Error(`${id}.${field} contains invalid regex "${source}": ${e.message}`);
  }
}

function lineCount(text) {
  return text.replace(/\r\n/g, '\n').split('\n').filter((line) => line.trim()).length;
}

export function verifyBehaviorAnswer(text, spec = {}) {
  const failures = [];
  const id = spec.id ?? '<anonymous>';

  if (typeof text !== 'string' || text.trim() === '') {
    add(failures, 'empty-answer', `${id}: answer text is empty`);
    return { ok: false, failures };
  }

  for (const patternSource of spec.mustMatch ?? []) {
    const pattern = compileRegex(patternSource, 'mustMatch', id);
    if (!pattern.test(text)) {
      add(failures, 'must-match-missing', `${id}: expected pattern not found: ${patternSource}`);
    }
  }

  for (const patternSource of spec.mustNotMatch ?? []) {
    const pattern = compileRegex(patternSource, 'mustNotMatch', id);
    const match = text.match(pattern);
    if (match) {
      add(failures, 'must-not-match-present', `${id}: forbidden pattern matched "${match[0]}": ${patternSource}`);
    }
  }

  if (spec.forbidJargon === true) {
    for (const [code, pattern] of INTERNAL_JARGON) {
      const match = text.match(pattern);
      if (match) {
        add(failures, 'internal-jargon-leak', `${id}: ${code} leaked as "${match[0]}"`);
      }
    }
  }

  if (Number.isInteger(spec.minNonEmptyLines) && lineCount(text) < spec.minNonEmptyLines) {
    add(failures, 'too-short', `${id}: expected at least ${spec.minNonEmptyLines} non-empty lines`);
  }
  if (Number.isInteger(spec.maxNonEmptyLines) && lineCount(text) > spec.maxNonEmptyLines) {
    add(failures, 'too-long', `${id}: expected at most ${spec.maxNonEmptyLines} non-empty lines`);
  }

  return { ok: failures.length === 0, failures };
}

export function verifyReadTrace(traceText, spec = {}) {
  const failures = [];
  const id = spec.id ?? '<anonymous>';

  for (const artifact of spec.mustReadArtifacts ?? []) {
    if (typeof artifact !== 'string' || artifact.length === 0) {
      add(failures, 'invalid-read-artifact', `${id}: mustReadArtifacts entries must be non-empty strings`);
      continue;
    }
    if (!traceText.includes(artifact)) {
      add(failures, 'required-artifact-not-read', `${id}: read trace does not include ${artifact}`);
    }
  }

  for (const artifact of spec.mustNotReadArtifacts ?? []) {
    if (typeof artifact !== 'string' || artifact.length === 0) {
      add(failures, 'invalid-read-artifact', `${id}: mustNotReadArtifacts entries must be non-empty strings`);
      continue;
    }
    if (traceText.includes(artifact)) {
      add(failures, 'forbidden-artifact-read', `${id}: read trace includes forbidden artifact ${artifact}`);
    }
  }

  return { ok: failures.length === 0, failures };
}

function loadCorpus(corpusPath) {
  if (!existsSync(corpusPath)) {
    throw new Error(`corpus not found: ${corpusPath}`);
  }
  const corpus = JSON.parse(readFileSync(corpusPath, 'utf8'));
  if (corpus.schemaVersion !== 'lumin-behavior-corpus.v1') {
    throw new Error(`unsupported schemaVersion: ${corpus.schemaVersion ?? '<missing>'}`);
  }
  if (!Array.isArray(corpus.cases)) {
    throw new Error('corpus must contain a "cases" array');
  }
  return corpus;
}

export function verifyBehaviorCorpus(corpusPath) {
  const absCorpus = path.resolve(corpusPath);
  const baseDir = path.dirname(absCorpus);
  const corpus = loadCorpus(absCorpus);
  const results = [];
  const failures = [];

  for (const spec of corpus.cases) {
    const id = spec.id ?? '<missing-id>';
    if (!spec.id) {
      failures.push({ id, code: 'missing-id', detail: 'case is missing id' });
      continue;
    }
    if (!spec.answer) {
      failures.push({ id, code: 'missing-answer', detail: `${id}: case is missing answer path` });
      continue;
    }

    const answerPath = path.resolve(baseDir, spec.answer);
    if (!existsSync(answerPath)) {
      failures.push({ id, code: 'answer-not-found', detail: `${id}: ${answerPath}` });
      continue;
    }

    const result = verifyBehaviorAnswer(readFileSync(answerPath, 'utf8'), spec);
    if ((spec.mustReadArtifacts?.length ?? 0) > 0 || (spec.mustNotReadArtifacts?.length ?? 0) > 0) {
      if (!spec.trace) {
        result.failures.push({
          code: 'trace-missing',
          detail: `${id}: case declares artifact-read expectations but has no trace path`,
        });
      } else {
        const tracePath = path.resolve(baseDir, spec.trace);
        if (!existsSync(tracePath)) {
          result.failures.push({ code: 'trace-not-found', detail: `${id}: ${tracePath}` });
        } else {
          result.failures.push(...verifyReadTrace(readFileSync(tracePath, 'utf8'), spec).failures);
        }
      }
      result.ok = result.failures.length === 0;
    }
    const expectPass = spec.expectPass !== false;
    const behavedAsExpected = expectPass ? result.ok : !result.ok;
    results.push({ id, expectPass, result, behavedAsExpected });

    if (!behavedAsExpected) {
      const code = expectPass ? 'unexpected-failure' : 'expected-failure-passed';
      const detail = expectPass
        ? `${id}: ${result.failures.map((f) => `${f.code}: ${f.detail}`).join('; ')}`
        : `${id}: expected verifier failure, but answer passed`;
      failures.push({ id, code, detail });
    }
  }

  return {
    ok: failures.length === 0,
    checked: results.length,
    failures,
    results,
  };
}

function usage() {
  return [
    'usage: node test-harness/lib/verify-behavior-corpus.mjs <cases.json>',
    '',
    'Checks saved model-answer fixtures without calling a model.',
    'Negative fixtures may set "expectPass": false to pin verifier behavior.',
  ].join('\n');
}

function main(argv) {
  if (argv.includes('--help') || argv.includes('-h')) {
    console.log(usage());
    return 0;
  }
  const corpusPath = argv[0];
  if (!corpusPath) {
    console.error('[verify-behavior-corpus] missing cases.json');
    console.error(usage());
    return 2;
  }

  let result;
  try {
    result = verifyBehaviorCorpus(corpusPath);
  } catch (e) {
    console.error(`[verify-behavior-corpus] ${e.message}`);
    return 2;
  }

  for (const item of result.results) {
    const status = item.behavedAsExpected ? 'PASS' : 'FAIL';
    const polarity = item.expectPass ? 'should pass' : 'should fail';
    console.log(`[verify-behavior-corpus] ${status} ${item.id} (${polarity})`);
    if (!item.behavedAsExpected) {
      for (const failure of item.result.failures) {
        console.log(`  - ${failure.code}: ${failure.detail}`);
      }
    }
  }

  if (result.ok) {
    console.log(`[verify-behavior-corpus] OK (${result.checked} case(s))`);
    return 0;
  }

  console.error('[verify-behavior-corpus] FAIL');
  for (const failure of result.failures) {
    console.error(`- ${failure.code}: ${failure.detail}`);
  }
  return 1;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exit(main(process.argv.slice(2)));
}
