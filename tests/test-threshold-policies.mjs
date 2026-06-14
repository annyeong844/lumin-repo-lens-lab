// Threshold policy metadata contract.

import { strict as assert } from 'node:assert';

import {
  THRESHOLD_POLICY_SCHEMA_VERSION,
  getThresholdPolicy,
  thresholdPolicySummary,
} from '../_lib/threshold-policies.mjs';

const functionPolicy = getThresholdPolicy('function-clone-near-policy');
assert.equal(functionPolicy.schemaVersion, THRESHOLD_POLICY_SCHEMA_VERSION);
assert.equal(functionPolicy.policyId, 'function-clone-near-policy');
assert.equal(functionPolicy.policyVersion, 'function-clone-near-policy-v1');
assert.equal(functionPolicy.policyClass, 'review');
assert.equal(functionPolicy.thresholds.minNearScore, 0.62);
assert.equal(functionPolicy.thresholds.maxNearCandidates, 50);
assert.ok(/^sha256:[a-f0-9]{64}$/.test(functionPolicy.policyHash));
assert.ok(/^sha256:[a-f0-9]{64}$/.test(functionPolicy.thresholdHash));

const inlinePolicy = getThresholdPolicy('inline-pattern-policy');
assert.equal(inlinePolicy.policyClass, 'review');
assert.equal(inlinePolicy.thresholds.minOccurrences, 3);
assert.equal(inlinePolicy.thresholds.maxCatchStatements, 2);

const resolverPolicy = getThresholdPolicy('resolver-blind-zone-policy');
assert.equal(resolverPolicy.policyClass, 'confidence');
assert.equal(resolverPolicy.thresholds.unresolvedRatio, 0.15);
assert.equal(resolverPolicy.thresholds.absoluteUnresolvedCount, 1000);
assert.equal(resolverPolicy.thresholds.prefixConcentrationMinUnresolved, 100);
assert.equal(resolverPolicy.thresholds.prefixConcentrationShare, 0.8);

const summary = thresholdPolicySummary([
  'function-clone-near-policy',
  'inline-pattern-policy',
]);
assert.deepEqual(summary.map((p) => p.policyId), [
  'function-clone-near-policy',
  'inline-pattern-policy',
]);
assert.equal(summary[0].thresholds.minNearScore, 0.62);
assert.equal(summary[0].thresholdHash, functionPolicy.thresholdHash);
assert.equal(summary[0].calibrationCorpus?.corpusId, 'calibration-2026-05-prewrite-v1');
assert.equal(summary[0].calibrationCorpus?.entryCount, 3);
assert.ok(summary[0].calibrationCorpus?.metrics?.includes('precisionProxy'));
assert.ok(!('notes' in summary[0]), 'summary should stay compact');

console.log('threshold policy metadata tests passed');
