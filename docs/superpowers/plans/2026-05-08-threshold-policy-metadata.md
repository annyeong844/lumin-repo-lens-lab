# Threshold Policy Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make user-visible heuristic thresholds machine-readable through named policy metadata so they are no longer invisible magic numbers.

**Architecture:** Add one shared threshold-policy module that owns policy ids, versions, classes, calibration anchors, and threshold values. Wire the first slice into artifacts that already expose threshold-shaped behavior: `function-clones.json`, `inline-patterns.json`, and resolver blind-zone details in `manifest.json`. Keep behavior unchanged; this slice only makes existing thresholds explicit.

**Tech Stack:** Node.js ESM, existing JSON artifacts, existing custom test harness style.

---

### Task 1: Add RED Tests For Policy Metadata

**Files:**
- Modify: `tests/test-build-function-clone-index.mjs`
- Modify: `tests/test-inline-pattern-index.mjs`
- Modify: `tests/test-audit-repo.mjs`
- Create: `tests/test-threshold-policies.mjs`
- Modify: `scripts/update-test-doc.mjs`

- [ ] **Step 1: Add a pure policy test**

Create `tests/test-threshold-policies.mjs` that imports the new module expected in Task 2:

```js
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

const inlinePolicy = getThresholdPolicy('inline-pattern-policy');
assert.equal(inlinePolicy.thresholds.minOccurrences, 3);
assert.equal(inlinePolicy.thresholds.maxCatchStatements, 2);

const resolverPolicy = getThresholdPolicy('resolver-blind-zone-policy');
assert.equal(resolverPolicy.policyClass, 'confidence');
assert.equal(resolverPolicy.thresholds.unresolvedRatio, 0.15);
assert.equal(resolverPolicy.thresholds.absoluteUnresolvedCount, 1000);

const summary = thresholdPolicySummary([
  'function-clone-near-policy',
  'inline-pattern-policy',
]);
assert.deepEqual(summary.map((p) => p.policyId), [
  'function-clone-near-policy',
  'inline-pattern-policy',
]);
```

- [ ] **Step 2: Add artifact metadata assertions**

Extend existing tests:

```js
// tests/test-build-function-clone-index.mjs
assert('function clone artifact exposes near-candidate threshold policy',
  index.meta.thresholdPolicies?.some((policy) =>
    policy.policyId === 'function-clone-near-policy' &&
    policy.policyVersion === 'function-clone-near-policy-v1' &&
    policy.thresholds?.minNearScore === 0.62),
  JSON.stringify(index.meta.thresholdPolicies, null, 2));

// tests/test-inline-pattern-index.mjs
assert('inline-pattern artifact exposes inline threshold policy',
  index.meta.thresholdPolicies?.some((policy) =>
    policy.policyId === 'inline-pattern-policy' &&
    policy.thresholds?.minOccurrences === 3 &&
    policy.thresholds?.maxCatchStatements === 2),
  JSON.stringify(index.meta.thresholdPolicies, null, 2));

// tests/test-audit-repo.mjs resolver blind-zone block
assert('resolver blind-zone details expose threshold policy metadata',
  r.details?.thresholdPolicy?.policyId === 'resolver-blind-zone-policy' &&
    r.details.thresholdPolicy.thresholds?.unresolvedRatio === 0.15,
  JSON.stringify(r.details, null, 2));
```

- [ ] **Step 3: Run tests and verify RED**

Run:

```bash
node tests/test-threshold-policies.mjs
node tests/test-build-function-clone-index.mjs
node tests/test-inline-pattern-index.mjs
node tests/test-audit-repo.mjs
```

Expected: failures because `_lib/threshold-policies.mjs` and `meta.thresholdPolicies` do not exist yet.

### Task 2: Implement Shared Threshold Policies

**Files:**
- Create: `_lib/threshold-policies.mjs`
- Modify: `_lib/function-clone-artifact.mjs`
- Modify: `_lib/inline-pattern-artifact.mjs`
- Modify: `_lib/blind-zones.mjs`

- [ ] **Step 1: Create `_lib/threshold-policies.mjs`**

The module should export:

```js
export const THRESHOLD_POLICY_SCHEMA_VERSION = 'threshold-policy.v1';
export const THRESHOLD_POLICIES = Object.freeze({ ... });
export function getThresholdPolicy(policyId) { ... }
export function thresholdPolicySummary(policyIds) { ... }
```

Each policy must include:

- `schemaVersion`
- `policyId`
- `policyVersion`
- `policyClass`
- `policyHash`
- `thresholds`
- `calibration`
- `notes`

Use a deterministic SHA-256 hash over the canonical policy content excluding `policyHash`.

- [ ] **Step 2: Replace local threshold constants with policy-derived constants**

Use `getThresholdPolicy(...)` at module top level:

```js
const INLINE_PATTERN_POLICY = getThresholdPolicy('inline-pattern-policy');
const MIN_OCCURRENCES = INLINE_PATTERN_POLICY.thresholds.minOccurrences;
```

Do the same for function near-candidate thresholds and resolver blind-zone thresholds.

- [ ] **Step 3: Attach policy summaries to public artifacts**

Add:

```js
thresholdPolicies: thresholdPolicySummary(['function-clone-near-policy'])
```

to `function-clones.json.meta`, and:

```js
thresholdPolicies: thresholdPolicySummary(['inline-pattern-policy'])
```

to `inline-patterns.json.meta`.

For resolver blind zones, add a compact single `thresholdPolicy` object inside `zone.details`, because blind zones are embedded in `manifest.json` rather than written as a dedicated artifact.

### Task 3: Verification And Mirror

**Files:**
- Generated mirror: `skills/lumin-repo-lens-lab/_engine/**`
- Test docs: `tests/README.md`

- [ ] **Step 1: Run focused tests**

Run:

```bash
node tests/test-threshold-policies.mjs
node tests/test-build-function-clone-index.mjs
node tests/test-inline-pattern-index.mjs
node tests/test-audit-repo.mjs
```

- [ ] **Step 2: Update test docs**

Run:

```bash
npm run update-test-doc
npm run check:test-doc
```

- [ ] **Step 3: Rebuild skill mirror**

Run:

```bash
npm run build:skill
npm run check:public-plugin
```

- [ ] **Step 4: Focused syntax/lint**

Run:

```bash
npm run check
npm run lint
```

- [ ] **Step 5: Commit**

Run:

```bash
git add _lib/threshold-policies.mjs _lib/function-clone-artifact.mjs _lib/inline-pattern-artifact.mjs _lib/blind-zones.mjs tests/test-threshold-policies.mjs tests/test-build-function-clone-index.mjs tests/test-inline-pattern-index.mjs tests/test-audit-repo.mjs tests/README.md skills/lumin-repo-lens-lab
git commit -m "Expose threshold policy metadata"
```
