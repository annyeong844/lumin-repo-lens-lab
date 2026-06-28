// Calibration corpus registry contract.

import { strict as assert } from 'node:assert';

import {
  CALIBRATION_CORPUS_SCHEMA_VERSION,
  calibrationCorpusSummary,
  getCalibrationCorpus,
  listCalibrationCorpusIds,
} from '../_lib/calibration-corpora.mjs';
import { THRESHOLD_POLICIES } from '../_lib/threshold-policies.mjs';

const ids = listCalibrationCorpusIds();
assert.ok(ids.includes('calibration-2026-05-prewrite-v1'));
assert.ok(ids.includes('calibration-2026-05-resolver-v1'));

for (const policy of Object.values(THRESHOLD_POLICIES)) {
  const corpusId = policy.calibration?.corpus;
  assert.ok(corpusId, `${policy.policyId} must name a calibration corpus`);
  const corpus = getCalibrationCorpus(corpusId);
  assert.equal(corpus.schemaVersion, CALIBRATION_CORPUS_SCHEMA_VERSION);
  assert.equal(corpus.corpusId, corpusId);
  assert.ok(corpus.metrics.length > 0, `${corpusId} must define metric names`);
}

const prewrite = getCalibrationCorpus('calibration-2026-05-prewrite-v1');
assert.equal(prewrite.purpose, 'pre-write cue and threshold calibration');
assert.ok(prewrite.metrics.includes('precisionProxy'));
assert.ok(prewrite.metrics.includes('noiseRate'));

const resolver = getCalibrationCorpus('calibration-2026-05-resolver-v1');
assert.equal(resolver.purpose, 'resolver blind-zone and completeness calibration');
assert.ok(resolver.metrics.includes('unresolvedInternalRate'));
assert.ok(resolver.metrics.includes('falseGlobalBlockerCount'));

const summary = calibrationCorpusSummary([
  'calibration-2026-05-prewrite-v1',
  'calibration-2026-05-resolver-v1',
]);
assert.deepEqual(summary.map((item) => item.corpusId), [
  'calibration-2026-05-prewrite-v1',
  'calibration-2026-05-resolver-v1',
]);
assert.ok(!('notes' in summary[0]), 'summary should stay compact');

assert.throws(
  () => getCalibrationCorpus('missing-corpus'),
  /Unknown calibration corpus: missing-corpus/
);

console.log('calibration corpus registry tests passed');
