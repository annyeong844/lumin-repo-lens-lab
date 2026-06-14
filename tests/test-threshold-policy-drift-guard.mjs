// Threshold policy drift guard.

import { strict as assert } from 'node:assert';

import { thresholdPolicyDriftSnapshot } from '../_lib/threshold-policies.mjs';

const snapshot = thresholdPolicyDriftSnapshot([
  'function-clone-near-policy',
  'inline-pattern-policy',
  'resolver-blind-zone-policy',
]);

assert.deepEqual(snapshot.map((item) => item.policyId), [
  'function-clone-near-policy',
  'inline-pattern-policy',
  'resolver-blind-zone-policy',
]);

assert.deepEqual(snapshot, [
  {
    policyId: 'function-clone-near-policy',
    policyVersion: 'function-clone-near-policy-v1',
    policyClass: 'review',
    thresholdHash: 'sha256:ba963d4a06d50a37633a99576aeda79230ad8870878802ac66942d82cf9459da',
    calibrationCorpus: 'calibration-2026-05-prewrite-v1',
    calibrationNote: 'agent-entry resolver calibration threshold contract',
  },
  {
    policyId: 'inline-pattern-policy',
    policyVersion: 'inline-pattern-policy-v1',
    policyClass: 'review',
    thresholdHash: 'sha256:d78e2ad5095b375535ce08e70de769a7356bab3e9b185a37790388794652c6b3',
    calibrationCorpus: 'calibration-2026-05-prewrite-v1',
    calibrationNote: 'pre-write inline extraction cues spec',
  },
  {
    policyId: 'resolver-blind-zone-policy',
    policyVersion: 'resolver-blind-zone-policy-v1',
    policyClass: 'confidence',
    thresholdHash: 'sha256:21c9c0517943eeb1457da9bde82fdcdc8edcffba794375fadb6f0eaa113e4e6d',
    calibrationCorpus: 'calibration-2026-05-resolver-v1',
    calibrationNote: 'agent-entry resolver completeness contract',
  },
]);

console.log('threshold policy drift guard tests passed');
