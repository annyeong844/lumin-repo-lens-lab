import { describe, expect, it } from 'vitest';

import { thresholdPolicyDriftSnapshot } from '../_lib/threshold-policies.mjs';

const GUARDED_POLICY_IDS = [
  'function-clone-near-policy',
  'inline-pattern-policy',
  'resolver-blind-zone-policy',
];

const EXPECTED_THRESHOLD_POLICY_DRIFT_SNAPSHOT = [
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
];

describe('threshold policy drift guard', () => {
  it('keeps the guarded threshold policies in the reviewed order', () => {
    const snapshot = thresholdPolicyDriftSnapshot(GUARDED_POLICY_IDS);

    expect(snapshot.map((item) => item.policyId)).toEqual(GUARDED_POLICY_IDS);
  });

  it('requires explicit review for threshold version, hash, or calibration drift', () => {
    const snapshot = thresholdPolicyDriftSnapshot(GUARDED_POLICY_IDS);

    expect(snapshot).toEqual(EXPECTED_THRESHOLD_POLICY_DRIFT_SNAPSHOT);
  });
});
