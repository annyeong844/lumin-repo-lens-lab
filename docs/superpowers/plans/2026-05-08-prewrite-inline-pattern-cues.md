# Pre-Write Inline Pattern Cues Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect `inline-patterns.json` to `/lumin-repo-lens-lab:pre-write` so explicit `intent.refactorSources` can surface repeated inline catch-block patterns as `AGENT_REVIEW_CUE`.

**Architecture:** Keep the inline pattern producer separate from pre-write. Add a focused lookup adapter that reads `inline-patterns.json`, intersects pattern occurrences with explicit `refactorSources`, and passes a normal pre-write lookup into the cue-tier classifier. Missing artifact support reports `UNAVAILABLE`; no heuristic nearby-file surfacing is added in this slice.

**Tech Stack:** Node.js ESM scripts, built-in `node:test`, existing pre-write JSON/Markdown renderer, existing cold-cache preflight runner.

---

## File Structure

- Modify `_lib/pre-write-intent.mjs`
  - Accept optional `refactorSources`.
  - Validate repository-relative files and optional positive integer lines.
- Create `_lib/pre-write-lookup-inline-patterns.mjs`
  - Convert `refactorSources` plus `inline-patterns.json` into one pre-write lookup.
  - Match by file and optional line-range intersection.
- Modify `_lib/pre-write-cue-tiers.mjs`
  - Accept `lookup.kind === 'inline-pattern'`.
  - Emit `AGENT_REVIEW_CUE` for matched inline pattern groups.
  - Emit lane-level `UNAVAILABLE` when the artifact is missing.
- Modify `_lib/pre-write-render.mjs`
  - Use inline-specific review wording for `inline-extraction` cues.
- Modify `_lib/pre-write-cold-cache.mjs`
  - Include `build-inline-pattern-index.mjs` when the intent contains `refactorSources`.
- Modify `pre-write.mjs`
  - Load `inline-patterns.json`.
  - Run the inline-pattern lookup when `refactorSources` are present.
- Add tests:
  - `tests/test-pre-write-inline-patterns.mjs`
  - Extend `tests/test-pre-write-cue-tiers.mjs`
  - Extend `tests/test-pre-write-intent.mjs`
- Update generated test docs and skill mirror after code is green.

---

### Task 1: Intent Contract

**Files:**
- Modify: `_lib/pre-write-intent.mjs`
- Test: `tests/test-pre-write-intent.mjs`

- [ ] **Step 1: Write the failing validation tests**

Add tests showing that a valid `refactorSources` array is preserved and invalid paths/lines are rejected:

```js
const withRefactorSources = validatePreWriteIntent({
  names: ['writeOrDestroyConnection'],
  shapes: [],
  files: ['src/connection-write.ts'],
  dependencies: [],
  plannedTypeEscapes: [],
  refactorSources: [
    {
      file: 'src/server.ts',
      lines: [498, 577, 661, 689],
      why: 'extract repeated catch-destroy handling',
    },
  ],
});

assert.deepEqual(withRefactorSources.refactorSources, [
  {
    file: 'src/server.ts',
    lines: [498, 577, 661, 689],
    why: 'extract repeated catch-destroy handling',
  },
]);

assert.throws(
  () => validatePreWriteIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
    refactorSources: [{ file: '../server.ts', lines: [1] }],
  }),
  /refactorSources\[0\]\.file/,
);

assert.throws(
  () => validatePreWriteIntent({
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
    refactorSources: [{ file: 'src/server.ts', lines: [0] }],
  }),
  /refactorSources\[0\]\.lines\[0\]/,
);
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-pre-write-intent.mjs
```

Expected: FAIL because `refactorSources` is not validated or preserved yet.

- [ ] **Step 3: Implement minimal validation**

Add a `normalizeRefactorSourceEntry(entry, index)` helper. Rules:

- `file` must be a non-empty repository-relative path.
- Reject absolute paths, drive-letter paths, `..`, and backslashes.
- `lines`, when present, must be a non-empty array of positive integers.
- `why`, when present, must be a non-empty string.

Only include `refactorSources` in normalized output when the input includes it.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-pre-write-intent.mjs
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add _lib/pre-write-intent.mjs tests/test-pre-write-intent.mjs
git commit -m "feat: accept pre-write refactor sources"
```

---

### Task 2: Inline Pattern Lookup And Cue Classification

**Files:**
- Create: `_lib/pre-write-lookup-inline-patterns.mjs`
- Modify: `_lib/pre-write-cue-tiers.mjs`
- Test: `tests/test-pre-write-cue-tiers.mjs`

- [ ] **Step 1: Write failing classifier tests**

Add a test that an inline pattern lookup emits only review evidence:

```js
const result = classifyPreWriteCues({
  intent: { names: ['writeOrDestroyConnection'] },
  lookups: [
    {
      kind: 'inline-pattern',
      result: 'INLINE_PATTERN_MATCH',
      groups: [
        {
          patternHash: 'sha256:catch-destroy',
          kind: 'catch-block',
          size: 4,
          ownerFiles: ['src/server.ts'],
          occurrences: [
            { file: 'src/server.ts', line: 498, endLine: 500 },
            { file: 'src/server.ts', line: 577, endLine: 579 },
          ],
          reviewReason: 'same normalized catch block; verify socket ownership before extracting',
        },
      ],
    },
  ],
});

assert.equal(result.cueCards.length, 1);
const cue = result.cueCards[0].cues[0];
assert.equal(cue.cueTier, CUE_TIERS.AGENT_REVIEW);
assert.equal(cue.evidenceLane, 'inline-extraction');
assert.equal(cue.claim, 'repeated inline statement pattern');
assert.equal(cue.evidence[0].artifact, 'inline-patterns.json');
assert.equal(cue.evidence[0].occurrenceCount, 4);
```

Add a second test for missing artifacts:

```js
const missing = classifyPreWriteCues({
  intent: {},
  lookups: [
    {
      kind: 'inline-pattern',
      result: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'inline-patterns.json',
    },
  ],
});

assert.equal(missing.unavailableEvidence[0].evidenceLane, 'inline-extraction');
assert.equal(missing.unavailableEvidence[0].status, UNAVAILABLE_STATUS);
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-pre-write-cue-tiers.mjs
```

Expected: FAIL because `inline-pattern` lookups are ignored.

- [ ] **Step 3: Create lookup helper**

Implement:

```js
export function lookupInlinePatterns(refactorSources, { inlinePatterns } = {}) {
  if (!Array.isArray(refactorSources) || refactorSources.length === 0) {
    return { kind: 'inline-pattern', result: 'NO_INLINE_PATTERN_INTENT', groups: [] };
  }

  if (!inlinePatterns || !Array.isArray(inlinePatterns.groups)) {
    return {
      kind: 'inline-pattern',
      result: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'inline-patterns.json',
    };
  }

  // Match groups whose occurrence file intersects a refactorSource file and,
  // when lines are provided, whose occurrence range contains one of those lines.
}
```

Sort matched groups by size descending, then `patternHash`.

- [ ] **Step 4: Add cue-tier handling**

In `_lib/pre-write-cue-tiers.mjs`, add an `addInlinePatternLookup` branch:

- `INLINE_PATTERN_MATCH` creates an `AGENT_REVIEW_CUE`.
- `UNAVAILABLE` appends `unavailableEvidence`.
- `NO_INLINE_PATTERN_MATCH` and `NO_INLINE_PATTERN_INTENT` produce no cards.

The cue must not use `SAFE_CUE`.

- [ ] **Step 5: Verify GREEN**

Run:

```bash
node tests/test-pre-write-cue-tiers.mjs
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add _lib/pre-write-lookup-inline-patterns.mjs _lib/pre-write-cue-tiers.mjs tests/test-pre-write-cue-tiers.mjs
git commit -m "feat: classify inline extraction cues"
```

---

### Task 3: Pre-Write CLI Integration

**Files:**
- Modify: `pre-write.mjs`
- Modify: `_lib/pre-write-cold-cache.mjs`
- Modify: `_lib/pre-write-render.mjs`
- Test: `tests/test-pre-write-inline-patterns.mjs`

- [ ] **Step 1: Write the end-to-end failing test**

Create a fixture repo in the test with one `src/server.ts` containing four repeated:

```ts
try {
  writeWebSocketTextMessage(connection.socket, payload);
} catch {
  connection.socket.destroy();
}
```

Write an intent file:

```json
{
  "names": ["writeOrDestroyConnection", "WriteOrDestroyResult"],
  "shapes": [],
  "files": ["src/connection-write.ts"],
  "dependencies": [],
  "plannedTypeEscapes": [],
  "refactorSources": [
    {
      "file": "src/server.ts",
      "lines": [4, 10, 16, 22],
      "why": "extract repeated catch-destroy handling"
    }
  ]
}
```

Run `pre-write.mjs` against the fixture and assert:

- `inline-patterns.json` was produced by cold preflight.
- JSON output contains a cue card with `evidenceLane: "inline-extraction"`.
- Markdown contains `Agent review cues`.
- Markdown contains `repeated inline statement pattern`.
- Markdown does not contain `Safe to extract` or `Duplicate behavior found`.

Add a second test with `--no-fresh-audit` and no `inline-patterns.json`:

- JSON output contains `unavailableEvidence` for `inline-extraction`.
- Markdown does not invent a review cue.

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-pre-write-inline-patterns.mjs
```

Expected: FAIL because `pre-write.mjs` does not load or request `inline-patterns.json`.

- [ ] **Step 3: Wire cold-cache preflight**

In `_lib/pre-write-cold-cache.mjs`:

- Add an `INLINE_PATTERNS_PRODUCER` for `build-inline-pattern-index.mjs`.
- Include it when `intent.refactorSources` has entries.
- Keep producer ordering deterministic after the existing function-clones producer.

- [ ] **Step 4: Wire pre-write lookup**

In `pre-write.mjs`:

- Import `lookupInlinePatterns`.
- Load `inline-patterns.json` via the existing `loadIfExists`.
- Push the inline-pattern lookup when `intent.refactorSources` has entries.

- [ ] **Step 5: Add renderer wording**

In `_lib/pre-write-render.mjs`, keep the existing review cue section but use lane-specific action text:

```text
action: inspect the cited occurrence ranges before extracting helper code.
```

only for `inline-extraction`.

- [ ] **Step 6: Verify GREEN**

Run:

```bash
node tests/test-pre-write-inline-patterns.mjs
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add pre-write.mjs _lib/pre-write-cold-cache.mjs _lib/pre-write-render.mjs tests/test-pre-write-inline-patterns.mjs
git commit -m "feat: surface inline patterns in pre-write"
```

---

### Task 4: Docs, Mirror, And Verification

**Files:**
- Modify: `docs/spec/pre-write-inline-extraction-cues.md`
- Modify: generated test docs if needed
- Modify: `skills/lumin-repo-lens-lab/**` through `npm run build:skill`

- [ ] **Step 1: Update implementation status**

In `docs/spec/pre-write-inline-extraction-cues.md`, update:

- Status from implementation deferred to P2 implemented.
- Phase list so P1 and P2 are marked implemented.
- Open question answer that v1 requires explicit `refactorSources`.

- [ ] **Step 2: Regenerate docs and skill mirror**

Run:

```bash
npm run update-test-doc
npm run build:skill
```

Expected: generated docs and skill mirror update without errors.

- [ ] **Step 3: Run focused verification**

Run:

```bash
node tests/test-pre-write-intent.mjs
node tests/test-pre-write-cue-tiers.mjs
node tests/test-pre-write-inline-patterns.mjs
npm run check
npm run lint
npm run check:test-doc
npm run check:drift
npm run check:public-plugin
npm run check:doc-script-refs
```

Expected: all PASS.

- [ ] **Step 4: Run broad local verification**

Run:

```bash
npm test
```

Expected: all test suites PASS.

- [ ] **Step 5: Commit**

```bash
git add docs/spec/pre-write-inline-extraction-cues.md docs tests skills
git commit -m "docs: mark inline pre-write cues implemented"
```

---

### Task 5: PR Preparation

**Files:**
- No new source files unless verification finds an issue.

- [ ] **Step 1: Check branch state**

Run:

```bash
git status --short --branch
git log --oneline --max-count=5
```

Expected: branch contains only commits for this PR.

- [ ] **Step 2: Push and open draft PR**

Run:

```bash
git push -u origin codex/prewrite-inline-pattern-cues
```

Open a draft PR titled:

```text
Add pre-write inline extraction cues
```

PR body should mention:

- `refactorSources` intent support.
- `inline-patterns.json` cold-cache preflight.
- `AGENT_REVIEW_CUE`, not `SAFE_CUE`.
- Missing artifact `UNAVAILABLE`.
- Local verification commands.

- [ ] **Step 3: Report to user**

Report:

- PR URL.
- Verification results.
- Whether GitHub Actions were intentionally skipped or unavailable.

---

## Self-Review

- Spec coverage: P2 requirements are covered by Tasks 1-3; docs/mirror/verification by Task 4; PR by Task 5.
- Scope: This plan does not add heuristic nearby-file inline cue surfacing. It requires explicit `refactorSources`, matching the conservative v1 decision.
- Safety: Inline evidence is review-only. No blocking behavior and no safe extraction claim are introduced.
